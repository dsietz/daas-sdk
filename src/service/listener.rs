use super::*;
use crate::doc::*;
use crate::storage::DaaSDocStorage;
use crate::storage::local::{LocalStorage};

pub trait DaaSListener {
    fn process_data(mut doc: DaaSDoc) -> Result<DaaSDoc, DaaSError> {
        let storage = LocalStorage::new("./test".to_string());
        match storage.upsert_daas_doc(doc) {
            Ok(d) => Ok(d),
            Err(e) => {
                error!("{}", e);
                Err(DaaSError)
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[test]
    fn test_process_data() {
        let _ = env_logger::builder().is_test(true).try_init();

        struct MyListener {};
        impl DaaSListener for MyListener{};
        
        let serialized = r#"{"_id":"order|clothing|iStore|15000","_rev":null,"source_name":"iStore","source_uid":15000,"category":"order","subcategory":"clothing","author":"iStore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":[],"data_obj":{"status":"new"}}"#;
        let doc = DaaSDoc::from_serialized(&serialized);
        
        assert!(MyListener::process_data(doc).is_ok());
    }
}