//! The storage module contains the supported platforms for storing DaaS documents.
//! All the storage devices must implement the DaaSDoc traits.
//!

use super::*;
use crate::doc::*;
use crate::errors::*;

/// Trait for storage devices that manage DaaS documents
pub trait DaaSDocStorage {
    fn upsert_daas_doc(&self, daas_doc: DaaSDoc) -> Result<DaaSDoc, UpsertError>;
    fn get_doc_by_id(
        &self,
        doc_id: String,
        doc_rev: Option<String>,
    ) -> Result<DaaSDoc, RetrieveError>;
}

pub mod local;
pub mod s3;
