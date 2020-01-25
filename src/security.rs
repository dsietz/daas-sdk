//! The `security` module provides the functionality and structures that the Data as a Service (DaaS) utilizes to enforce the Privacy by Design `Deidentification` strategy.
//! 
//! These security features can be implemented in two manners:
//! 
//! 1. Instantiating a `DaaSGuard` object and calling it's methods
//! 2. Implementing the `DaaSSecurityGuard` traits for your own defined structure
//!
//! # Examples
//!
//! Utilizing the DaaSGuard structure to generate a RSA keypair
//!
//! ```
//! extern crate daas;
//!
//! use daas::security::{DaaSGuard, DaaSSecurityGuard};
//!
//! fn main() {
//!     let guard = DaaSGuard {};
//!     let keypair = guard.generate_keypair();
//!     assert!(keypair.is_ok());    
//! }
//! ```
//! 
//! Implementing the DaaSSecurityGuard trait to generate a RSA keypair
//!
//! ```
//! extern crate daas;
//!
//! use daas::security::{DaaSSecurityGuard};
//!
//! fn main() {
//!     struct MyStruct {}
//!     impl MyStruct {
//!         fn hello(&self) -> String {
//!             "Hello World!".to_string()
//!         }
//!     }
//!     impl DaaSSecurityGuard for MyStruct {}
//! 
//!     let my_obj = MyStruct {};
//!     let keypair = my_obj.generate_keypair();
//! 
//!     println!("{}", my_obj.hello());
//!     assert!(keypair.is_ok());    
//! }
//! ```
//! 
//! Following the steps to send data from source to target in a secured manner
//!
//! 1. Generate a symmetric key for use with another algorithm using AES
//! 2. Encrypt the data using the AES symmetric key
//! 3. Then you RSA-encrypt the symmetric key and transmit that
//! 
//! ```
//! extern crate daas;
//! extern crate openssl;
//! 
//! use daas::security::{DaaSGuard, DaaSSecurityGuard};
//! use openssl::rsa::{Padding};
//! use std::io;
//! use std::io::prelude::*;
//! use std::fs::File;
//! 
//! fn get_priv_pem() -> Vec<u8> {
//!     let mut f = File::open("./tests/keys/priv-key.pem").unwrap();
//!     let mut priv_pem = Vec::new();
//!     f.read_to_end(&mut priv_pem).unwrap();
//!     
//!     priv_pem
//! }
//! 
//! fn get_pub_pem() -> Vec<u8> {
//!     let mut f = File::open("./tests/keys/pub-key.pem").unwrap();
//!     let mut pub_pem = Vec::new();
//!     f.read_to_end(&mut pub_pem).unwrap();
//!     
//!     pub_pem
//! }
//!
//! fn main() {
//!     let priv_key = get_priv_pem();
//!     let pub_key = get_pub_pem();
//!     let guard = DaaSGuard {};
//!     let key = guard.generate_symmetric_key();
//!     let nonce = guard.generate_nonce();
//!     let padding = Padding::PKCS1;
//!     let mut f = File::open("./tests/example_audio_clip.mp3").unwrap();
//!     let mut mp3 = Vec::new();
//!     f.read_to_end(&mut mp3).unwrap();
//!     
//!     // 1. encrypt the mps data using the symmetric key
//!     let encrypted_data = match guard.encrypt_data(key.clone(), Some(&nonce.clone()), mp3.clone()) {
//!         Ok(msg) => {
//!             assert!(true);
//!             msg                
//!         },
//!         Err(err) => {
//!             assert!(false);
//!             panic!("{:?}", err);
//!         },
//!     };    
//!     
//!     // 2. Encrypt the symmetric key
//!     let encrypted_key = match guard.encrypt_symmetric_key(pub_key, key.clone(), padding) {
//!         Ok(e_key) => {
//!             assert_eq!(e_key.len(), 256);
//!             e_key
//!         },
//!         Err(err) => {
//!             assert!(false);
//!             panic!("{:?}", err);
//!         },
//!     };
//!     
//!     // The data source sends the following items to the recipient: 
//!     // + padding
//!     // + nonce
//!     // + encrypted symmetric key
//!     // + encrypted mps data
//!     
//!     // 3. Decrypt the symmetric key
//!     let decrypted_key = match guard.decrypt_symmetric_key(priv_key, encrypted_key, padding) {
//!         Ok(e_key) => {
//!             assert_eq!(e_key.len(), 16);
//!             e_key
//!         },
//!         Err(err) => {
//!             assert!(false);
//!             panic!("{:?}", err);
//!         },
//!     };
//!     
//!     // 4. Decrypt the data using the symmetric key
//!     let decrypted_data = match guard.decrypt_data(key, Some(&nonce), encrypted_data) {
//!         Ok(msg) => {
//!             assert!(true);
//!             msg                
//!         },
//!         Err(err) => {
//!             assert!(false);
//!             panic!("{:?}", err);
//!         },
//!     }; 
//!     
//!     assert_eq!(mp3, decrypted_data);
//! }
//! ```
use super::*;
use crate::errors::{BadKeyPairError, DecryptionError, DaaSSecurityError};
use std::cmp::max;
use openssl::rsa::{Rsa, Padding};
use openssl::symm::{decrypt, encrypt, Cipher};
use rand::Rng; 
use rand::distributions::Alphanumeric;

