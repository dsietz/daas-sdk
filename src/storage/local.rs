use super::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

/// A document storage management solution
pub struct LocalStorage {
    /// The directory path where to storage the DaaS documents
    pub path: String,
}

impl LocalStorage {
    /// Constructor
    /// 
    /// # Arguments
    /// 
    /// * dir_path: String - The location of the directory where to store the Daas documents.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// extern crate daas;
    ///
    /// use daas::storage::local::LocalStorage;
    ///
    /// fn main() {
    ///     let storage = LocalStorage::new("./tmp".to_string());
    /// }
    /// ```
    pub fn new(dir_path: String) -> LocalStorage {
        LocalStorage {
            path: dir_path,
        }
    }

    // Determines if the Daas document file exists
    fn doc_exists(&self, file_uuid: String) -> bool {
        let p = self.get_doc_path(file_uuid).clone();
        let doc = Path::new(&p);
        doc.is_file()
    }

    // Calculates the path where the DaaS document will be located
    fn get_doc_path(&self, doc_id: String) -> String {
        format!("{}/{}",&self.path, doc_id)
    }

    /// Save a new Daas document into persistant storage
    /// 
    /// # Arguments
    /// 
    /// * doc: DaaSDoc - The new DaaS document to save.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use pbd::dua::DUA;
    /// use daas::doc::{DaaSDoc};
    /// use daas::storage::local::LocalStorage;
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let src = "iStore".to_string();
    ///     let uid = 5001;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();     
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
    ///     let storage = LocalStorage::new("./tmp".to_string());
    /// 
    ///     assert!(storage.upsert_daas_doc(doc).is_ok());
    /// }
    /// ```
    pub fn upsert_daas_doc(&self, mut doc: DaaSDoc) -> Result<DaaSDoc, DaaSError>{
        let file_rev = match LocalStorage::next_rev(doc._rev.clone()) {
                    Ok(r) => {
                        r
                    },
                    Err(e) => {
                        return Err(e)
                    },
                };

        let file_uuid = format!("{}_{}", doc._id.clone(), file_rev);
        
        doc._rev = Some(file_rev.clone());
        
        let json_doc = doc.serialize();
        let mut file = match File::create(self.get_doc_path(file_uuid.clone())) {
            Ok(f) => {
                debug!("Created file {}", self.get_doc_path(file_uuid.clone()));
                f
            },
            Err(e) => {
                error!("Could not create DaaS document file {} because of {}.", self.get_doc_path(file_uuid.clone()), e);
                return Err(DaaSError)
            }
        };

        match file.write_all(json_doc.as_bytes()) {
            Ok(_) => {
                info!("Successfully inserted DaaS document {}", self.get_doc_path(file_uuid.clone()));
            },
            Err(_e) => {
                error!("Could not write content to the Daas document {}", self.get_doc_path(file_uuid.clone()))
            },
        }

        Ok(doc)
    }

    // Calculates the next version of the DaaS document
    fn next_rev(revision: Option<String>) -> Result<String, DaaSError> {
        match revision {
            None => Ok("0".to_string()),
            Some(rev) => {
                match rev.parse::<usize>() {
                    Ok(r) => {
                        let new_rev = r + 1;
                        Ok(new_rev.to_string())
                    },
                    Err(_e) => {
                        error!("Could not increment the revision of the DaaS document!");
                        Err(DaaSError)
                    }, 
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pbd::dua::DUA;

    fn get_dua() -> Vec<DUA>{
        let mut v = Vec::new();
        v.push( DUA {
                    agreement_name: "billing".to_string(),
                    location: "www.dua.org/billing.pdf".to_string(),
                    agreed_dtm: 1553988607,
                });
        v
    }

    fn get_daas_doc() -> DaaSDoc {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let data = json!({
            "status": "new"
        });
        let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);

        doc
    }

    #[test]
    fn test_doc_exists() {
        let loc = LocalStorage::new("./tmp".to_string()); 
        assert!(loc.doc_exists("test_doc_exists.test".to_string()));
        assert!(!loc.doc_exists("test_doc_exists.tests".to_string()));
    }

    #[test]
    fn test_new_ok() {
        let loc = LocalStorage::new("./tmp".to_string());
        assert!(true);
    }

    #[test]
    fn test_next_rev() {
        assert_eq!(LocalStorage::next_rev(Some("1".to_string())).unwrap(), "2".to_string());
    }

    #[test]
    fn test_upsert_new() {
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tmp".to_string());
        let doc = get_daas_doc();
        let file_name = format!("{}_{}", doc._id.clone(), 0);

        assert!(loc.upsert_daas_doc(doc).is_ok());
        assert!(Path::new(&format!("{}/{}", loc.path, file_name)).is_file());
    }

    #[test]
    fn test_upsert_version() {
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tmp".to_string());
        let serialized = r#"{"_id":"order~clothing~iStore~6000","_rev":"3","source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":[],"data_obj":{"status":"new"}}"#;
        let doc = DaaSDoc::from_serialized(&serialized);
        let updated_doc = loc.upsert_daas_doc(doc).unwrap();

        assert_eq!(updated_doc._rev.unwrap(), "4".to_string());
    }
}