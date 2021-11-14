use futures::future::{err, ok, Ready};
use std::time::SystemTime;

#[macro_export]
macro_rules! author_struct {
    ( $a:ident ) => {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct $a {
            name: String,
        }
    };
}

#[macro_export]
macro_rules! author_fn_get_name {
    () => {
        fn get_name(&self) -> String {
            self.name.clone()
        }
    };
}

#[macro_export]
macro_rules! author_fn_new {
    () => {
        fn new() -> Self {
            Self {
                name: "Anonymous".to_string(),
            }
        }
    };
}

#[macro_export]
macro_rules! author_fn_set_name {
    () => {
        fn set_name(&mut self, name: String) -> Result<Self, MissingAuthorError> {
            self.name = name;
            Ok(self.clone())
        }
    };
}

#[macro_export]
macro_rules! author_from_request {
    ( $a:ty ) => {
        impl FromRequest for $a {
            type Config = ();
            type Future = Ready<Result<Self, Self::Error>>;
            type Error = LocalError;
            // convert request to future self
            fn from_request(
                req: &HttpRequest,
                payload: &mut actix_web::dev::Payload,
            ) -> Self::Future {
                let mut author = <$a>::new();

                match author.extract_author(req, payload) {
                    Ok(name) => match author.set_name(name) {
                        Ok(_) => ok(author),
                        Err(e) => {
                            error!("{}", e);
                            err(e)
                        }
                    },
                    Err(e) => {
                        error!("{}", e);
                        err(e)
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! get_unix_now {
    ( $( $x:expr ),* ) => {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    };
}

/// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::MissingAuthorError;
    use crate::service::extractor::{AuthorExtractor, LocalError};
    use actix_web::{FromRequest, HttpRequest};
    use log::*;
    use std::{thread, time};

    #[test]
    fn test_author() {
        author_struct!(TestAuthor);
        impl AuthorExtractor for TestAuthor {
            fn extract_author(
                &mut self,
                req: &HttpRequest,
                _payload: &mut actix_web::dev::Payload,
            ) -> Result<String, MissingAuthorError> {
                Ok("TestMe".to_string())
            }
            // Use macros to write the default functions
            author_fn_get_name!();
            author_fn_new!();
            author_fn_set_name!();
        }
        // Use macros to write the implmentation of the FromRequest trait
        author_from_request!(TestAuthor);

        let mut test_author = TestAuthor::new();
        test_author.set_name("TestMe".to_string()).unwrap();

        assert_eq!(test_author.get_name(), "TestMe".to_string());
    }

    #[test]
    fn test_get_unix_now() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        thread::sleep(time::Duration::from_secs(1));

        assert!(now < get_unix_now!())
    }
}
