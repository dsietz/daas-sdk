use super::*;
use crate::eventing::broker::{DaaSKafkaBroker, DaaSKafkaProcessor};
use crate::doc::*;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage, Message};

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
}
