//! The storage module contains the supported platforms for storing DaaS documents. 
//! All the storage devices must implement the DaaSDoc traits. 
//! 

use super::*;
use crate::doc::*;


/// Trait for storage devices that manage DaaS documents
pub trait DaaSDocStorage {
    fn upsert_daasdoc(self, mut daas_doc: DaaSDoc) -> Result<DaaSDoc, DaaSError>; 
} 

pub mod local;