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
//! use daas::doc::{DaaSDoc};
//!
//! fn main() {
//!		let src = "iStore".to_string();
//!		let uid = 5000;
//!		let cat = "order".to_string();
//!		let sub = "clothing".to_string();
//!		let auth = "istore_app".to_string();
//!     let mut dua = Vec::new();
//!     dua.push(DUA {
//!         agreement_name: "billing".to_string(),
//!         location: "https://dua.org/agreements/v1/billing.pdf".to_string(),
//!         agreed_dtm: 1553988607,
//!     });
//!		let data = json!({
//!         "product": "leather coat",
//!         "quantity": 1,
//!		    "status": "new"
//!		});
//! 
//!		let doc = DaaSDoc::new(src, uid, cat, sub, auth, dua, data);
//! 
//!     assert_eq!(doc.source_uid, uid);
//! }
//! ```

use crate::*;
use std::error;
use std::fmt;
use std::collections::BTreeMap;
use serde_json::value::*;
use futures::Future;
use pbd::dua::DUA;

// Repesentation of a map for storing metadata about the data object
type Metadata = BTreeMap<String, String>;

/// Traits
pub trait UpsertDaaSDoc {
    fn upsert_daasdoc(self, mut daas_doc: DaaSDoc) -> Result< DaaSDoc, DaaSError>; 
} 

/// Defining a DaaS Error
/// see https://doc.rust-lang.org/src/std/io/error.rs.html#229-512 for further details
#[derive(Debug, Clone)]
pub struct DaaSError;

/// Define how a DaaSError will be displayed
impl fmt::Display for DaaSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

