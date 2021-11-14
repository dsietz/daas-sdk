use super::*;
use crate::doc::DaaSDoc;
use kafka::client::KafkaClient;
use kafka::error::{ErrorKind, KafkaCode};
use kafka::producer::{Producer, Record, RequiredAcks};
use std::thread;
use std::time::Duration;

pub trait DaaSKafkaProcessor {
    fn make_topic(doc: DaaSDoc) -> String {
        format!("{}.{}.{}", doc.category, doc.subcategory, doc.source_name)
    }
    fn broker_message_with_client<'a, 'b>(
        client: KafkaClient,
        doc: &'a mut DaaSDoc,
        topic: &'b str,
    ) -> Result<(), kafka::error::ErrorKind>;
    fn broker_message<'a, 'b>(
        &self,
        doc: &'a mut DaaSDoc,
        topic: &'b str,
    ) -> Result<(), kafka::error::ErrorKind>;
}

pub struct DaaSKafkaBroker {
    pub brokers: Vec<String>,
}

impl DaaSKafkaProcessor for DaaSKafkaBroker {
    fn broker_message_with_client<'a, 'b>(
        mut client: KafkaClient,
        doc: &'a mut DaaSDoc,
        topic: &'b str,
    ) -> Result<(), kafka::error::ErrorKind> {
        let mut attempt = 0;

        loop {
            attempt += 1;
            let _ = client.load_metadata(&[topic])?;
            if client
                .topics()
                .partitions(topic)
                .map(|p| p.len())
                .unwrap_or(0)
                > 0
            {
                break;
            } else if attempt > 2 {
                // try up to 3 times
                // return some error
                return Err(ErrorKind::Kafka(KafkaCode::UnknownTopicOrPartition));
            }
            debug!("Attempt #{} to connect to the Kafka broker...", attempt);
            thread::sleep(Duration::from_secs(1));
        }

        let mut producer = Producer::from_client(client)
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()?;

        producer.send(&Record {
            topic: topic,
            partition: -1,
            key: doc._id.clone(),
            value: doc.serialize().as_bytes(),
        })?;

        Ok(())
    }

    fn broker_message<'a, 'b>(
        &self,
        doc: &'a mut DaaSDoc,
        topic: &'b str,
    ) -> Result<(), kafka::error::ErrorKind> {
        let client = KafkaClient::new(self.brokers.clone());

        DaaSKafkaBroker::broker_message_with_client(client, doc, topic)
    }
}

impl DaaSKafkaBroker {
    pub fn new(brokers: Vec<String>) -> DaaSKafkaBroker {
        DaaSKafkaBroker { brokers: brokers }
    }

    pub fn default() -> DaaSKafkaBroker {
        DaaSKafkaBroker {
            brokers: vec!["localhost:9092".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pbd::dtc::Tracker;
    use pbd::dua::DUA;

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

    fn get_daas_doc() -> DaaSDoc {
        let src = "iStore".to_string();
        let uid = 6000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let dtc = get_dtc(src.clone(), uid.clone(), cat.clone(), sub.clone());
        let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();

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

    #[test]
    fn test_make_topic() {
        assert_eq!(
            DaaSKafkaBroker::make_topic(get_daas_doc()),
            "order.clothing.iStore".to_string()
        );
    }

    #[test]
    fn test_send_message() {
        let my_broker = DaaSKafkaBroker::default();
        let mut my_doc = get_daas_doc();

        match my_broker.broker_message(&mut my_doc, "testTopic") {
            Ok(_v) => {
                assert!(true);
            }
            Err(e) => {
                println!("Failed to send message to {:?}: {:?}", my_broker.brokers, e);
                assert!(false);
            }
        }
    }
}
