use super::*;
use crate::doc::*;
use crate::errors::daaserror::DaaSProcessingError;
use crate::eventing::broker::{DaaSKafkaBroker, DaaSKafkaProcessor};
use crate::storage::s3::*;
use futures::executor::block_on;
use kafka::client::KafkaClient;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use rusoto_s3::StreamingBody;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread;

pub struct DaaSProcessorMessage<'a> {
    pub offset: i64,
    pub key: &'a [u8],
    pub doc: DaaSDoc,
    pub topic: &'a str,
}

pub trait DaaSProcessorService {
    fn keep_listening(rx: &Receiver<bool>) -> bool;
    fn start_listening<T>(
        consumer: Consumer,
        rx: &Receiver<bool>,
        o: Option<&T>,
        callback: fn(
            DaaSProcessorMessage,
            Option<KafkaClient>,
            Option<&T>,
        ) -> Result<i32, DaaSProcessingError>,
    );
    fn stop_listening(controller: &Sender<bool>);
}

#[async_trait]
pub trait DaaSGenesisProcessorService {
    fn default_topics(doc: &DaaSDoc) -> Vec<String> {
        let mut topics = Vec::new();
        topics.push(DaaSKafkaBroker::make_topic(doc.clone()));
        topics.push(doc.category.clone());
        topics.push(format!("{}.{}", doc.category, doc.subcategory));
        topics.push(doc.source_name.clone());

        topics
    }

    fn broker_document(
        client: KafkaClient,
        doc: DaaSDoc,
        send_to: Option<Vec<String>>,
    ) -> Result<i32, DaaSProcessingError> {
        let hosts = client.hosts().to_vec();

        // if a send to topic is not provided, then use the default topics
        let topics = match send_to {
            Some(t) => t,
            None => {
                let v = Self::default_topics(&doc);
                v
            }
        };

        for topic in topics.iter() {
            match DaaSKafkaBroker::broker_message_with_client(
                KafkaClient::new(hosts.clone()),
                &mut doc.clone(),
                &topic.clone(),
            ) {
                Ok(_v) => {}
                Err(e) => {
                    error!("Failed to broker message to {:?}. Error: {:?}", topic, e);
                    return Err(DaaSProcessingError::BrokerError);
                }
            }
        }

        Ok(1)
    }

    fn provision_document<
        'a,
        T: S3BucketManager + Clone + std::marker::Send + std::marker::Sync,
    >(
        mut msg: DaaSProcessorMessage<'a>,
        client: Option<KafkaClient>,
        s3_bucket: Option<&T>,
    ) -> Result<i32, DaaSProcessingError> {
        //let send_to_topic: Option<&str> = Some("newbie");

        // 1. Store the DaaSDoc in S3 Bucket
        info!("Putting document {} in S3", msg.doc._id);

        let content: StreamingBody = msg.doc.serialize().into_bytes().into();

        match s3_bucket
            .unwrap()
            .clone()
            .upload_file(format!("{}/{}.daas", msg.topic, msg.doc._id), content)
        {
            Ok(_s) => {
                // 2. Broker the DaaSDoc if a Client is provided and use dynamic topic
                match client {
                    Some(clnt) => {
                        info!("Brokering document {} ... ", msg.doc._id);
                        // this needs to await this call
                        Self::broker_document(clnt, msg.doc.clone(), None)
                    }
                    None => Ok(1),
                }
            }
            Err(e) => {
                error!(
                    "Could not place DaasDoc {} in S3 storage. Error: {:?}",
                    msg.doc._id, e
                );
                return Err(DaaSProcessingError::UpsertError);
            }
        }
    }

    fn run(
        hosts: Vec<String>,
        fallback_offset: FetchOffset,
        group_offset: GroupOffsetStorage,
        bucket: S3BucketMngr,
    ) -> Sender<bool> {
        let (tx, rx) = channel();
        let consumer = Consumer::from_hosts(hosts)
            .with_topic("genesis".to_string())
            .with_fallback_offset(fallback_offset)
            .with_group("genesis-consumers".to_string())
            .with_offset_storage(group_offset)
            .create()
            .unwrap();

        let _handler = thread::spawn(move || {
            DaaSProcessor::start_listening(
                consumer,
                &rx,
                Some(&bucket),
                DaasGenesisProcessor::provision_document,
            );
        });

        tx
    }

    fn stop(tx: Sender<bool>) {
        DaaSProcessor::stop_listening(&tx);
    }
}

pub struct DaaSProcessor {}

