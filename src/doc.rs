//! The `daas` module provides functionality and Object Oriented implementation of a Data as a Service (DaaS) object.
//!
//! # Examples
//!
//!
//! Create DaaS object that represents a new purchase of clothing from an online store.
//!
//! ```
//! #[macro_use] 
//! extern crate serde_json;
//! extern crate pbd;
//! extern crate daas;
//!
//! use serde_json::value::*;
//! use pbd::dua::DUA;
//! use pbd::dtc::Tracker;
//! use daas::doc::{DaaSDoc};
//!
//! fn main() {
//!		let src = "iStore".to_string();
//!		let uid = 5000;
//!		let cat = "order".to_string();
//!		let sub = "clothing".to_string();
//!		let auth = "istore_app".to_string();
//!		let mut dua = Vec::new();
//!		dua.push(DUA {
//!		    agreement_name: "billing".to_string(),
//!		    location: "https://dua.org/agreements/v1/billing.pdf".to_string(),
//!		    agreed_dtm: 1553988607,
//!		});
//!     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
//!		let data = String::from(r#"{
//!         "product": "leather coat",
//!         "quantity": 1,
//!		    "status": "new"
//!		}"#).as_bytes().to_vec();
//! 
//!		let doc = DaaSDoc::new(src, uid, cat, sub, auth, dua,tracker, data);
//! 
//!     assert_eq!(doc.source_uid, uid);
//! }
//! ```

use crate::*;
use std::collections::BTreeMap;
use pbd::dua::DUA;
use pbd::dtc::Tracker;
use serde_json::Value;

// Repesentation of a map for storing metadata about the data object
type Metadata = BTreeMap<String, String>;

/// Represents an existing DaaS document (after it has been saved and assigned a _rev value)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DaaSDoc {
    /// The unique identifier
    pub _id: String,
    /// The revision number
    pub _rev: Option<String>,
    /// The name of the data source
    pub source_name: String,
    /// The unique identifier that the data source provides
    pub source_uid: usize,
    /// The name of the category (e.g.: order)
    pub category: String,
    /// The name of the subcategory (e.g.: clothing)
    pub subcategory: String,
    /// The name of the author who created the document
    pub author: String,
    /// The indicator that represents if the document is waiting to be processed (processed = true, needs to be processed = false)
    pub process_ind: bool,
    /// The Unix Epoch time when the document was last updated, (e.g.: 1555972752)
    pub last_updated: u64,
    /// The list of Data Usage Agreements for the data represented in the DaaS Document
    pub data_usage_agreements: Vec<DUA>,
    /// The Data Tracker Chain that represents the lineage of the DaaS Document
    pub data_tracker: Tracker,
    // The list of metadata about the data object (key, value)
    pub meta_data: Metadata,
    // List of tags to provide context about the data object
    pub tags: Vec<String>,
    /// The byte slice that represents the data from the data source managed by the DaaS document
    pub data_obj: Vec<u8>,
}

/// Represents an new DaaS document (before it has been saved and assigned a _rev value)
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DaaSDocNoRev{
    /// The unique identifier
    pub _id: String,
    /// The name of the data source
    pub source_name: String,
    /// The unique identifier that the data source provides
    pub source_uid: usize,
    /// The name of the category (e.g.: order)
    pub category: String,
    /// The name of the subcategory (e.g.: clothing)
    pub subcategory: String,
    /// The name of the author who created the document
    pub author: String,
    /// The indicator that represents if the document is waiting to be processed (processed = true, needs to be processed = false)
    pub process_ind: bool,
    /// The Unix Epoch time when the document was last updated, (e.g.: 1555972752)
    pub last_updated: u64,
    /// The list of Data Usage Agreements for the data represented in the DaaS Document
    pub data_usage_agreements: Vec<DUA>,
    /// The Data Tracker Chain that represents the lineage of the DaaS Document
    pub data_tracker: Tracker,
    /// The byte slice that represents the data from the data source managed by the DaaS document
    pub data_obj: Vec<u8>,
}

impl DaaSDoc {
    /// Delimiter used for building the unique identifier value for the DaaS document
    //pub const DELIMITER: &'static str = "~";

