use super::*;
use std::fmt;
//use std::marker::PhantomData;
use actix_web::{FromRequest, HttpRequest};
use base64::decode;

// 
// The Author Extractor
// 
pub type LocalError = MissingAuthorError;

pub trait AuthorExtractor {
    fn extract_author(&mut self, req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Result<String, MissingAuthorError>;
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String) -> Result<Self, MissingAuthorError> where Self: std::marker::Sized;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    name: String,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefaultAuthor {
    name: String,
}

impl fmt::Display for DefaultAuthor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}


impl DefaultAuthor {
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
    
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) -> Result<Self, MissingAuthorError> {
        self.name = name;
        Ok(self.clone())
    }
}
/*
impl<A> FromRequest for AuthorExtractor::<A> {
    type Config = ();
    type Future = Result<Self, Self::Error>;
    type Error = LocalError;
    // convert request to future self
    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {       
        match AuthorExtractor::<A>.extract_author(req, payload) {
            Ok(name) => {
                AuthorExtractor::<A>.set_name(name)
            },
            Err(err) => {
                error!("{}", err);
                Err(err)
            },
        }
    }
}
*/
impl FromRequest for DefaultAuthor {
    type Config = ();
    type Future = Result<Self, Self::Error>;
    type Error = LocalError;
    // convert request to future self
    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let mut author = DefaultAuthor {
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