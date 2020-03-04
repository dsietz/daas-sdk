use super::*;
use super::extractor::{Author, AuthorExtractor};
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