impl DaaSProcessorService for DaaSProcessor {
    fn keep_listening(rx: &Receiver<bool>) -> bool {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                info!("Shutting down DaaSProcessor ...");
                false
            }
            Err(TryRecvError::Empty) => true,
        }
    }

    fn start_listening<T>(
        mut consumer: Consumer,
        rx: &Receiver<bool>,
        o: Option<&T>,
        callback: fn(
            DaaSProcessorMessage,
            Option<KafkaClient>,
            Option<&T>,
        ) -> Result<i32, DaaSProcessingError>,
    ) {
        while DaaSProcessor::keep_listening(rx) {
            for messageset in consumer.poll().unwrap().iter() {
                for message in messageset.messages() {
                    debug!("... {}", String::from_utf8(message.value.to_vec()).unwrap());

                    let document = match DaaSDoc::from_serialized(message.value) {
                        Ok(d) => d,
                        Err(err) => {
                            error!("Coud not create DaaSDoc. Error: {}", err);
                            println!("Skipping document because [{}]", err);
                            continue;
                        }
                    };
                    match callback(
                        DaaSProcessorMessage {
                            offset: message.offset,
                            key: message.key,
                            doc: document.clone(),
                            topic: messageset.topic(),
                        },
                        Some(KafkaClient::new(consumer.client().hosts().to_vec())),
                        o,
                    ) {
                        Ok(_i) => {
                            match consumer.consume_message(
                                messageset.topic(),
                                messageset.partition(),
                                message.offset,
                            ) {
                                Ok(_c) => {}
                                Err(err) => {
                                    error!("{}", err);
                                    panic!("{}", err);
                                }
                            }
                        }
                        Err(err) => {
                            warn!("Could not process the DaasDoc {} [topic:{}, partition:{}, offset:{}]. Error: {:?}", 
                                    document._id,
                                    messageset.topic(),
                                    messageset.partition(),
                                    message.offset,
                                    err);
                        }
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
    use pbd::dtc::Tracker;
    use pbd::dua::DUA;
    use rusoto_core::Region;
    use std::thread;
    use std::time::Duration;

    fn get_bucket() -> S3BucketMngr {
        S3BucketMngr::new(Region::UsEast1, "daas-test-bucket".to_string())
    }

    fn get_default_daasdoc() -> DaaSDoc {
        let src = "ButtonsRUs".to_string();
        let uid = 1212345;
        let cat = "button".to_string();
        let sub = "comedy".to_string();
        let auth = "button_app".to_string();
        let dua = get_dua();
        let dtc = get_dtc(src.clone(), uid.clone(), cat.clone(), sub.clone());
        let data = String::from(r#"{"status": "completed"}"#)
            .as_bytes()
            .to_vec();
        let doc = DaaSDoc::new(
            src.clone(),
            uid,
            cat.clone(),
            sub.clone(),
            auth.clone(),
            dua,
            dtc,
            data,
        );

        doc
    }

    fn get_dua() -> Vec<DUA> {
        let mut v = Vec::new();
        v.push(DUA {
            agreement_name: "billing".to_string(),
            location: "www.dua.org/billing.pdf".to_string(),
            agreed_dtm: 1553988607,
        });
        v
    }

    fn get_dtc(src_name: String, src_uid: usize, cat: String, subcat: String) -> Tracker {
        Tracker::new(DaaSDoc::make_id(
            cat.clone(),
            subcat.clone(),
            src_name.clone(),
            src_uid,
        ))
    }

    #[test]
    fn test_default_topics() {
        struct MySrv {}
        impl DaaSGenesisProcessorService for MySrv {}
        let topics = MySrv::default_topics(&get_default_daasdoc());
        assert_eq!(topics.len(), 4);
        assert_eq!(topics[0], "button.comedy.ButtonsRUs".to_string());
        assert_eq!(topics[1], "button".to_string());
        assert_eq!(topics[2], "button.comedy".to_string());
        assert_eq!(topics[3], "ButtonsRUs".to_string());
    }

    //can only be tested if there is access to the S3 bucket
    #[ignore]
    #[test]
    fn test_genesis_processor() {
        let _ = env_logger::builder().is_test(true).try_init();
        let my_broker = DaaSKafkaBroker::default();

        let serialized = r#"{"_id":"genesis~1","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":1582766489,"actor_id":"","previous_hash":"0"},"hash":"33962353871142597622255173163773323410","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes()).unwrap();
        assert!(my_broker.broker_message(&mut my_doc, "genesis").is_ok());

        let serialized = r#"{"_id":"genesis~2","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":1582766489,"actor_id":"","previous_hash":"0"},"hash":"33962353871142597622255173163773323410","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes()).unwrap();
        assert!(my_broker.broker_message(&mut my_doc, "genesis").is_ok());

        let stopper = DaasGenesisProcessor::run(
            vec!["localhost:9092".to_string()],
            FetchOffset::Earliest,
            GroupOffsetStorage::Kafka,
            get_bucket(),
        );
        thread::sleep(Duration::from_secs(5));
        DaasGenesisProcessor::stop(stopper);
    }

    #[test]
    fn test_process_data() {
        let _ = env_logger::builder().is_test(true).try_init();
        let my_broker = DaaSKafkaBroker::default();
        let topic = format!("{}", get_unix_now!());

        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":1582766489,"actor_id":"","previous_hash":"0"},"hash":"33962353871142597622255173163773323410","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes()).unwrap();
        assert!(my_broker.broker_message(&mut my_doc, &topic).is_ok());

        let (tx, rx) = channel();
        let consumer = Consumer::from_hosts(vec!["localhost:9092".to_string()])
            .with_topic(topic.clone())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group(format!("{}-consumer", topic.clone()))
            .with_offset_storage(GroupOffsetStorage::Kafka)
            .create()
            .unwrap();

        let _handler = thread::spawn(move || {
            DaaSProcessor::start_listening(
                consumer,
                &rx,
                Some(&(1 as i8)),
                |msg: DaaSProcessorMessage, _clnt: Option<KafkaClient>, _t: Option<&i8>| {
                    assert_eq!(msg.doc._id, "order~clothing~iStore~15000".to_string());
                    Ok(1)
                },
            );
        });

        thread::sleep(Duration::from_secs(5));
        DaaSProcessor::stop_listening(&tx);
    }
}
