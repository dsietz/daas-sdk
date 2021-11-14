use super::*;
use actix_web::{FromRequest, HttpRequest};
use base64::decode;
use std::fmt;

//
// The common trait for all Author Extractors
//
pub type LocalError = MissingAuthorError;

pub trait AuthorExtractor {
    fn extract_author(
        &mut self,
        req: &HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Result<String, MissingAuthorError>;
    fn get_name(&self) -> String;
    fn new() -> Self;
    fn set_name(&mut self, name: String) -> Result<Self, MissingAuthorError>
    where
        Self: std::marker::Sized;
}

//
// The Base64Author Extractor
//

// Use macros to crate our Base64Author structure
author_struct!(Base64Author);

impl fmt::Display for Base64Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl AuthorExtractor for Base64Author {
    fn extract_author(
        &mut self,
        req: &HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Result<String, MissingAuthorError> {
        match req.headers().get("Authorization") {
            Some(hdr) => match hdr.to_str() {
                Ok(encoded) => match decode(&encoded.replace("Basic ", "")) {
                    Ok(decoded) => match String::from_utf8(decoded) {
                        Ok(base) => Ok(base.split(':').collect::<Vec<&str>>()[0].to_string()),
                        Err(err) => {
                            debug!("{}", err);
                            Err(MissingAuthorError)
                        }
                    },
                    Err(err) => {
                        debug!("{}", err);
                        Err(MissingAuthorError)
                    }
                },
                Err(err) => {
                    debug!("{}", err);
                    Err(MissingAuthorError)
                }
            },
            None => Err(MissingAuthorError),
        }
    }

    // Use macros to write the default functions
    author_fn_get_name!();
    author_fn_new!();
    author_fn_set_name!();
}

// Use macros to write the implmentation of the FromRequest trait
author_from_request!(Base64Author);
