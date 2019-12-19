use super::*;
use crate::eventing::broker;
use crate::doc::*;
use crate::storage::{DaaSDocStorage};
use crate::storage::local::{LocalStorage};
use std::thread;
// testing
use std::time::Duration;

pub trait DaaSListener {
    fn mark_doc_as_processed( storage: LocalStorage, mut doc: DaaSDoc) -> Result<DaaSDoc, UpsertError>{
        let daas_id = doc._id.clone();

        // mark the document as processed
        doc.process_ind = true;

        // save the modified document
        match storage.upsert_daas_doc(doc) {
            Ok(d) => {
                debug!("Daas document [{}] has been mark processes.", daas_id);
                Ok(d)
            },
            Err(e) => {
                error!("Could not save the DaaS document [{}] with process_ind=true. Error message: [{}]", daas_id, e);
                Err(UpsertError)
            },
        }
    }

    fn broker_document(mut doc: DaaSDoc) -> Result<DaaSDoc, BrokerError>{
        let daas_id = doc._id.clone();
        let topic = broker::make_topic(doc.clone());
        let brokers: Vec<String> = broker::KAFKA_BROKERS.split(",").map(|s|{s.to_string()}).collect();
        
        debug!("Sending document [{}] to broker using topic [{}]. Waiting for response...", daas_id, topic);
        
        let rspns = match broker::produce_message(doc.clone().serialize().as_bytes(), &topic, brokers) {
            Ok(_v) => {
                debug!("Broker received Daas document.");
                Ok(doc)
            },
            Err(e) => {
                error!("Error from broker {}",e);
                Err(BrokerError)
            },
        };

        rspns
    }

    fn process_data(&self, doc: DaaSDoc) -> Result<DaaSDoc, UpsertError> {
        // store a local copy so data isn't lost
        let storage = LocalStorage::new("./tests".to_string());
        let doc = match storage.upsert_daas_doc(doc) {
            Ok(d) => d,
            Err(e) => {
                error!("{}", e);
                return Err(UpsertError)
            },
        };
                
        // start an detached thread to broker the document
        let mut doc2broker = doc.clone();
        thread::spawn(move || {
            match broker_document(doc2broker) {
                Ok(d) => {
                    match mark_doc_as_processed(storage, d) {
                        Ok(d2) => {

                        },
                        Err(e2) => {
                            error!("Could not mark the DaaS document {} as processed. Error message: [{}]", doc2broker._id, e2);
                        },
                    }
                },
                Err(e) => {
                    error!("Could not broker the DaaS document {}. Error message: [{}]", doc2broker._id, e);
                },
            }
        });

        // return with a Ok(doc)
        debug!("Sent back a status 200");
        Ok(doc)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_process_data() {
        let _ = env_logger::builder().is_test(true).try_init();

        struct MyListener {};
        impl DaaSListener for MyListener{};
        let listener = MyListener{};
        
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":[],"data_obj":{"status":"new"}}"#;
        let doc = DaaSDoc::from_serialized(&serialized);
        
        let handle = thread::spawn(|| {
            println!("Mock service running ...");
            assert!(listener.process_data(doc).is_ok());
            thread::sleep(Duration::from_secs(10));
            println!("Mock service stopped.");
        });

        handle.join().unwrap();
    }

    use std::fmt::Write;
    use std::time::Duration;
    use kafka::producer::{Producer, Record, RequiredAcks};

    #[ignore]
    #[test]
    fn test_kafka_producer(){
        let mut producer =
        Producer::from_hosts(vec!("18.212.66.92:9092".to_owned()))
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();

        let mut buf = String::with_capacity(2);
        for i in 0..10 {
          let _ = write!(&mut buf, "{}", i); // some computation of the message data to be sent
          producer.send(&Record::from_value("test", buf.as_bytes())).unwrap();
          buf.clear();
        }
    }
}