/// Trait that provides the DaaS security functionality 
pub trait DaaSSecurityGuard{
    /// Generates a RSA (private/public) keypair
    fn generate_keypair(&self) -> Result<(Vec<u8>,Vec<u8>,usize),DaaSSecurityError>{
        let rsa = Rsa::generate(2048).unwrap();
        let priv_key: Vec<u8> = rsa.private_key_to_pem().unwrap();
        let pub_key: Vec<u8> = rsa.public_key_to_pem().unwrap();
    
        Ok((priv_key, pub_key, rsa.size() as usize))
    }

    /// Generates a random alphanumeric key with a length of 16 characters
    fn generate_symmetric_key(&self) -> Vec<u8>{
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect::<String>()
            .as_bytes()
            .to_vec()
    }

    /// Generates a random alphanumeric nonce (a.k.a. IV) with a length of 16 characters
    fn generate_nonce(&self) -> Vec<u8>{
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect::<String>()
            .as_bytes()
            .to_vec()
    }

    /// Decrypts the data (small or large) using the symmetric key, IV and AES encryption algorithm
    fn decrypt_data(&self, key: Vec<u8>, nonce: Option<&[u8]>, data_to_decrypt: Vec<u8>) -> Result<Vec<u8>, DaaSSecurityError> {
        match decrypt(Cipher::aes_128_cbc(), &key, nonce, &data_to_decrypt) {
            Ok(data) => {
                Ok(data)
            },
            Err(err) => {
                println!("{}", err);
                Err(DaaSSecurityError::DecryptionError)
            },
        }
    }

    /// Encrypts the data (small or large) using the symmetric key, IV and AES encryption algorithm
    fn encrypt_data(&self, key: Vec<u8>, nonce: Option<&[u8]>, data_to_encrypt: Vec<u8>) -> Result<Vec<u8>, DaaSSecurityError> {
        match encrypt(Cipher::aes_128_cbc(), &key, nonce, &data_to_encrypt) {
            Ok(cipherdata) => {
                Ok(cipherdata)
            },
            Err(err) => {
                println!("{}", err);
                Err(DaaSSecurityError::EncryptionError)
            },
        }
    }

