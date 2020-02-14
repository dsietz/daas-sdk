use std::error;
use std::fmt;

// struct
#[derive(Debug, Clone)]
pub struct BadKeyPairError;

#[derive(Debug, Clone)]
pub struct BrokerError;

#[derive(Debug, Clone)]
pub struct DaaSDocError;

#[derive(Debug, Clone)]
pub struct DecryptionError;

#[derive(Debug, Clone)]
pub struct EncryptionError;

#[derive(Debug, Clone)]
pub struct RetrieveError;

#[derive(Debug, Clone)]
pub struct TamperedDataError;

#[derive(Debug, Clone)]
pub struct UpsertError;

// enums
pub enum DaaSEventingError {
    BrokerError
}

#[derive(Debug, Clone)]
pub enum DaaSSecurityError {
    BadKeyPairError,
    DecryptionError,
    EncryptionError,
    TamperedDataError,
}

pub enum DaaSStorageError {
    RetrieveError,
    UpsertError,
}

#[derive(Debug)]
pub enum DaaSError {
    DaaSDocError,
    DaaSStorageError,
    DaaSEventingError,
}

//impl
impl fmt::Display for BadKeyPairError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bad key pair provided.")
    }
}
impl error::Error for BadKeyPairError{}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to broker the DaaS document.")
    }
}
impl error::Error for BrokerError{}

impl fmt::Display for DaaSDocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to perform the operation on the DaaS document!")
    }
}
impl error::Error for DaaSDocError {}

impl fmt::Display for DecryptionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to decrypt the DaaS data!")
    }
}
impl error::Error for DecryptionError {}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to encrypt the DaaS data!")
    }
}
impl error::Error for EncryptionError {}

impl fmt::Display for RetrieveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to retrieve the DaaS document.")
    }
}
impl error::Error for RetrieveError{}

impl fmt::Display for TamperedDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DaaS document rejected. Tampered data data detected.")
    }
}
impl error::Error for TamperedDataError{}

impl fmt::Display for UpsertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to save or update the DaaS document.")
    }
}
impl error::Error for UpsertError{}