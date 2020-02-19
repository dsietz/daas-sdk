use super::*;
use crate::eventing::broker::{DaaSKafkaBroker, DaaSKafkaProcessor};
use crate::doc::*;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage, Message};
use std::thread;

pub trait DaaSProcessorService {
    fn start_listening(&mut self, callback: fn(kafka::consumer::Message));
    fn stop_listening(&mut self);
}

pub struct DaaSProcessor {
    pub consumer: Consumer,
    listen_ind: bool,
}

impl DaaSProcessorService for DaaSProcessor{
    fn start_listening(&mut self, callback: fn(kafka::consumer::Message)) {
        self.listen_ind = true;

        while self.listen_ind {
            for messageset in self.consumer.poll().unwrap().iter() {
                for message in messageset.messages() {
                    callback(Message {
                        offset: message.offset,
                        key: message.key,
                        value: message.value,
                    });
                }
                match self.consumer.consume_messageset(messageset) {
                    Ok(_c) => {},
                    Err(err) => panic!("{}",err),
                }
            }
            self.consumer.commit_consumed().unwrap();
        }
    }

    fn stop_listening(&mut self) {
        self.listen_ind = false;
    }
}

impl DaaSProcessor {
    pub fn new(consumer: Consumer) -> DaaSProcessor {
        DaaSProcessor {
            consumer: consumer,
            listen_ind: false,
        }
    }

    // uses the default settings for the `Consumer`
    pub fn default() -> DaaSProcessor {
        DaaSProcessor {
            consumer: Consumer::from_hosts(vec!("localhost:9092".to_string()))
                        .with_topic("genesis".to_string())
                        .with_fallback_offset(FetchOffset::Earliest)
                        .with_group("genesis-consumers".to_string())
                        .with_offset_storage(GroupOffsetStorage::Kafka)
                        .create()
                        .unwrap(),
            listen_ind: false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use std::thread;

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

        assert!(my_broker.broker_message(&mut my_doc, "order.clothing.iStore").is_ok());
        
        let _handle = thread::spawn(move || {
            data_provisioner.start_listening(|msg|{
                let daas_doc = DaaSDoc:: from_serialized(msg.value);
                debug!("Received DaasDoc ID: {}", daas_doc._id);
                assert_eq!(daas_doc._id, "order~clothing~iStore~15000".to_string());
            });            
            data_provisioner.stop_listening();
        });

        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn test_process_data_default() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut data_provisioner = DaaSProcessor::default();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        let my_broker = DaaSKafkaBroker::default();

        assert!(my_broker.broker_message(&mut my_doc, "genesis").is_ok());
        
        let _handle = thread::spawn(move || {
            data_provisioner.start_listening(|msg|{
                let daas_doc = DaaSDoc:: from_serialized(msg.value);
                debug!("Received DaasDoc ID: {}", daas_doc._id);
                assert_eq!(daas_doc._id, "order~clothing~iStore~15000".to_string());
            });            
            data_provisioner.stop_listening();
        });

        thread::sleep(Duration::from_secs(5));
    }
}
