use super::*;
use std::fmt;
use actix_web::{FromRequest, HttpRequest};

// 
// The Author Extractor
// 
pub type LocalError = MissingAuthorError;

pub trait AuthorExtractor {
    fn extract_author(&mut self, req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Result<String, MissingAuthorError>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    pub name: String,
}

impl Author {    
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: String) -> Result<Self, MissingAuthorError> {
        self.name = name;
        Ok(self.clone())
    }
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl FromRequest for Author where Author: AuthorExtractor {
    type Config = ();
    type Future = Result<Self, Self::Error>;
    type Error = LocalError;
    // convert request to future self
    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let mut author = Author {
            name: "Anonymous".to_string(),
        };
        
        match author.extract_author(req, payload) {
            Ok(name) => {
                author.set_name(name)
            },
            Err(err) => {
                error!("{}", err);
                Err(err)
            },
        }
    }
}

pub mod base64_author {
    use super::*;
    use base64::decode;

    impl AuthorExtractor for Author {
        fn extract_author(&mut self, req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Result<String, MissingAuthorError> {
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
} 