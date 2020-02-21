use super::*;
use crate::doc::*;
use crate::errors::daaserror::{DaaSProcessingError};
use crate::eventing::broker::{DaaSKafkaProcessor, DaaSKafkaBroker};
use crate::storage::s3::*;
use rusoto_core::Region;
use rusoto_s3::{StreamingBody};
use kafka::client::KafkaClient;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;

pub struct DaaSProcessorMessage<'a> {
        offset: i64,
        key: &'a [u8],
        doc: DaaSDoc,
        topic: &'a str,
}

pub trait DaaSProcessorService {
    fn keep_listening(rx: &Receiver<bool>) -> bool;
    fn start_listening<T>(consumer: Consumer, rx: &Receiver<bool>, o: Option<&T>, callback: fn(DaaSProcessorMessage, Option<KafkaClient>, Option<&T>) -> Result<i32, DaaSProcessingError>);
    fn stop_listening(controller: &Sender<bool>);
}

pub trait DaaSGenesisProcessorService {
    fn default_topics(doc: &DaaSDoc) -> Vec<String> {
        let mut topics = Vec::new();
        topics.push(DaaSKafkaBroker::make_topic(doc.clone()));

        topics
    }

    fn broker_document(client: KafkaClient, doc: DaaSDoc, send_to: Option<Vec<String>>) -> Result<i32, DaaSProcessingError>{
        let hosts = client.hosts().to_vec();

        // if a send to topic is not provided, then use the default topics
        let topics = match send_to {
            Some(t) => t,
            None => {
                let v = Self::default_topics(&doc);
                v
            },
        };

        for topic in topics.iter() {
            match DaaSKafkaBroker::broker_message_with_client(KafkaClient::new(hosts.clone()), &mut doc.clone(), &topic.clone()) {
                Ok(_v) => {},
                Err(e) => {
                    error!("Failed to broker message to {:?}. Error: {:?}", topic, e);
                    return Err(DaaSProcessingError::BrokerError)
                }
            }
        }

        Ok(1)
    }

    fn provision_document<T: S3BucketManager + Clone>(mut msg: DaaSProcessorMessage, client: Option<KafkaClient>, s3_bucket: Option<&T>) -> Result<i32, DaaSProcessingError> {
        let send_to_topic: Option<&str> = Some("newbie");

        // 1. Store the DaaSDoc in S3 Bucket
        info!("Putting document {} in S3", msg.doc._id);

        let content: StreamingBody = msg.doc.serialize().into_bytes().into();

        match s3_bucket.unwrap().clone().upload_file(format!("{}/{}.daas", msg.topic, msg.doc._id), content) {
            Ok(_s) => {},
            Err(err) => {
                error!("Could not place DaasDoc {} in S3 storage. Error: {:?}", msg.doc._id, err);
                return Err(DaaSProcessingError::UpsertError)
            },
        }

        // 2. Broker the DaaSDoc if a Client is provided and use dynamic topic
        match client {
            Some(clnt) => {
                info!("Brokering document {} ... ", msg.doc._id);
                Self::broker_document(clnt, msg.doc.clone(), None)
            },
            None => Ok(1),
        }        
    }

    fn run(hosts: Vec<String>, fallback_offset: FetchOffset, storage: GroupOffsetStorage) -> Sender<bool>{
        let (tx, rx) = channel();
        let consumer = Consumer::from_hosts(hosts)
                                .with_topic("genesis".to_string())
                                .with_fallback_offset(fallback_offset)
                                .with_group("genesis-consumers".to_string())
                                .with_offset_storage(storage)
                                .create()
                                .unwrap();

        let _handler = thread::spawn(move || {
                DaaSProcessor::start_listening(
                    consumer, 
                    &rx, 
                    Some(&S3BucketMngr::new(Region::UsEast1, "iapp-daas-test-bucket".to_string())),
                    DaasGenesisProcessor::provision_document);
            });
        
        tx
    }

