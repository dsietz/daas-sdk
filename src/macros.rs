use std::time::{SystemTime};

#[macro_export]
macro_rules! get_unix_now {
    ( $( $x:expr ),* ) => {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) =>n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    };
}

#[macro_export]
macro_rules! author_from_request {
    ( $a:ty ) => {
        impl FromRequest for $a {
            type Config = ();
            type Future = Result<Self, Self::Error>;
            type Error = LocalError;
            // convert request to future self
            fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {    
                let mut author = <$a>::new();

                match author.extract_author(req, payload) {
                    Ok(name) => {
                        author.set_name(name);
                        Ok(author)
                    },
                    Err(err) => {
                        error!("{}", err);
                        Err(err)
                    },
                }
            }
        }
    };
}

/// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_get_unix_now() {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        thread::sleep(time::Duration::from_secs(1));

        assert!(now < get_unix_now!())
    }
}