use super::*;
use crate::eventing::broker;
use crate::doc::*;
use crate::storage::{DaaSDocStorage};
use crate::storage::local::{LocalStorage};
use std::thread;

pub trait DaaSListenerService {
    fn index(params: Path<Info>, duas: DUAs, body: String, req: HttpRequest) -> HttpResponse;
    fn get_service_health_path() -> String {
        "/health".to_string()
    }
    fn get_service_path() -> String {
        "/{category}/{subcategory}/{source_name}/{source_uid}".to_string()
    }
    fn health(_req: HttpRequest) -> HttpResponse   {
        return HttpResponse::Ok()
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(r#"{"status":"OK"}"#)
    }
}

#[derive(Deserialize)]
pub struct Info {
    category: String,
    subcategory: String,
    source_name: String,
    source_uid: usize,
}

pub struct DaaSListener {}

impl DaaSListener {
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

    fn mark_doc_as_processed(storage: LocalStorage, mut doc: DaaSDoc) -> Result<DaaSDoc, UpsertError>{
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

    pub fn process_data(doc: DaaSDoc) -> Result<DaaSDoc, UpsertError> {
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
            match DaaSListener::broker_document(doc2broker.clone()) {
                Ok(d) => {
                    // based on cofiguration, should the local document be (1) updated or (2) deleted after processes
                    match DaaSListener::mark_doc_as_processed(storage, d) {
                        Ok(d2) => {
                            info!("DaaS coument {} has been successfully sent to the broker.", doc2broker._id);
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

        // return
        Ok(doc)
    }
}

impl DaaSListenerService for DaaSListener {
    fn index(params: Path<Info>, duas: DUAs, body: String, req: HttpRequest) -> HttpResponse {
        let cat: String = params.category.clone();
        let subcat: String = params.subcategory.clone();
        let srcnme: String = params.source_name.clone();
        let srcuid: usize = params.source_uid;

        // verify body is json
        let data = match serde_json::from_str(&body) {
            Ok(d) => d,
            _ => {
                return HttpResponse::BadRequest()
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(r#"{"error":"Bad Json"}"#) 
            },
        };

        let usr = "myself".to_string();
        let doc = DaaSDoc::new(srcnme, srcuid, cat, subcat, usr, duas.vec(), data);
        
        match DaaSListener::process_data(doc) {
            Ok(_d) => {
                HttpResponse::Ok()
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(r#"{"status":"ok"}"#) 
            },
            Err(_e) => {
                HttpResponse::UnprocessableEntity()
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(r#"{"error":"unable to process data"}"#)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_process_data() {
        let _ = env_logger::builder().is_test(true).try_init();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":[],"data_obj":{"status":"new"}}"#;
        let doc = DaaSDoc::from_serialized(&serialized);
        
        let handle = thread::spawn(move || {
            println!("Mock service running ...");
            assert!(DaaSListener::process_data(doc).is_ok());
            thread::sleep(Duration::from_secs(10));
            println!("Mock service stopped.");
        });

        handle.join().unwrap();
    }
}