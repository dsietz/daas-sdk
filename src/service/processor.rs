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

pub trait DaaSProcessorService<T> {
    fn keep_listening(rx: &Receiver<bool>) -> bool;
    fn start_listening(consumer: Consumer, rx: &Receiver<bool>, callback: fn(DaaSProcessorMessage));
    fn stop_listening(controller: &Sender<bool>);
}

pub struct DaaSProcessor {}

impl DaaSProcessorService<DaaSProcessor> for DaaSProcessor{
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

pub struct DaasGenesisProcessor {
    processor: DaaSProcessor,
}

impl DaasGenesisProcessor {
    pub fn provision_document(msg: DaaSProcessorMessage) {
        // 1. Store the DaaSDoc in S3 Bucket
        info!("Putting document {} in S3", msg.doc._id);
        // 2. Broker the DaaSDoc based on dynamic topic
        info!("Brokering document to topic {}", DaaSKafkaBroker::make_topic(msg.doc));
    }

    pub fn run(hosts: Vec<String>, fallback_offset: FetchOffset, storage: GroupOffsetStorage) -> Sender<bool>{
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


    pub fn stop(tx: Sender<bool>) {
        DaaSProcessor::stop_listening(&tx);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::eventing::broker::{DaaSKafkaBroker, DaaSKafkaProcessor};
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_genesis_processor() {
        let _ = env_logger::builder().is_test(true).try_init();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        let my_broker = DaaSKafkaBroker::default();
        
        assert!(my_broker.broker_message(&mut my_doc, "order.clothing.iStore").is_ok());

        let stopper = DaasGenesisProcessor::run(vec!("localhost:9092".to_string()), FetchOffset::Earliest, GroupOffsetStorage::Kafka);
        thread::sleep(Duration::from_secs(5));
        DaasGenesisProcessor::stop(stopper);
    }
/*
    #[test]
    fn test_process_data_new() {
        let _ = env_logger::builder().is_test(true).try_init();
        let consumer = Consumer::from_hosts(vec!("localhost:9092".to_string()))
                            .with_topic("order.clothing.iStore".to_string())
                            .with_fallback_offset(FetchOffset::Earliest)
                            .with_group("order.clothing.iStore-consumers".to_string())
                            .with_offset_storage(GroupOffsetStorage::Kafka)
                            .create()
                            .unwrap();
        let mut data_provisioner = DaaSProcessor::new(consumer);
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        let my_broker = DaaSKafkaBroker::default();
        let channel = sync_channel(1);

        assert!(my_broker.broker_message(&mut my_doc, "order.clothing.iStore").is_ok());
        
        data_provisioner.start_listening(&channel.1, |msg|{
            assert_eq!(msg.doc._id, "order~clothing~iStore~15000".to_string());
        });            
        thread::sleep(Duration::from_secs(5));
        data_provisioner.stop_listening(&channel.0);

    }

    #[test]
    fn test_process_data_default() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut data_provisioner = DaaSProcessor::default();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        let my_broker = DaaSKafkaBroker::default();
        let channel = sync_channel(1);

        assert!(my_broker.broker_message(&mut my_doc, "genesis").is_ok());
        
        data_provisioner.start_listening(&channel.1, |msg|{
            assert_eq!(msg.doc._id, "order~clothing~iStore~15000".to_string());
        }); 
        thread::sleep(Duration::from_secs(5));           
        data_provisioner.stop_listening(&channel.0);
    }
*/
}