    /// Constructor
    /// 
    /// # Arguments
    /// 
    /// * src_name: String - The name of the data source.</br>
    /// * src_uid: usize - The unique identifier that the data source provided.</br>
    /// * cat: String - The name of the category (e.g.: order).</br>
    /// * sub: String - The name of the subcategory (e.g.: clothing).</br>
    /// * auth: String - The name of the auithor who created the document.</br>
    /// * data: Value - The json value that represents the data from the data source managed by the DaaS document.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();     
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     
    ///     println!("{:?}", doc._id);
    /// }
    /// ```
    pub fn new(src_name: String, src_uid: usize, cat: String, subcat: String, auth: String, duas: Vec<DUA>, dtc: Tracker, data: Vec<u8>) -> DaaSDoc {
        let this_id = DaaSDoc::make_id(cat.clone(), subcat.clone(), src_name.clone(), src_uid);

        DaaSDoc {
            _id: this_id.clone(),
            _rev: None,
            source_name: src_name,
            source_uid: src_uid,
            category: cat,
            subcategory: subcat,
            author: auth,
            process_ind: false,
            last_updated: get_unix_now!(),
            data_usage_agreements: duas,
            data_tracker: dtc,
            meta_data: Metadata::new(),
            tags: Vec::new(),
            data_obj: data,
        }
    }

    /// Adds an entry to the metadata
    /// 
    /// # Arguments
    /// 
    /// * key: String - The key used to identify the name of the metadata property.</br>
    /// * value: String - The value used to define the metadata property.</br>
    ///    
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     doc.add_meta("foo".to_string(),"bar".to_string());
    ///   
    ///     assert_eq!(doc.get_meta("foo".to_string()), "bar");
    /// }
    /// ```
    pub fn add_meta(&mut self, key: String, value: String) {
        &self.meta_data.insert(key, value);
    }

