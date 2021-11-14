use super::extractor::AuthorExtractor;
use super::*;
use crate::doc::*;
use crate::eventing::broker::{DaaSKafkaBroker, DaaSKafkaProcessor};
use crate::storage::local::LocalStorage;
use crate::storage::DaaSDocStorage;
use std::thread;

pub trait DaaSListenerService {
    fn get_service_health_path() -> String {
        "/health".to_string()
    }
    fn get_service_path() -> String {
        "/{category}/{subcategory}/{source_name}/{source_uid}".to_string()
    }
    fn health(_req: HttpRequest) -> HttpResponse {
        return HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(r#"{"status":"OK"}"#);
    }
    // what about using a generic with the FromRequest trait to pass the Author
    fn index<A: AuthorExtractor>(
        params: Path<Info>,
        author: A,
        duas: DUAs,
        tracker: Tracker,
        body: String,
        req: HttpRequest,
    ) -> HttpResponse;
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
    fn broker_document(mut doc: DaaSDoc, topic: String) -> Result<DaaSDoc, BrokerError> {
        let daas_id = doc._id.clone();
        let my_broker = DaaSKafkaBroker::default();

        debug!(
            "Sending document [{}] to broker using topic [{}]. Waiting for response...",
            daas_id, topic
        );

        let rspns = match my_broker.broker_message(&mut doc, &topic) {
            Ok(_v) => {
                debug!("Broker received Daas document.");
                Ok(doc)
            }
            Err(e) => {
                error!("Error from broker {}", e);
                Err(BrokerError)
            }
        };

        rspns
    }

    fn mark_doc_as_processed(storage: LocalStorage, doc: DaaSDoc) -> Result<DaaSDoc, UpsertError> {
        let daas_id = doc._id.clone();

        // save the modified document
        match storage.mark_doc_as_processed(doc) {
            Ok(d) => {
                debug!("Daas document [{}] has been mark processes.", daas_id);
                Ok(d)
            }
            Err(e) => {
                error!("Could not save the DaaS document [{}] with process_ind=true. Error message: [{}]", daas_id, e);
                Err(UpsertError)
            }
        }
    }

    pub fn process_data(
        mut doc: DaaSDoc,
        broker_topic: Option<String>,
    ) -> Result<DaaSDoc, UpsertError> {
        // validate the document
        doc = match doc.validate() {
            Ok(s) => s,
            Err(_err) => return Err(UpsertError),
        };

        // store a local copy so data isn't lost
        let storage = LocalStorage::new(LocalStorage::get_local_path());
        let doc = match storage.upsert_daas_doc(doc) {
            Ok(d) => {
                info!(
                    "DaaS docoument {} has been successfully upserted.",
                    d.clone()._id
                );
                d
            }
            Err(e) => {
                error!("{}", e);
                return Err(UpsertError);
            }
        };

        // start a detached thread to broker the document
        let doc2broker = doc.clone();
        let topic = match broker_topic {
            Some(t) => t,
            None => DaaSKafkaBroker::make_topic(doc.clone()),
        };
        thread::spawn(move || {
            match DaaSListener::broker_document(doc2broker.clone(), topic) {
                Ok(d) => {
                    // based on cofiguration, should the local document be (1) updated or (2) deleted after processes
                    match DaaSListener::mark_doc_as_processed(storage, d) {
                        Ok(_d2) => {
                            info!(
                                "DaaS docoument {} has been successfully sent to the broker.",
                                doc2broker._id
                            );
                        }
                        Err(e2) => {
                            error!("Could not mark the DaaS document {} as processed. Error message: [{}]", doc2broker._id, e2);
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Could not broker the DaaS document {}. Error message: [{}]",
                        doc2broker._id, e
                    );
                }
            }
        });

        // return
        Ok(doc)
    }
}

impl DaaSListenerService for DaaSListener {
    fn index<A: AuthorExtractor>(
        params: Path<Info>,
        author: A,
        duas: DUAs,
        tracker: Tracker,
        body: String,
        req: HttpRequest,
    ) -> HttpResponse {
        let cat: String = params.category.clone();
        let subcat: String = params.subcategory.clone();
        let srcnme: String = params.source_name.clone();
        let srcuid: usize = params.source_uid;

        let content_type = match req.headers().get("Content-Type") {
            Some(ct) => ct.to_str().unwrap(),
            None => "unknown",
        };

        let usr = author.get_name();
        let mut doc = DaaSDoc::new(
            srcnme,
            srcuid,
            cat,
            subcat,
            usr,
            duas.vec(),
            tracker.clone(),
            body.as_bytes().to_vec(),
        );
        doc.add_meta("content-type".to_string(), content_type.to_string());

        match DaaSListener::process_data(doc, Some("genesis".to_string())) {
            Ok(_d) => HttpResponse::Ok()
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(r#"{"status":"ok"}"#),
            Err(_e) => HttpResponse::UnprocessableEntity()
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(r#"{"error":"unable to process data"}"#),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    /*
        #[test]
        fn test_extract_auth_ok() {
            /*
            let req = test::TestRequest::get().uri("/")
                .header("Authorization", base64::encode(b"myself:password"))
                .to_request();
                */

            //let uri = Uri::from_shared("http://example.com/foo".to_string().as_bytes()).unwrap();
            let uri = "http://example.com/foo".parse::<Uri>().unwrap();
            let mut headers = HeaderMap::new();
            headers.insert(HeaderName::from_lowercase(b"authorization").unwrap(), HeaderValue::from_str(&base64::encode(b"myself:password")).unwrap());
            let req = actix_web::HttpRequest::new(Method::GET, uri, Version::HTTP_2,headers, None);

            assert_eq!(DaaSListener::extract_author(req), "myself");
        }
    */
    #[test]
    fn test_process_data() {
        let _ = env_logger::builder().is_test(true).try_init();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":0,"actor_id":"","previous_hash":"0"},"hash":"33962353871142597622255173163773323410","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let doc = DaaSDoc::from_serialized(&serialized.as_bytes()).unwrap();

        let handle = thread::spawn(move || {
            assert!(DaaSListener::process_data(doc, None).is_ok());
            thread::sleep(Duration::from_secs(10));
        });

        handle.join().unwrap();
    }

    #[test]
    fn test_process_data_tampered_with() {
        let _ = env_logger::builder().is_test(true).try_init();
        let serialized = r#"{"_id":"order~clothing~iStore~15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~15000","index":0,"timestamp":1582766489,"actor_id":"","previous_hash":"0"},"hash":"33962353871142597622255173163773323410","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let doc = DaaSDoc::from_serialized(&serialized.as_bytes()).unwrap();

        let handle = thread::spawn(move || {
            assert!(DaaSListener::process_data(doc, None).is_err());
            thread::sleep(Duration::from_secs(10));
        });

        handle.join().unwrap();
    }
}
