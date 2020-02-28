use super::*;
use std::fs;
use std::fs::{File};
use std::io::prelude::*;
use std::path::Path;

/// A document storage management solution
pub struct LocalStorage {
    /// The directory path where to storage the DaaS documents (default: "./")
    pub path: String,
}

impl Default for LocalStorage {
    // provide a LocalStorage object with the default values
    fn default() -> Self {
        LocalStorage{
            path: ".".to_string(),
        }
    }
}

impl DaaSDocStorage for LocalStorage {
    /// Save a Daas document into persistant storage based upon the revision of the document
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
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    /// use daas::storage::DaaSDocStorage;
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
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     let storage = LocalStorage::new("./tmp".to_string());
    /// 
    ///     assert!(storage.upsert_daas_doc(doc).is_ok());
    /// }
    /// ```
    fn upsert_daas_doc(&self, mut doc: DaaSDoc) -> Result<DaaSDoc, UpsertError>{
        // make sure the DaaS document provided is the latest revision
        let latest_rev = self.latest_rev(doc._id.clone());

        match doc._rev.clone() {
            Some(r) => {
                if latest_rev != r {
                    warn!("The DaaSDoc doesn't have the latest revision!");
                    return Err(UpsertError)
                }
            },
            None => {},
        }

        // get the latest revision number and increment it
        let file_rev = match LocalStorage::next_rev(Some(latest_rev)) {
                    Ok(r) => {
                        r
                    },
                    Err(_e) => {
                        warn!("Couldn't get the next revision for the DaaSDoc!");
                        return Err(UpsertError)
                    },
                };

        // Calculate the file name for the DaaS document  
        let file_uuid = LocalStorage::make_doc_uuid(doc._id.clone(), file_rev.clone());
        
        //create the full directory path if doesn't exists
        let doc_dir_path = self.get_dir_path(file_uuid.clone());
        match LocalStorage::ensure_dir_path(doc_dir_path.clone()) {
            Err(_e) => {
                error!("Could not create dynamic directory path {} to store DaaS document {}", doc_dir_path.clone(), file_uuid.clone());
                return Err(UpsertError)
            },
            Ok(_)  => {
                debug!("Created dynamic directory path {} ...", doc_dir_path.clone());
            },
        }

        // update the revision number of the DaaS document
        doc._rev = Some(file_rev.clone());
        
        // Try to create the file
        let json_doc = doc.serialize();
        let mut file = match File::create(self.get_doc_path(file_uuid.clone())) {
            Ok(f) => {
                debug!("Created file {}", self.get_doc_path(file_uuid.clone()));
                f
            },
            Err(e) => {
                error!("Could not create DaaS document file {} because of {}.", self.get_doc_path(file_uuid.clone()), e);
                return Err(UpsertError)
            }
        };

        // write the DaaS document content to the file 
        match file.write_all(json_doc.as_bytes()) {
            Ok(_) => {
                info!("Successfully inserted DaaS document {}", self.get_doc_path(file_uuid.clone()));
            },
            Err(_e) => {
                error!("Could not write content to the Daas document {}", self.get_doc_path(file_uuid.clone()))
            },
        }

        // return a Ok Result with the new/updated DaaS document
        Ok(doc)
    }

    /// Retrieves a saved Daas document from storage
    /// 
    /// # Arguments
    /// 
    /// * doc_id: String - The _id of the DaaS document to retrieved.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate daas;
    ///
    /// use daas::doc::{DaaSDoc};
    /// use daas::storage::DaaSDocStorage;
    /// use daas::storage::local::LocalStorage;
    ///
    /// fn main() {
    ///     let storage = LocalStorage::new("./tests".to_string());
    ///     let daas_doc = storage.get_doc_by_id("order~clothing~iStore~5000".to_string(), None).unwrap();
    /// 
    ///     assert_eq!(daas_doc._rev.unwrap(), "3".to_string());
    /// }
    /// ```
    fn get_doc_by_id(&self, doc_id: String, doc_rev: Option<String>) -> Result<DaaSDoc, RetrieveError> {
        let path = match doc_rev {
            Some(r) => LocalStorage::make_doc_uuid(self.get_doc_path(doc_id), r),
            None =>    LocalStorage::make_doc_uuid(self.get_doc_path(doc_id.clone()), self.latest_rev(doc_id)),
        };
        
        info!("Retrieving DaaS document {} ...", path.clone());

        let serialized: String = match fs::read_to_string(path.clone()) {
                Ok(c) => {
                    c
                },
                Err(e) => {
                    error!("Could not read the DaaS document {} from storage. {}", path, e);
                    return Err(RetrieveError)
                },
            };
        
        match DaaSDoc::from_serialized(&serialized.as_bytes()) {
            Ok(doc) => Ok(doc),
            Err(err) => {
                error!("{}", err);
                return Err(RetrieveError)
            },
        }
    }
}

