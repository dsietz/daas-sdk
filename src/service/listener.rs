use super::*;
use crate::doc::*;
use crate::storage::DaaSDocStorage;
use crate::storage::local::{LocalStorage};
use std::thread;
// testing
use std::time::Duration;

pub trait DaaSListener {
    fn process_data(mut doc: DaaSDoc) -> Result<DaaSDoc, DaaSError> {
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
        
        // start an indepentent thread to broker the document
        thread::spawn(|| {
            debug!("Sent document to broker. Waiting for response...");
            //testing
            thread::sleep(Duration::from_secs(1));
            debug!("Broker responded ...")

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
            thread::sleep(Duration::from_secs(5));
            println!("Mock service stopped.");
        });

        handle.join().unwrap();
    }
}