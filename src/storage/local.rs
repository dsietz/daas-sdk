use super::*;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct LocalStorage {
    pub path: String,
}

impl LocalStorage {
    pub fn new(dir_path: String) -> LocalStorage {
        LocalStorage {
            path: dir_path,
        }
    }

    fn doc_exists(&self, doc_id: String) -> bool {
        let p = self.get_doc_path(doc_id).clone();
        let doc = Path::new(&p);
        doc.is_file()
    }

    fn get_doc_path(&self, doc_id: String) -> String {
        format!("{}/{}",&self.path, doc_id)
    }

    pub fn insert_daas_doc(&self, mut doc: DaaSDoc) -> Result<DaaSDoc, DaaSError>{
        match self.doc_exists(doc._id.clone()) {
            false => {
                doc._rev = match LocalStorage::next_rev("0".to_string()) {
                    Ok(r) => {
                        Some(r)
                    },
                    Err(e) => {
                        return Err(e)
                    },
                };

                let json_doc = doc.serialize();

                let mut file = match File::create(self.get_doc_path(doc._id.clone())) {
                    Ok(f) => {
                        debug!("Created file {}", self.get_doc_path(doc._id.clone()));
                        f
                    },
                    Err(e) => {
                        error!("Could not create DaaS document file {} because of {}.", self.get_doc_path(doc._id.clone()), e);
                        return Err(DaaSError)
                    }
                };

                match file.write_all(json_doc.as_bytes()) {
                    Ok(_) => {
                        info!("Successfully inserted DaaS document {}", self.get_doc_path(doc._id.clone()));
                    },
                    Err(_e) => {
                        error!("Could not write content to the Daas document {}", self.get_doc_path(doc._id.clone()))
                    },
                }

                Ok(doc)
            },
            true => {
                error!("Cannot insert DaaS document {} because document exists. Use upsert function to force an overwrite.", &self.get_doc_path(doc._id));
                Err(DaaSError)
            },
        }
    }

    fn next_rev(rev: String) -> Result<String, DaaSError> {
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
/*
impl DaaSDocStorage for LocalStorage {
    fn upsert_daasdoc(self, mut daas_doc: DaaSDoc) -> Result<DaaSDoc, DaaSError>{
        Ok()
    }
}
*/

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
    fn test_new_ok() {
        let loc = LocalStorage::new("./tmp".to_string());
        assert!(true);
    }

    #[test]
    fn test_upsert() {
        let _ = env_logger::builder().is_test(true).try_init();

        let loc = LocalStorage::new("./tmp".to_string());
        let doc = get_daas_doc();
        let file_name = doc._id.clone();
        loc.insert_daas_doc(doc);

        assert!(Path::new(&format!("{}/{}", loc.path, file_name)).is_file());
    }
}