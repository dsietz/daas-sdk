use super::*;
use crate::eventing::broker;
use crate::doc::*;
use crate::storage::DaaSDocStorage;
use crate::storage::local::{LocalStorage};
use std::thread;
// testing
use std::time::Duration;

pub trait DaaSListener {
    fn process_data(doc: DaaSDoc) -> Result<DaaSDoc, DaaSError> {
        // 1. Encrypt the data object (pbd crate feature?)
        // 2. Store the DaaSDoc in local storage
        // 3. Respond to sender 200
        // 4. Send to broker (as separate thread)
        //    4a. if successful, set processed flag = true
        //    4b. if failure, repeat step 4
        //    4c. log activity

        // store a local copy so data isn't lost
        let storage = LocalStorage::new("./tests".to_string());
        let doc = match storage.upsert_daas_doc(doc) {
            Ok(d) => d,
            Err(e) => {
                error!("{}", e);
                return Err(DaaSError)
            },
        };
                
        // start an detached thread to broker the document
        let mut doc2broker = doc.clone();
        thread::spawn(move || {
            let daas_id = doc2broker._id.clone();
            let topic = broker::make_topic(doc2broker.clone());
            //let brokers = vec![broker::KAFKA_BROKERS.to_string()];
            debug!("Sending document [{}] to broker using topic [{}]. Waiting for response...", daas_id, topic);
            let brokers: Vec<String> = broker::KAFKA_BROKERS.split(",").map(|s|{s.to_string()}).collect();
            match broker::produce_message(doc2broker.clone().serialize().as_bytes(), &topic, brokers) {
                Ok(_v) => {
                    debug!("Broker received Daas document.");
                    
                    // mark the document as processed
                    doc2broker.process_ind = true;
                    //let daas_id = doc2broker._id.clone();

                    // save the modified document
                    match storage.upsert_daas_doc(doc2broker) {
                        Ok(_d) => debug!("Daas document [{}] has been mark processes.", daas_id),
                        Err(e) => error!("Could not save the DaaS document [{}] with process_ind=true. Error message: [{}]", daas_id, e),
                    }
                },
                Err(e) => {
                    error!("Error from broker {}",e);
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
        
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":[],"data_obj":{"status":"new"}}"#;
        let doc = DaaSDoc::from_serialized(&serialized);
        
        let handle = thread::spawn(|| {
            println!("Mock service running ...");
            assert!(MyListener::process_data(doc).is_ok());
            thread::sleep(Duration::from_secs(10));
            println!("Mock service stopped.");
        });

        handle.join().unwrap();
    }

    use std::fmt::Write;
    use std::time::Duration;
    use kafka::producer::{Producer, Record, RequiredAcks};

    #[test]
    fn test_kafka_producer(){
        let mut producer =
        Producer::from_hosts(vec!("localhost:9092".to_owned()))
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