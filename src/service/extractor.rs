use super::*;
use std::fmt;
use crate::service::listener::*;
use actix_web::{FromRequest, HttpRequest};
use actix_web::http::header::HeaderValue;
use base64::decode;

// 
// The Author Extractor
// 
pub type LocalError = MissingAuthorError;

pub trait AuthorExtractor {
    fn new(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Result<Author, MissingAuthorError>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    pub name: String,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl Author {
    pub fn default() -> Author {
        let def_auth = "Anonymous".to_string();
        warn!("Using [{}] as author.", def_auth);
        
        Author {
            name: def_auth,
        }
    }
}

impl AuthorExtractor for Author {
    fn new(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Result<Self,MissingAuthorError> {
        match req.headers().get("Authorization") {
            Some(hdr) => {
                match hdr.to_str() {
                    Ok(encoded) => {
                        match decode(&encoded.replace("Basic ","")) {
                            Ok(decoded) => {
                                match String::from_utf8(decoded) {
                                    Ok(base) => {
                                        Ok(Author {
                                            name: base.split(':').collect::<Vec<&str>>()[0].to_string(),
                                        })
                                    },
                                    Err(err) => {
                                        debug!("{}", err);
                                        Ok(Self::default())
                                    },
                                }
                            },
                            Err(err) => {
                                debug!("{}", err);
                                Ok(Self::default())
                            },
                        }
                    },
                    Err(err) => {
                        debug!("{}", err);
                        Ok(Self::default())
                    },
                }
            },
            None => {
                Ok(Self::default())
            },
        }
    }
}

impl FromRequest for Author {
    type Config = ();
    type Future = Result<Self, Self::Error>;
    type Error = LocalError;
    // convert request to future self
    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        Self::new(req, payload)
    }
}