// This is important for other errors to wrap this one.
impl error::Error for DaaSError {
    fn description(&self) -> &str {
        "Unable to perform the DaaS operation"
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

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
    // The list of metadata about the data object (key, value)
    pub meta_data: Metadata,
    // List of tags to provide context about the data object
    pub tags: Vec<String>,
    /// The JSON value that represents the data from the data source managed by the DaaS document
    pub data_obj: Value,
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
    /// The JSON value that represents the data from the data source managed by the DaaS document
    pub data_obj: Value,
}

impl DaaSDoc {
    /// Delimiter used for building the unique identifier value for the DaaS document
    pub const DELIMITER: &'static str = "|";

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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
    ///     
    ///     println!("{:?}", doc._id);
    /// }
    /// ```
    pub fn new(src_name: String, src_uid: usize, cat: String, subcat: String, auth: String, duas: Vec<DUA>, data: Value) -> DaaSDoc {
        DaaSDoc {
            _id: DaaSDoc::make_id(cat.clone(), subcat.clone(), src_name.clone(), src_uid),
            _rev: None,
            source_name: src_name,
            source_uid: src_uid,
            category: cat,
            subcategory: subcat,
            author: auth,
            process_ind: false,
            last_updated: get_unix_now!(),
            data_usage_agreements: duas,
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
    ///     
    ///     assert_eq!(doc.data_obj.get("status").unwrap(), "new");
    /// }
    /// ```
    pub fn data_obj_as_ref(&mut self) -> &mut Value {
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
    ///     let serialized = r#"{"_id":"order|clothing|iStore|5000","_rev":null,"source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":[],"data_obj":{"status":"new"}}"#;
    ///     let doc = DaaSDoc::from_serialized(&serialized);
  	///     
    ///     assert_eq!(doc.source_uid, 5000);
    /// }
    /// ```
    pub fn from_serialized(serialized: &str) -> DaaSDoc {
		serde_json::from_str(&serialized).unwrap()
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
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
    fn make_id(cat: String, subcat: String, src_name: String, src_uid: usize) -> String {
        format!("{}{}{}{}{}{}{}",cat, DaaSDoc::DELIMITER, subcat, DaaSDoc::DELIMITER, src_name, DaaSDoc::DELIMITER, src_uid).to_string()
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
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
    ///     let data = json!({
    ///         "status": "new"
    ///     });
    ///     
    ///     let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
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
            data_obj: self.data_obj.clone(),
        };

        let serialized: String = serde_json::to_string(&no_rev).unwrap();

        serialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_dua() -> Vec<DUA>{
        let mut v = Vec::new();
        v.push( DUA {
                    agreement_name: "billing".to_string(),
                    location: "www.dua.org/billing.pdf".to_string(),
                    agreed_dtm: 1553988607,
                });
        v
    }

    #[test]
    fn test_dua_from_serialized() {
        let serialized = r#"{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}"#;
        let dua = DUA::from_serialized(serialized);

        assert_eq!(dua.agreement_name, "billing".to_string());
        assert_eq!(dua.location, "www.dua.org/billing.pdf".to_string());
        assert_eq!(dua.agreed_dtm, 1553988607);
    }

    #[test]
    fn test_dua_serialize() {
        let serialized = r#"{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}"#;
        let dua = &mut get_dua()[0];

        assert_eq!(dua.serialize(), serialized);
    }

    
    #[test]
    fn test_has_tag_ok() {
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
        let data = json!({
            "status": "new"
        });
        let _doc = DaaSDoc::new(src, uid, cat, sub, auth, dua, data);
        
        assert!(true);
    }

    #[test]
    fn test_doc_id_ok() {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let id = format!("{}|{}|{}|{}",cat, sub, src, uid).to_string();
        let dua = get_dua();
        let data = json!({
            "status": "new"
        });
        let doc = DaaSDoc::new(src, uid, cat, sub, auth, dua, data);
        
        assert_eq!(doc._id, id);
    }

    #[test]
    fn test_doc_rev_empty() {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let data = json!({
            "status": "new"
        });
        let doc = DaaSDoc::new(src, uid, cat, sub, auth, dua, data);
        
        assert!(doc._rev.is_none());
    }

    #[test]
    fn test_doc_attributes_ok() {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let data = json!({
            "status": "new"
        });
        let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
        
        assert_eq!(doc.source_name, src);
        assert_eq!(doc.source_uid, uid);
        assert_eq!(doc.category, cat);
        assert_eq!(doc.subcategory, sub);
        assert_eq!(doc.process_ind, false);
    } 

    #[test]
    fn test_doc_data_ok() {
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let data = json!({
            "status": "new"
        });
        let doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth.clone(), dua, data);
        
        assert_eq!(doc.data_obj.get("status").unwrap(), "new");
    }     

    #[test]
    fn test_from_serialize(){
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let id = format!("{}|{}|{}|{}",cat, sub, src, uid).to_string();
        let serialized = r#"{"_id":"order|clothing|iStore|5000","_rev":null,"source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"last_updated":1553988607,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":[],"data_obj":{"status":"new"}}"#;
        let dua = get_dua();
        let doc = DaaSDoc::from_serialized(&serialized);
  	
        assert_eq!(doc._id, id);
        assert!(doc._rev.is_none());
        assert_eq!(doc.source_name, src);
        assert_eq!(doc.source_uid, uid);
        assert_eq!(doc.category, cat);
        assert_eq!(doc.subcategory, sub);
        assert_eq!(doc.author, auth);
        assert_eq!(doc.process_ind, false);
        assert_eq!(doc.data_usage_agreements[0].agreement_name, dua[0].agreement_name);
		assert_eq!(doc.data_obj.get("status").unwrap(), "new");
    }    

    #[test]
    fn test_meta_data_ok() {
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
        doc.add_meta("foo".to_string(),"bar".to_string());
        
        assert_eq!(doc.get_meta("foo".to_string()), "bar");
    }   

    // fails becuase of the last_updated unix timestamp is genrated at runtime
    #[ignore]
    #[test]
    fn test_serialize(){
        let src = "iStore".to_string();
        let uid = 5000;
        let cat = "order".to_string();
        let sub = "clothing".to_string();
        let auth = "istore_app".to_string();
        let dua = get_dua();
        let data = json!({
            "status": "new"
        });
        let mut doc = DaaSDoc::new(src.clone(), uid, cat.clone(), sub.clone(), auth, dua, data);
        let serialized = r#"{"_id":"order|clothing|iStore|5000","_rev":null,"source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"meta_data":{},"tags":["foo","bar"],"data_obj":{"status":"new"}}"#;
        doc.add_tag("foo".to_string());
        doc.add_tag("bar".to_string());
		assert_eq!(doc.serialize(), serialized);
    }    

    // fails becuase of the last_updated unix timestamp is genrated at runtime
    #[ignore]
    #[test]
    fn test_serialize_without_rev(){
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
        let no_rev = r#"{"_id":"order|clothing|iStore|5000","source_name":"iStore","source_uid":5000,"category":"order","subcategory":"clothing","author":"istore_app","process_ind":false,"data_usage_agreements":[{"agreement_name":"billing","location":"www.dua.org/billing.pdf","agreed_dtm":1553988607}],"data_obj":{"status":"new"}}"#;
		
        assert_eq!(doc.serialize_without_rev(), no_rev.to_string());
    } 
    
    #[test]
    fn test_tagging_ok() {
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
        doc.add_tag("foo".to_string());
        doc.add_tag("bar".to_string());
        
        assert_eq!(doc.get_tags().len(), 2);
    }  
}