    /// Decrypts the symmetric key using RSA algorithm for the specified padding
    fn decrypt_symmetric_key(&self, priv_key: Vec<u8>, encrypted_key: Vec<u8>, padding: Padding) -> Result<Vec<u8>,DaaSSecurityError> {
        let receiver = match Rsa::private_key_from_pem(&priv_key) {
            Ok(rsa) => rsa,
            Err(err) => {
                debug!("{}", err);
                return Err(DaaSSecurityError::BadKeyPairError);
            },
        };
        //let sz = std::cmp::max(encrypted_data.len() as usize, priv_key.len() as usize);
        let mut message: Vec<u8> = vec![0; encrypted_key.len()];
        
        match receiver.private_decrypt(&encrypted_key, message.as_mut_slice(), padding){
            Ok(_sz) => {
                Ok(self.clean_decrypted(message))
            },
            Err(err) => {
                debug!("{}", err);
                return Err(DaaSSecurityError::DecryptionError);
            },
        }
    }

    /// Encrypts the symmetric key using RSA algorithm for the specified padding
    fn encrypt_symmetric_key(&self, pub_key: Vec<u8>, key_to_encrypt: Vec<u8>, padding: Padding) -> Result<Vec<u8>,DaaSSecurityError> {
        let sender = match Rsa::public_key_from_pem(&pub_key){
            Ok(rsa) => rsa,
            Err(err) => {
                debug!("{}", err);
                return Err(DaaSSecurityError::BadKeyPairError);
            },
        };
        let mut encrypted_data: Vec<u8> = vec![0; sender.size() as usize];
        sender.public_encrypt(&key_to_encrypt, encrypted_data.as_mut_slice(), padding).unwrap(); 

        Ok(encrypted_data)
    }

    /// Removes the control NUL characters form the decrypted message
    fn clean_decrypted(&self, mut message: Vec<u8>) -> Vec<u8> {
        //remove the control NUL characters
        let zero: u8 = 0;
        let mut c: usize = 0;
        let mut message_trimmed: Vec<u8> = Vec::new();

        for chr in message {
            if chr.is_ascii_control() && chr == zero {
                c = c + 1;
            } else {
                message_trimmed.push(chr);
            }
        }

        debug!("There are {} zero control characters.", c);
        message_trimmed
    }
}