impl LocalStorage {
    /// Delimiter used for building the unique identifier value for the DaaS document
    //pub const DELIMITER: &'static str = "~";

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
        match LocalStorage::ensure_dir_path(dir_path.clone()){
            Err(_e) => {
                warn!("Could not create directory path {} for local storage of the DaaS documents.", dir_path);
                warn!("Using default settings ...");
                LocalStorage::default()
            },
            _ => {
                LocalStorage {
                    path: dir_path,
                }
            },
        }
    }

    // Ensures that the directory path where the DaaS documents exists - if not create the entire path
    fn ensure_dir_path(dir_path: String) -> std::io::Result<()> {
        fs::create_dir_all(dir_path)
    }

    // Determines if the Daas document file exists
    #[allow(dead_code)]
    fn doc_exists(&self, file_uuid: String) -> bool {
        println!("Searching for DaaS document {} ...", self.get_doc_path(file_uuid.clone()));
        let p = self.get_doc_path(file_uuid.clone());
        let doc = Path::new(&p);
        doc.is_file()
    }

    // Calculates the full path where the DaaS document will be located
    fn get_doc_path(&self, doc_uuid: String) -> String {
        let dir: Vec<&str> = doc_uuid.split(DELIMITER).collect();
        format!("{}/{}/{}/{}/{}/{}",&self.path, dir[0], dir[1], dir[2], dir[3], doc_uuid)
    }

    // Calculates the base path where the DaaS document will be located
    fn get_dir_path(&self, doc_uuid: String) -> String {
        let dir: Vec<&str> = doc_uuid.split(DELIMITER).collect();
        format!("{}/{}/{}/{}/{}",&self.path, dir[0], dir[1], dir[2], dir[3])
    }

    fn make_doc_uuid(doc_id: String, rev: String) -> String {
        format!("{}{}{}", doc_id, DELIMITER, rev)
    }

    pub fn mark_doc_as_processed(&self, doc: DaaSDoc) -> Result<DaaSDoc, UpsertError>{
        let mut doc = match self.get_doc_by_id(doc._id, doc._rev) {
            Ok(d) => d,
            Err(e) => {
                error!("Error: cannot mark DaaS document as processed. {}", e);
                return Err(UpsertError)
            }
        };

        doc.process_ind = true;

        // overwrite the file

        // Calculate the file name for the DaaS document  
        let file_uuid = LocalStorage::make_doc_uuid(doc._id.clone(), doc._rev.clone().unwrap());

        // Try to create the file
        let json_doc = doc.serialize();
        let mut file = match File::create(self.get_doc_path(file_uuid.clone())) {
            Ok(f) => {
                debug!("Created file {}", self.get_doc_path(file_uuid.clone()));
                f
            },
            Err(e) => {
                error!("Could not create DaaS document file {} because of {}.", self.get_doc_path(file_uuid.clone()), e);
                return Err(UpsertError)
            }
        };

        // write the DaaS document content to the file 
        match file.write_all(json_doc.as_bytes()) {
            Ok(_) => {
                info!("Successfully inserted DaaS document {}", self.get_doc_path(file_uuid.clone()));
                Ok(doc)
            },
            Err(_e) => {
                error!("Could not write content to the Daas document {}", self.get_doc_path(file_uuid.clone()));
                return Err(UpsertError)
            },
        }
    }

    // Calculates the next version of the DaaS document
    fn next_rev(revision: Option<String>) -> Result<String, DaaSDocError> {
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
                        Err(DaaSDocError)
                    }, 
                }
            }
        }
    }

    // find the latest revision for the DaaS document based on the doc._id
    fn latest_rev(&self, doc_id: String) -> String {
        //otherwise find latest revision
        let dir_path = self.get_dir_path(doc_id.clone());
        let base_dir = Path::new(&dir_path);

        match base_dir.is_dir() {
            true => {
                debug!("Searching in {} for latest version for {} ...", dir_path.clone(), doc_id);
                let mut latest_rev = "0".to_string(); 
                let mut paths: Vec<_> = fs::read_dir(dir_path).unwrap().filter_map(|r| r.ok()).collect();

                paths.sort_by_key(|dir| dir.path());
                latest_rev = format!("{}", paths.pop().unwrap().file_name().into_string().unwrap().split(DELIMITER).collect::<Vec<&str>>().last().unwrap());
                
                latest_rev
            },
            false => {
                // set to zero for not existing document
                "0".to_string()
            },
        }
    }
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
    fn test_make_doc_uuid() {
        let doc_id = "order~clothing~iStore~5000".to_string();
        let rev = "0".to_string();
        let expected = format!("{}~{}",doc_id.clone(),rev.clone());
        
        assert_eq!(LocalStorage::make_doc_uuid(doc_id,rev), expected); 
    }

    #[test]
    fn test_doc_exists() {
        let loc = LocalStorage::new("./tests".to_string()); 
        assert!(loc.doc_exists("test~doc~exists~0001~0.test".to_string()));
        assert!(!loc.doc_exists("test~doc~exists~0001~0.tests".to_string()));
    }

    #[test]
    fn test_default() {
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::default(); 
        assert_eq!(loc.path, ".".to_string());
    }

    #[test]
    fn test_get_doc_by_id_latest(){
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tests".to_string());
        
        match loc.get_doc_by_id("order~clothing~iStore~5000".to_string(), None) {
            Ok(doc) => {
                match doc._rev {
                    Some(r) => assert_eq!(r, "3".to_string()),
                    None => assert!(false),
                }
            },
            Err(err) => panic!("Could not find the latest document. Error:{}", err),
        }
    }

    #[test]
    fn test_get_doc_by_id_rev_found(){
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tests".to_string());
        
        let rslt = match loc.get_doc_by_id("order~clothing~iStore~5000".to_string(), Some("2".to_string())) {
            Err(_e) => false,
            _ => true,
        };

        assert!(rslt);
    }

    #[test]
    fn test_get_doc_by_id_rev_not_found(){
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tests".to_string());
        
        let rslt = match loc.get_doc_by_id("order~clothing~iStore~5000".to_string(), Some("15".to_string())) {
            Err(_e) => true,
            _ => false,
        };

        assert!(rslt);
    }

    #[test]
    fn test_get_doc_path() {
        let loc = LocalStorage::new("./tmp".to_string());
        assert_eq!(loc.get_doc_path("order~clothing~iStore~5000~0".to_string()), "./tmp/order/clothing/iStore/5000/order~clothing~iStore~5000~0".to_string())
    }

    #[test]
    fn test_get_dir_path() {
        let loc = LocalStorage::new("./tmp".to_string());
        assert_eq!(loc.get_dir_path("order~clothing~iStore~5000~0".to_string()), "./tmp/order/clothing/iStore/5000".to_string())
    }

    #[test]
    fn test_ensure_dir_path() {
        assert!(LocalStorage::ensure_dir_path("./tmp".to_string()).is_ok());
    }

    #[test]
    fn test_new_ok() {
        let _loc = LocalStorage::new("./tmp".to_string());
        assert!(true);
    }

    #[test]
    fn test_next_rev() {
        assert_eq!(LocalStorage::next_rev(Some("1".to_string())).unwrap(), "2".to_string());
    }

    #[test]
    fn test_upsert_new() {
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tests".to_string());
        let doc = get_daas_doc();
        let file_name = LocalStorage::make_doc_uuid(doc._id.clone(), 0.to_string());

        assert!(loc.upsert_daas_doc(doc).is_ok());
        assert!(Path::new(&format!("{}/order/clothing/iStore/6000/{}", loc.path, file_name)).is_file());
    }

    #[test]
    fn test_upsert_binary_new() {
        // prepare the DaaS data
        let src = "iStore".to_string();
        let uid = 16500;
        let cat = "order".to_string();
        let sub = "music".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let dtc = get_dtc(src.clone(),uid.clone(),cat.clone(),sub.clone());

        let mut file1 = match File::open("./tests/example_audio_clip.mp3") {
            Ok(aud) => aud,
            Err(err) => {
                panic!("Cannot read the audio file: {}",err);
            },
        };

        let mut data = Vec::new();
        file1.read_to_end(&mut data).unwrap();

        // store the DaaSDoc
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tests".to_string());
        let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, dtc, data); 
        let file_name = LocalStorage::make_doc_uuid(doc._id.clone(), 0.to_string());

        assert!(loc.upsert_daas_doc(doc).is_ok());
        assert!(Path::new(&format!("{}/order/music/iStore/16500/{}", loc.path, file_name)).is_file());        
        
        // reteive the DaaSDoc from storage
        let mut f = File::open(format!("{}/order/music/iStore/16500/{}", loc.path, file_name)).unwrap();
        let mut content = Vec::new();
        f.read_to_end(&mut content).unwrap();
        let doc = DaaSDoc::from_serialized(&content).unwrap();

        // create an audio file from the DaaSDoc data object
        let mut file2 = File::create(Path::new(&format!("{}/order/music/iStore/16500/example_audio_clip.mp3", loc.path))).unwrap();
        match file2.write_all(&doc.data_obj) {
            Ok(_aud) => assert!(true),
            Err(_e) => assert!(false),
        }

        // make sure both files are the same
        assert_eq!(fs::metadata("./tests/example_audio_clip.mp3").unwrap().len(), fs::metadata(format!("{}/order/music/iStore/16500/example_audio_clip.mp3", loc.path)).unwrap().len());
    }

    #[test]
    fn test_upsert_bad_revision() {
        let _ = env_logger::builder().is_test(true).try_init();
        let loc = LocalStorage::new("./tmp".to_string());
        let serialized = r#"
        {"_id":"order~clothing~iStore~6000","_rev":"4","source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"last_updated":1553988607,
        "data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],
        "data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~6000","index":0,"timestamp":0,"actor_id":"","previous_hash":"0"},"hash":"266723159776784443356201446382797864672","nonce":5}]},
        "meta_data":{},"tags":[],
        "data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let doc = DaaSDoc::from_serialized(&serialized.as_bytes()).unwrap();

        assert!(loc.upsert_daas_doc(doc).is_err());
    }
}