    fn stop(tx: Sender<bool>) {
        DaaSProcessor::stop_listening(&tx);
    }
}

pub struct DaaSProcessor {}

impl DaaSProcessorService for DaaSProcessor{
    fn keep_listening(rx: &Receiver<bool>) -> bool {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                info!("Shutting down DaaSProcessor ...");
                false
            },
            Err(TryRecvError::Empty) => {
                true
            } 
        }
    }
    
    fn start_listening<T>(mut consumer: Consumer, rx: &Receiver<bool>, o: Option<&T>, callback: fn(DaaSProcessorMessage, Option<KafkaClient>, Option<&T>) -> Result<i32, DaaSProcessingError>) {       
        while DaaSProcessor::keep_listening(rx) {
            for messageset in consumer.poll().unwrap().iter() {
                for message in messageset.messages() {
                    match callback( DaaSProcessorMessage {
                                        offset: message.offset,
                                        key: message.key,
                                        doc: DaaSDoc::from_serialized(message.value),
                                        topic: messageset.topic(),
                                    },
                                    Some(KafkaClient::new(consumer.client().hosts().to_vec())),
                                    o ) {
                        Ok(_i) => {
                            match consumer.consume_message(messageset.topic(),messageset.partition(),message.offset){
                                Ok(_c) => {},
                                Err(err) => panic!("{}",err),
                            }
                        },
                        Err(err) => {
                            warn!("Could not process the DaasDoc {} [topic:{}, partition:{}, offset:{}]. Error: {:?}", 
                                    DaaSDoc::from_serialized(message.value)._id,
                                    messageset.topic(),
                                    messageset.partition(),
                                    message.offset,
                                    err);
                        },
                    }
                }
            }
            consumer.commit_consumed().unwrap();
        }
    }

    fn stop_listening(controller: &Sender<bool>) {
        controller.send(true).unwrap();
    }
}

impl DaaSProcessor {}

pub struct DaasGenesisProcessor {}

impl DaaSGenesisProcessorService for DaasGenesisProcessor {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::eventing::broker::{DaaSKafkaBroker, DaaSKafkaProcessor};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_genesis_processor() {
        let _ = env_logger::builder().is_test(true).try_init();
        let my_broker = DaaSKafkaBroker::default();

        let serialized = r#"{"_id":"genesis~1","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        assert!(my_broker.broker_message(&mut my_doc, "genesis").is_ok());
        
        let serialized = r#"{"_id":"genesis~2","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        assert!(my_broker.broker_message(&mut my_doc, "genesis").is_ok());

        let stopper = DaasGenesisProcessor::run(vec!("localhost:9092".to_string()), FetchOffset::Earliest, GroupOffsetStorage::Kafka);
        thread::sleep(Duration::from_secs(5));
        DaasGenesisProcessor::stop(stopper);
    }

    #[test]
    fn test_process_data() {
        let _ = env_logger::builder().is_test(true).try_init();
        let my_broker = DaaSKafkaBroker::default();
        let topic = format!("{}", get_unix_now!());        
        
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        assert!(my_broker.broker_message(&mut my_doc, &topic).is_ok());
        
        let (tx, rx) = channel();
        let mut consumer = Consumer::from_hosts(vec!("localhost:9092".to_string()))
                            .with_topic(topic.clone())
                            .with_fallback_offset(FetchOffset::Earliest)
                            .with_group(format!("{}-consumer", topic.clone()))
                            .with_offset_storage(GroupOffsetStorage::Kafka)
                            .create()
                            .unwrap();
        
        let _handler = thread::spawn(move || {
            DaaSProcessor::start_listening(consumer, &rx, Some(&(1 as i8)), |msg: DaaSProcessorMessage, _clnt: Option<KafkaClient>, _t: Option<&i8>|{
                assert_eq!(msg.doc._id, "order~clothing~iStore~15000".to_string());
                Ok(1)
            });
        });
                    
        thread::sleep(Duration::from_secs(5));
        DaaSProcessor::stop_listening(&tx);
    }
}
