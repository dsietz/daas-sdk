use super::*;
use std::fmt;
use actix_web::{FromRequest, HttpRequest};
use base64::decode;

// 
// The Author Extractor
// 
pub type LocalError = MissingAuthorError;

pub trait AuthorExtractor {
    fn extract_author(&mut self, req: &HttpRequest) -> Result<String, MissingAuthorError>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {}

/// Base64Author
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Base64Author {}

impl AuthorExtractor for Base64Author {  
    fn extract_author(&mut self, req: &HttpRequest) -> Result<String, MissingAuthorError> {
        match req.headers().get("Authorization") {
            Some(hdr) => {
                match hdr.to_str() {
                    Ok(encoded) => {
                        match decode(&encoded.replace("Basic ","")) {
                            Ok(decoded) => {
                                match String::from_utf8(decoded) {
                                    Ok(base) => {
                                        Ok(base.split(':').collect::<Vec<&str>>()[0].to_string())
                                    },
                                    Err(err) => {
                                        debug!("{}", err);
                                        Err(MissingAuthorError)
                                    },
                                }
                            },
                            Err(err) => {
                                debug!("{}", err);
                                Err(MissingAuthorError)
                            },
                        }
                    },
                    Err(err) => {
                        debug!("{}", err);
                        Err(MissingAuthorError)
                    },
                }
            },
            None => {
                Err(MissingAuthorError)
            },
        }
    }
}