/// Represents the DaaS Security Gaurd
pub struct DaaSGuard {}
impl DaaSSecurityGuard for DaaSGuard{}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::prelude::*;
    use std::fs::File;

    fn get_priv_pem() -> Vec<u8> {
        let mut f = File::open("./tests/keys/priv-key.pem").unwrap();
        let mut priv_pem = Vec::new();
        f.read_to_end(&mut priv_pem).unwrap();
        
        priv_pem
    }

    fn get_pub_pem() -> Vec<u8> {
        let mut f = File::open("./tests/keys/pub-key.pem").unwrap();
        let mut pub_pem = Vec::new();
        f.read_to_end(&mut pub_pem).unwrap();
        
        pub_pem
    }

    #[test]
    fn test_generate_nonce() {
        let guard = DaaSGuard {};
        let nonce = guard.generate_nonce();
        println!("{:?}", nonce);
        assert_eq!(nonce.len(),16);        
    }

    #[test]
    fn test_generate_symmetric_key() {
        let guard = DaaSGuard {};
        let key = guard.generate_symmetric_key();
        println!("{:?}", key);
        assert_eq!(key.len(),16);        
    }

    #[test]
    fn test_generate_keypair() {
        let guard = DaaSGuard {};
        let keypair = guard.generate_keypair();
        assert!(keypair.is_ok());        
    }

    #[test]
    fn test_decrypt_data() {
        let guard = DaaSGuard {};
        let key: &[u8] = &[120, 70, 69, 82, 79, 54, 69, 104, 122, 119, 49, 97, 73, 120, 120, 80];
        let nonce: &[u8] = &[116, 85, 83, 118, 121, 112, 103, 50, 99, 101, 54, 105, 67, 54, 51, 88];
        let message_received: &[u8] = &[89, 60, 190, 161, 62, 26, 88, 4, 100, 161, 230, 105, 14, 4, 162, 163];

        match guard.decrypt_data(key.to_vec(), Some(&nonce), message_received.to_vec()) {
            Ok(msg) => {
                assert_eq!("_test123!# ".to_string(), String::from_utf8(msg).unwrap());
            },
            Err(err) => {
                assert!(false);
            },
        }
    }

    #[test]
    fn test_encrypt_data() {
        let guard = DaaSGuard {};
        let key = guard.generate_symmetric_key();
        let nonce = guard.generate_nonce();
        let message_sent: Vec<u8> = String::from("_test123!# ").into_bytes();

        match guard.encrypt_data(key, Some(&nonce), message_sent) {
            Ok(msg) => {
                assert!(true);
            },
            Err(err) => {
                assert!(false);
            },
        }
    }

    #[test]
    fn test_happy_path_mp3() {
        let priv_key = get_priv_pem();
        let pub_key = get_pub_pem();
        let guard = DaaSGuard {};
        let key = guard.generate_symmetric_key();
        let nonce = guard.generate_nonce();
        let padding = Padding::PKCS1;
        let mut f = File::open("./tests/example_audio_clip.mp3").unwrap();
        let mut mp3 = Vec::new();
        f.read_to_end(&mut mp3).unwrap();

        // 1. encrypt the mps data using the symmetric key
        let encrypted_data = match guard.encrypt_data(key.clone(), Some(&nonce.clone()), mp3.clone()) {
            Ok(msg) => {
                assert!(true);
                msg                
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        };    

        // 2. Encrypt the symmetric key
        let encrypted_key = match guard.encrypt_symmetric_key(pub_key, key.clone(), padding) {
            Ok(e_key) => {
                assert_eq!(e_key.len(), 256);
                e_key
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        };

        // The data source sends the following items to the recipient: 
        // + padding
        // + nonce
        // + encrypted symmetric key
        // + encrypted mps data

        // 3. Decrypt the symmetric key
        let decrypted_key = match guard.decrypt_symmetric_key(priv_key, encrypted_key, padding) {
            Ok(e_key) => {
                assert_eq!(e_key.len(), 16);
                e_key
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        };

        // 4. Decrypt the data using the symmetric key
        let decrypted_data = match guard.decrypt_data(key, Some(&nonce), encrypted_data) {
            Ok(msg) => {
                assert!(true);
                msg                
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        }; 

        assert_eq!(mp3, decrypted_data);
    }

    #[test]
    fn test_encrypt_decrypt_mp3() {
        let guard = DaaSGuard {};
        let key = guard.generate_symmetric_key();
        let nonce = guard.generate_nonce();

        let mut f = File::open("./tests/example_audio_clip.mp3").unwrap();
        let mut mp3 = Vec::new();
        f.read_to_end(&mut mp3).unwrap();

        let encrypted_data = match guard.encrypt_data(key.clone(), Some(&nonce.clone()), mp3.clone()) {
            Ok(msg) => {
                assert!(true);
                msg                
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        };        

        let decrypted_data = match guard.decrypt_data(key, Some(&nonce), encrypted_data) {
            Ok(msg) => {
                assert!(true);
                msg                
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        }; 

        assert_eq!(mp3, decrypted_data);
    }  
    
    #[test]
    fn test_encrypt_decrypt_symmetric_key() {
        let guard = DaaSGuard {};
        let keypair = guard.generate_keypair().unwrap();
        let priv_key = keypair.0;
        let pub_key = keypair.1;
        let rsa_size = keypair.2;
        let padding = Padding::PKCS1;
        let key = guard.generate_symmetric_key();
        let nonce = guard.generate_nonce();
        let message_sent: Vec<u8> = String::from("_test123!# ").into_bytes();

        let encrypted_key = match guard.encrypt_symmetric_key(pub_key, key.clone(), padding) {
            Ok(e_key) => {
                assert_eq!(e_key.len(), 256);
                e_key
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        };

        let decrypted_key = match guard.decrypt_symmetric_key(priv_key, encrypted_key, padding) {
            Ok(e_key) => {
                assert_eq!(e_key.len(), 16);
                e_key
            },
            Err(err) => {
                assert!(false);
                panic!("{:?}", err);
            },
        };

        assert_eq!(key, decrypted_key);
    }
}