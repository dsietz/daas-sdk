//! The storage module contains the supported platforms for storing DaaS documents. 
//! All the storage devices must implement the DaaSDoc traits. 
//! 

use super::*;
use crate::doc::*;
use std::error;
use std::fmt;

enum DaaSStorageError {
    RetrieveError,
    UpsertError,
}

#[derive(Debug, Clone)]
pub struct RetrieveError;

impl fmt::Display for RetrieveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to retrieve the DaaS document.")
    }
}

impl error::Error for RetrieveError{}

#[derive(Debug, Clone)]
pub struct UpsertError;

impl fmt::Display for UpsertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to save or update the DaaS document.")
    }
}

impl error::Error for UpsertError{}


/// Trait for storage devices that manage DaaS documents
pub trait DaaSDocStorage {
    fn upsert_daas_doc(&self, mut daas_doc: DaaSDoc) -> Result<DaaSDoc, UpsertError>; 
    fn get_doc_by_id(&self, doc_id: String, doc_rev: Option<String>) -> Result<DaaSDoc, RetrieveError>;
} 

pub mod local;