    /// Adds a tag
    /// 
    /// # Arguments
    /// 
    /// * tag: String - The textual label to use as a tag.</br>
    ///    
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua,tracker,  data);
    ///     doc.add_tag("foo".to_string());
    ///     doc.add_tag("bar".to_string());
    ///   
    ///     println!("There are {} tags for this DaaSDoc", doc.get_tags().len());
    /// }
    /// ```
    pub fn add_tag(&mut self, tag: String) {
        &self.tags.push(tag);
    }

    /// Returns the data source json value as a reference
    /// 
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     
    ///     let dat: Value = serde_json::from_str(&String::from_utf8(doc.data_obj).unwrap()).unwrap();
    ///     assert_eq!(dat.get("status").unwrap(), "new");
    /// }
    /// ```
    pub fn data_obj_as_ref(&mut self) -> &mut Vec<u8> {
        &mut self.data_obj
    }    

    /// Constructs a DaaSDoc object from a serialized string
    /// 
    /// # Arguments
    /// 
    /// * serialized: &str - The string that represents the serialized object.</br>
    /// 
    /// #Example
    ///
    /// ```
    /// extern crate daas;
    ///
    /// use daas::doc::DaaSDoc;
    ///
    /// fn main() {
    ///     let serialized = r#"{"_id":"order|clothing|iStore|5000","_rev":null,"source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~5000","index":0,"timestamp":0,"actor_id":""},"hash":"247170281044197649349807793181887586965","previous_hash":"0","nonce":5}]},"meta_data":{},"tags":[],"data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
    ///     let doc = DaaSDoc::from_serialized(&serialized.as_bytes());
  	///     
    ///     assert_eq!(doc.source_uid, 5000);
    /// }
    /// ```
    pub fn from_serialized(serialized: &[u8]) -> DaaSDoc {
		serde_json::from_slice(&serialized).unwrap()
    }

    /// Returns the value of a metadata entry
    /// 
    /// # Arguments
    /// 
    /// * key: String - The key used to identify the name of the metadata property.</br>
    /// * value: String - The value used to define the metadata property.</br>
    ///    
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     doc.add_meta("foowho".to_string(),"me".to_string());
    ///     doc.add_meta("foo".to_string(),"bar".to_string());
    ///   
    ///     println!("foo {}", doc.get_meta("foowho".to_string()) );
    /// }
    /// ```
    pub fn get_meta(&mut self, key: String) -> String{
        self.meta_data.get(&key).unwrap().to_string()
    }

    /// Returns a list of related tags
    ///    
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     doc.add_tag("foo".to_string());
    ///     doc.add_tag("bar".to_string());
    ///   
    ///     println!("Tags: {:?}", doc.get_tags());
    /// }
    /// ```
    pub fn get_tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    /// Determines if a tag is related to the DaaSDoc
    ///    
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     doc.add_tag("foo".to_string());
    ///     doc.add_tag("bar".to_string());
    ///   
    ///     assert!(doc.has_tag("foo".to_string()));
    /// }
    /// ```
    pub fn has_tag(&self, tag: String) -> bool {
        self.tags.contains(&tag)
    }

    /// A shared function that returns the unique identifier
    ///
    /// # Arguments
    ///
    /// * cat: String - The name of the category (e.g.: order).</br>
    /// * sub: String - The name of the subcategory (e.g.: clothing).</br>
    /// * src_name: String - The name of the data source.</br>
    /// * src_uid: usize - The unique identifier that the data source provided.</br>
    ///
    pub fn make_id(cat: String, subcat: String, src_name: String, src_uid: usize) -> String {
        format!("{}{}{}{}{}{}{}",cat, DELIMITER, subcat, DELIMITER, src_name, DELIMITER, src_uid).to_string()
    } 

    /// Serializes the DaaSDoc object
    /// 
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker;
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     
    ///     println!("{:?}", doc.serialize());
    /// }
    /// ```
    pub fn serialize(&mut self) -> String {
		serde_json::to_string(&self).unwrap()
    }

    /// Serializes the DaaSDoc object without the _rev attribute
    /// 
    /// #Example
    ///
    /// ```
    /// #[macro_use] 
    /// extern crate serde_json;
    /// extern crate pbd;
    /// extern crate daas;
    ///
    /// use serde_json::value::*;
    /// use pbd::dua::DUA;
    /// use pbd::dtc::Tracker; 
    /// use daas::doc::{DaaSDoc};
    ///
    /// fn main() {
    ///     let src = "iStore".to_string();
    ///     let src = "iStore".to_string();
    ///     let uid = 5000;
    ///     let cat = "order".to_string();
    ///     let sub = "clothing".to_string();
    ///     let auth = "istore_app".to_string();
    ///     let mut dua = Vec::new();
    ///     dua.push(DUA::new("billing".to_string(),"https://dua.org/agreements/v1/billing.pdf".to_string(),1553988607));
    ///     let tracker = Tracker::new(DaaSDoc::make_id(cat.clone(), sub.clone(), src.clone(), uid.clone()));
    ///     let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, tracker, data);
    ///     
    ///     println!("{:?}", doc.serialize_without_rev());
    /// }
    /// ```
    pub fn serialize_without_rev(&mut self) -> String {
        let no_rev: DaaSDocNoRev = DaaSDocNoRev {
            _id: self._id.clone(),
            source_name: self.source_name.clone(),
            source_uid: self.source_uid.clone(),
            category: self.category.clone(),
            subcategory: self.subcategory.clone(),
            author: self.author.clone(),
            process_ind: self.process_ind.clone(),
            last_updated: get_unix_now!(),
            data_usage_agreements: self.data_usage_agreements.clone(),
            data_tracker: self.data_tracker.clone(),
            data_obj: self.data_obj.clone(),
        };

        let serialized: String = serde_json::to_string(&no_rev).unwrap();

        serialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use std::fs::File;

    fn get_default_daasdoc() -> DaaSDoc {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let dtc = get_dtc(src.clone(),uid.clone(),cat.clone(),sub.clone());
        let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
        let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, dtc, data);

        doc
    }

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
   
    #[test]
    fn test_has_tag_ok() {
        let mut doc = get_default_daasdoc();
        doc.add_tag("foo".to_string());
        doc.add_tag("bar".to_string());
        
        assert_eq!(doc.has_tag("foo".to_string()), true);
        assert_eq!(doc.has_tag("me".to_string()), false);
    }  
    
    #[test]
    fn test_new_obj_ok() {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
        let _doc = get_default_daasdoc();
        
        assert!(true);
    }

    #[test]
    fn test_doc_id_ok() {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let id = format!("{}~{}~{}~{}",cat, sub, src, uid).to_string();
        let dua = get_dua();
        let dtc = get_dtc(src.clone(),uid.clone(),cat.clone(),sub.clone());
        let data = String::from(r#"{"status": "new"}"#).as_bytes().to_vec();
        let doc = DaaSDoc::new(src, uid, cat, sub, auth, dua, dtc, data);
        
        assert_eq!(doc._id, id);
    }

    #[test]
    fn test_doc_rev_empty() {
        let doc = get_default_daasdoc();
        
        assert!(doc._rev.is_none());
    }

    #[test]
    fn test_doc_attributes_ok() {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let doc = get_default_daasdoc();
        
        assert_eq!(doc.author, auth);
        assert_eq!(doc.source_name, src);
        assert_eq!(doc.source_uid, uid);
        assert_eq!(doc.category, cat);
        assert_eq!(doc.subcategory, sub);
        assert_eq!(doc.process_ind, false);
    } 

    #[test]
    fn test_doc_binary_data_ok() {
        let src = "iStore".to_string();
        let uid = 16500;
        let cat = "order".to_string();
        let sub = "music".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let dtc = get_dtc(src.clone(),uid.clone(),cat.clone(),sub.clone());
        
        let mut f = match File::open("./tests/example_audio_clip.mp3") {
            Ok(aud) => aud,
            Err(err) => {
                panic!("Cannot read the audio file: {}",err);
            },
        };

        let mut data = Vec::new();
        f.read_to_end(&mut data).unwrap();

        let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, dtc, data); 
        
        assert_eq!(doc.data_obj_as_ref().len(),764176);
    } 
    
    #[test]
    fn test_doc_data_ok() {
        let doc = get_default_daasdoc();
        let dat: Value = serde_json::from_str(&String::from_utf8(doc.data_obj).unwrap()).unwrap();
        
        assert_eq!(dat.get("status").unwrap(), "new");
    } 

    #[test]
    fn test_from_serialize(){
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let id = format!("{}~{}~{}~{}",cat, sub, src, uid).to_string();
        let serialized = r#"
        {"_id":"order~clothing~iStore~5000","_rev":null,"source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"last_updated":1553988607,
        "data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],
        "data_tracker":{"chain":[{"identifier":{"data_id":"order~clothing~iStore~5000","index":0,"timestamp":0,"actor_id":""},"hash":"247170281044197649349807793181887586965","previous_hash":"0","nonce":5}]},
        "meta_data":{},"tags":[],
        "data_obj":[123,34,115,116,97,116,117,115,34,58,32,34,110,101,119,34,125]}"#;
        let dua = get_dua();
        let doc = DaaSDoc::from_serialized(&serialized.as_bytes());
        let dat: Value = serde_json::from_str(&String::from_utf8(doc.data_obj).unwrap()).unwrap();
  	
        assert_eq!(doc._id, id);
        assert!(doc._rev.is_none());
        assert_eq!(doc.source_name, src);
        assert_eq!(doc.source_uid, uid);
        assert_eq!(doc.category, cat);
        assert_eq!(doc.subcategory, sub);
        assert_eq!(doc.author, auth);
        assert_eq!(doc.process_ind, false);
        assert_eq!(doc.data_usage_agreements[0].agreement_name, dua[0].agreement_name);
		assert_eq!(dat.get("status").unwrap(), "new");
    }    

    #[test]
    fn test_meta_data_ok() {
        let mut doc = get_default_daasdoc();
        doc.add_meta("foo".to_string(),"bar".to_string());
        
        assert_eq!(doc.get_meta("foo".to_string()), "bar");
    }   
    
    #[test]
    fn test_tagging_ok() {
        let mut doc = get_default_daasdoc();
        doc.add_tag("foo".to_string());
        doc.add_tag("bar".to_string());
        
        assert_eq!(doc.get_tags().len(), 2);
    }  
}