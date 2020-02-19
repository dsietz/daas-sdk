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
    pub fn new(host: String,  topic: String, group_id: String) -> DaaSProcessor {
        DaaSProcessor {
            consumer: Consumer::from_hosts(vec!(host.to_owned()))
                        .with_topic(topic.to_owned())
                        .with_fallback_offset(FetchOffset::Earliest)
                        .with_group(group_id.to_owned())
                        .with_offset_storage(GroupOffsetStorage::Kafka)
                        .create()
                        .unwrap(),
            listen_ind: false,
        }
    }

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
    fn test_process_data() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut data_provisioner = DaaSProcessor::default();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":""},"hash":"103351245680471505841311888122193174123","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let mut my_doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        let my_broker = DaaSKafkaBroker::default();

        my_broker.broker_message(&mut my_doc, "genesis");
        
        let handle = thread::spawn(move || {
            println!("Mock processor service running ...");
            data_provisioner.start_listening(|msg|{
                let daas_doc = DaaSDoc:: from_serialized(msg.value);
                debug!("Received DaasDoc ID: {}", daas_doc._id);
                assert_eq!(daas_doc._id, "order~clothing~iStore~15000".to_string());
            });            
            data_provisioner.stop_listening();
            println!("Mock processor service stopped.");
        });

        thread::sleep(Duration::from_secs(5));
    }
}
