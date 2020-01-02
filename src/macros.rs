use super::*;

macro_rules! get_unix_now {
    ( $( $x:expr ),* ) => {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) =>n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
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