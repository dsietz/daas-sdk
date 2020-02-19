use super::*;
use crate::doc::*;
use crate::eventing::broker::{DaaSKafkaProcessor, DaaSKafkaBroker};
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;

pub struct DaaSProcessorMessage<'a> {
        offset: i64,
        key: &'a [u8],
        doc: DaaSDoc,
}

pub trait DaaSProcessorService {
    fn keep_listening(rx: &Receiver<bool>) -> bool;
    fn start_listening(consumer: Consumer, rx: &Receiver<bool>, callback: fn(DaaSProcessorMessage));
    fn stop_listening(controller: &Sender<bool>);
}

pub trait DaaSGenesisProcessorService {
    fn provision_document(msg: DaaSProcessorMessage) {
        // 1. Store the DaaSDoc in S3 Bucket
        error!("Putting document {} in S3", msg.doc._id);
        // 2. Broker the DaaSDoc based on dynamic topic
        error!("Brokering document to topic {}", DaaSKafkaBroker::make_topic(msg.doc));
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
                DaaSProcessor::start_listening(consumer, &rx, DaasGenesisProcessor::provision_document);
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
    
    fn start_listening(mut consumer: Consumer, rx: &Receiver<bool>, callback: fn(DaaSProcessorMessage)) {
        while DaaSProcessor::keep_listening(rx) {
            for messageset in consumer.poll().unwrap().iter() {
                for message in messageset.messages() {
                    callback(DaaSProcessorMessage {
                        offset: message.offset,
                        key: message.key,
                        doc: DaaSDoc:: from_serialized(message.value),
                    });
                }
                match consumer.consume_messageset(messageset) {
                    Ok(_c) => {},
                    Err(err) => panic!("{}",err),
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
        let topic = format!("{}", get_unix_now!()).to_string();
        let consumer = Consumer::from_hosts(vec!("localhost:9092".to_string()))
                            .with_topic(topic.clone())
                            .with_fallback_offset(FetchOffset::Earliest)
                            .with_group(format!("{}-consumers", topic.clone()).to_string())
                            .with_offset_storage(GroupOffsetStorage::Kafka)
                            .create()
                            .unwrap();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        let my_broker = DaaSKafkaBroker::default();
        let (tx, rx) = channel();

        assert!(my_broker.broker_message(&mut my_doc, &topic).is_ok());
        
        let _handler = thread::spawn(move || {
            DaaSProcessor::start_listening(consumer, &rx, |msg|{
                assert_eq!(msg.doc._id, "order~clothing~iStore~15000".to_string());
            });
        });
                    
        thread::sleep(Duration::from_secs(5));
        DaaSProcessor::stop_listening(&tx);
    }
}
