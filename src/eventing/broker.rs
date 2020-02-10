use super::*;
use crate::doc::DaaSDoc;
use std::{thread};
use std::time::Duration;
use kafka::client::KafkaClient;
use kafka::producer::{Producer, Record, RequiredAcks};
use kafka::error::{ErrorKind, KafkaCode};

pub static KAFKA_BROKERS: &str = "localhost:9092";

pub fn make_topic(doc: DaaSDoc) -> String {
    format!("{}.{}.{}", doc.category, doc.subcategory, doc.source_name)
}

pub fn produce_message<'a, 'b>(data: &'a [u8], topic: &'b str, brokers: Vec<String>) -> Result<(), kafka::error::ErrorKind> {
    let mut client = KafkaClient::new(brokers);

    let mut attempt = 0;
    loop {
        attempt += 1;
        let _ = client.load_metadata(&[topic])?;
        if client.topics().partitions(topic).map(|p| p.len()).unwrap_or(0) > 0 { // <-- HERE
            break;
        } else if attempt > 2 { // try up to 3 times
            // return some error
            return Err(ErrorKind::Kafka(KafkaCode::UnknownTopicOrPartition));
        }
        thread::sleep(Duration::from_secs(1));
    }

    let mut producer =
        Producer::from_client(client)
             .with_ack_timeout(Duration::from_secs(1))
             .with_required_acks(RequiredAcks::One)
             .create()?;

    producer.send(&Record{
        topic: topic,
        partition: -1,
        key: (),
        value: data,
    })?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    
    use pbd::dua::DUA;
    use pbd::dtc::Tracker;

    fn get_dua() -> Vec<DUA>{
        let mut v = Vec::new();
        v.push( DUA {
                    agreement_name: "billing".to_string(),
                    location: "www.dua.org/billing.pdf".to_string(),
                    agreed_dtm: 1553988607,
                });
        v
    }

    fn get_dtc(src_name: String, src_uid: usize, cat: String, subcat: String) -> Tracker {
        Tracker::new(DaaSDoc::make_id(cat.clone(), subcat.clone(), src_name.clone(), src_uid))
    }

    fn get_daas_doc() -> DaaSDoc {
        let src = "iStore".to_string();
        let uid = 6000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let dtc = get_dtc(src.clone(),uid.clone(),cat.clone(),sub.clone());
        let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
        
        let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, dtc, data);

        doc
    }

    #[test]
    fn test_make_topic(){
        assert_eq!(broker::make_topic(get_daas_doc()), "order.clothing.iStore".to_string());
    }

    #[ignore]
    #[test]
    fn test_send_message() {
        match produce_message("Hello Kafka...".as_bytes(), "testTopic", vec!(KAFKA_BROKERS.to_string())) {
                Ok(_v) => {
                    assert!(true);
                },
                Err(e) => {
                    println!("Failed to send message to {}: {:?}", KAFKA_BROKERS.to_string(), e);
                    assert!(false);
                }
        }
    }
}