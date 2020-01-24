use super::*;
use crate::errors::{BadKeyPairError, DecryptionError, DaaSSecurityError};
use std::cmp::max;
use openssl::rsa::{Rsa, Padding};
use openssl::symm::{decrypt, encrypt, Cipher};
use rand::Rng; 
use rand::distributions::Alphanumeric;
/*
Due to the mathematics (and padding) behind RSA encryption, you can only encrypt very small values.
In order to use RSA encryption with larger values, typically you generate a symmetric key for use with another algorithm, such as AES. 
Then you encrypt the data using the AES symmetric key (there is no limitation on size using a symmetric encryption algorithm) and then you RSA-encrypt 
just the symmetric key and transmit that. AES keys are 16-32 bytes in size so they can easily fit within the RSA-encryption limitations.
Then the recipient decrypts the symmetric key using their private RSA key and then they decrypt the encrypted data using the decrypted symmetric key.
RSA encryption is also much slower than AES encryption, so this yields better performance anyway.

SEE ALSO: https://docs.rs/openssl/0.10.26/openssl/aes/index.html
*/

// priv_key = the priuvate key as pem
trait DaaSSecurityGaurd{
    fn generate_keypair(&self) -> Result<(Vec<u8>,Vec<u8>,usize),DaaSSecurityError>{
        let rsa = Rsa::generate(2048).unwrap();
        let priv_key: Vec<u8> = rsa.private_key_to_pem().unwrap();
        let pub_key: Vec<u8> = rsa.public_key_to_pem().unwrap();
    
        Ok((priv_key, pub_key, rsa.size() as usize))
    }

    fn generate_symmetric_key(&self) -> Vec<u8>{
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect::<String>()
            .as_bytes()
            .to_vec()
    }

    fn generate_nonce(&self) -> Vec<u8>{
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .collect::<String>()
            .as_bytes()
            .to_vec()
    }

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
   
    fn decrypt_symmetric_key(&self, priv_key: Vec<u8>, encrypted_data: Vec<u8>, padding: Padding) -> Result<Vec<u8>,DaaSSecurityError> {
        let receiver = match Rsa::private_key_from_pem(&priv_key) {
            Ok(rsa) => rsa,
            Err(err) => {
                debug!("{}", err);
                return Err(DaaSSecurityError::BadKeyPairError);
            },
        };
        //let sz = std::cmp::max(encrypted_data.len() as usize, priv_key.len() as usize);
        let mut message: Vec<u8> = vec![0; encrypted_data.len()];
        
        match receiver.private_decrypt(&encrypted_data, message.as_mut_slice(), padding){
            Ok(_sz) => {
                Ok(self.clean_decrypted(message))
            },
            Err(err) => {
                debug!("{}", err);
                return Err(DaaSSecurityError::DecryptionError);
            },
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::prelude::*;
    use std::fs::File;

    struct Guard {}
    impl DaaSSecurityGaurd for Guard{}

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
        let guard = Guard {};
        let nonce = guard.generate_nonce();
        println!("{:?}", nonce);
        assert_eq!(nonce.len(),16);        
    }

    #[test]
    fn test_generate_symmetric_key() {
        let guard = Guard {};
        let key = guard.generate_symmetric_key();
        println!("{:?}", key);
        assert_eq!(key.len(),16);        
    }

    #[test]
    fn test_generate_keypair() {
        let guard = Guard {};
        let keypair = guard.generate_keypair();
        assert!(keypair.is_ok());        
    }

    #[test]
    fn test_decrypt_data() {
        let guard = Guard {};
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
        let guard = Guard {};
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
    fn test_encrypt_decrypt_mp3() {
        let guard = Guard {};
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
        let guard = Guard {};
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
