use super::*;
use std::fmt;
//use std::marker::PhantomData;
use actix_web::{FromRequest, HttpRequest};
use base64::decode;

// 
// The Author Extractor
// 
pub type LocalError = MissingAuthorError;

#[derive(Serialize, Debug, Clone)]
pub struct Author {
    name: String,
    //callback: PhantomData<fn(&HttpRequest, &mut actix_web::dev::Payload) -> Result<Self, MissingAuthorError>>,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl Author {
    fn default(req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Result<Self, MissingAuthorError> {
        let default = Self {
                        name: "Anonymous".to_string()
                    };

        match req.headers().get("Authorization") {
            Some(hdr) => {
                match hdr.to_str() {
                    Ok(encoded) => {
                        match decode(&encoded.replace("Basic ","")) {
                            Ok(decoded) => {
                                match String::from_utf8(decoded) {
                                    Ok(base) => {
                                        Ok(Self {
                                            name: base.split(':').collect::<Vec<&str>>()[0].to_string(),
                                        })
                                    },
                                    Err(err) => {
                                        debug!("{}", err);
                                        Ok(default)
                                    },
                                }
                            },
                            Err(err) => {
                                debug!("{}", err);
                                Ok(default)
                            },
                        }
                    },
                    Err(err) => {
                        debug!("{}", err);
                        Ok(default)
                    },
                }
            },
            None => {
                Ok(default)
            },
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    fn new(req: &HttpRequest, payload: &mut actix_web::dev::Payload, callback: fn(&HttpRequest, &mut actix_web::dev::Payload) -> Result<Self, MissingAuthorError>) -> Result<Author, MissingAuthorError> {
        callback(req, payload)
    }
}

impl FromRequest for Author {
    type Config = ();
    type Future = Result<Self, Self::Error>;
    type Error = LocalError;
    // convert request to future self
    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        Self::new(req, payload, Self::default)
    }
}

