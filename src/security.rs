use super::*;
use crate::errors::{BadKeyPairError, DecryptionError, DaaSSecurityError};
use std::cmp::max;
use openssl::rsa::{Rsa, Padding};

/*
Due to the mathematics (and padding) behind RSA encryption, you can only encrypt very small values.
In order to use RSA encryption with larger values, typically you generate a symmetric key for use with another algorithm, such as AES. Then you encrypt the data using the AES symmetric key (there is no limitation on size using a symmetric encryption algorithm) and then you RSA-encrypt just the symmetric key and transmit that. AES keys are 16-32 bytes in size so they can easily fit within the RSA-encryption limitations.
Then the recipient decrypts the symmetric key using their private RSA key and then they decrypt the encrypted data using the decrypted symmetric key.
RSA encryption is also much slower than AES encryption, so this yields better performance anyway.

SEE ALSO: https://docs.rs/openssl/0.10.26/openssl/aes/index.html
*/


fn generate_keypair() -> Result<(Vec<u8>,Vec<u8>,usize),DaaSSecurityError>{
    let rsa = Rsa::generate(2048).unwrap();
    let priv_key: Vec<u8> = rsa.private_key_to_pem().unwrap();
    let pub_key: Vec<u8> = rsa.public_key_to_pem().unwrap();

    Ok((priv_key, pub_key, rsa.size() as usize))
}

// priv_key = the priuvate key as pem
trait DaaSSecurityGaurd{
    fn encrypt_data(&self, pub_key: Vec<u8>, data_to_encrypt: Vec<u8>, padding: Padding) -> Result<Vec<u8>,DaaSSecurityError> {
        let sender = match Rsa::public_key_from_pem(&pub_key){
            Ok(rsa) => rsa,
            Err(err) => {
                debug!("{}", err);
                return Err(DaaSSecurityError::BadKeyPairError);
            },
        };
        let mut encrypted_data: Vec<u8> = vec![0; sender.size() as usize];
        sender.public_encrypt(&data_to_encrypt, encrypted_data.as_mut_slice(), padding).unwrap(); 

        Ok(encrypted_data)
    }
    
    fn decrypt_data(&self, priv_key: Vec<u8>, encrypted_data: Vec<u8>, padding: Padding) -> Result<Vec<u8>,DaaSSecurityError> {
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
    fn test_generate_keypair() {
        let keypair = generate_keypair();
        assert!(keypair.is_ok());        
    }

    #[test]
    fn test_encrypt() {
        let pub_pem = get_pub_pem();
        let message_sent: Vec<u8> = String::from("_test123!# ").into_bytes();
        let padding = Padding::PKCS1;        

        struct Guard {};
        impl DaaSSecurityGaurd for Guard{};
        let guard = Guard {};

        match guard.encrypt_data(pub_pem, message_sent.clone(), padding){
            Ok(encrypted_data) => {
                debug!("Sent: {}", String::from_utf8(message_sent.to_vec()).unwrap());
                debug!("Encrypted Data: {:?}", encrypted_data.clone());
                assert_eq!(encrypted_data.len(), 256);
            },
            Err(err) => {
                debug!("{:?}", err);
                assert!(false);
            },
        }
    }

    #[test]
    fn test_decrypt() {
        let priv_pem = get_priv_pem();
        let padding = Padding::PKCS1;
        let message_sent: Vec<u8> = String::from("_test123!# ").into_bytes();      
        let encrypted_data: [u8; 256] = [114, 148, 223, 139, 68, 254, 180, 209, 175, 6, 170, 62, 239, 205, 52, 1, 87, 217, 50, 77, 228, 115, 161, 216, 58, 130, 138, 193, 96, 113, 204, 104, 31, 175, 68, 139, 176, 82, 146, 149, 118, 163, 111, 190, 192, 121, 209, 88, 15, 156, 219, 68, 195, 187, 181, 223, 212, 223, 228, 83, 78, 127, 215, 85, 250, 6, 190, 88, 120, 220, 95, 16, 30, 113, 2, 69, 210, 29, 183, 245, 95, 9, 121, 23, 138, 66, 108, 74, 125, 50, 249, 99, 218, 6, 178, 2, 155, 90, 141, 188, 192, 171, 231, 7, 21, 125, 211, 89, 157, 153, 173, 106, 198, 109, 76, 153, 250, 63, 179, 207, 170, 99, 239, 124, 22, 204, 21, 64, 238, 21, 147, 207, 135, 167, 223, 195, 138, 40, 56, 188, 102, 25, 128, 207, 227, 96, 176, 60, 199, 226, 180, 88, 93, 106, 51, 2, 25, 27, 189, 54, 6, 144, 252, 7, 190, 60, 10, 218, 78, 205, 35, 183, 227, 193, 206, 133, 32, 197, 240, 215, 231, 125, 141, 22, 239, 186, 195, 198, 252, 167, 178, 88, 128, 141, 205, 117, 128, 26, 72, 136, 195, 55, 191, 50, 46, 245, 39, 141, 76, 196, 43, 249, 117, 117, 158, 161, 119, 54, 190, 101, 230, 95, 8, 186, 183, 229, 129, 216, 197, 70, 134, 88, 81, 24, 98, 32, 135, 125, 102, 72, 125, 17, 115, 185, 167, 17, 94, 28, 196, 171, 124, 62, 9, 202, 28, 72];

        struct Guard {};
        impl DaaSSecurityGaurd for Guard{};
        let guard = Guard {};

        match guard.decrypt_data(priv_pem, encrypted_data.to_vec(), padding){
            Ok(msg) => {
                debug!("Sent: {}", String::from_utf8(message_sent.to_vec()).unwrap());
                debug!("Received: {}", String::from_utf8(msg.to_vec()).unwrap());
                assert_eq!(message_sent, msg);
            },
            Err(err) => {
                debug!("{:?}", err);
                assert!(false);
            },
        }
    }

    #[test]
    fn test_decrypt_bad_key() {
        let priv_pem = get_priv_pem();
        let pub_pem = generate_keypair().unwrap().1;
        let padding = Padding::PKCS1;
        let message_sent: Vec<u8> = String::from("_test123!# ").into_bytes();   

        struct Guard {};
        impl DaaSSecurityGaurd for Guard{};
        let guard = Guard {};

        let encrypted_data = guard.encrypt_data(pub_pem, message_sent.clone(), padding).unwrap();

        match guard.decrypt_data(priv_pem, encrypted_data.to_vec(), padding){
            Ok(_msg) => {
                assert!(false);
            },
            Err(_err) => {
                assert!(true);
            },
        }
    }

    #[test]
    fn test_encrypt_decrypt_mp3() {
        let priv_pem = get_priv_pem();
        let pub_pem = get_pub_pem();
        let padding = Padding::NONE;

        let mut f = File::open("./tests/example_audio_clip.mp3").unwrap();
        let mut mp3 = Vec::new();
        f.read_to_end(&mut mp3).unwrap();

        struct Guard {};
        impl DaaSSecurityGaurd for Guard{};
        let guard = Guard {};

        let encrypted_data = guard.encrypt_data(pub_pem, mp3.clone(), padding).unwrap();

        match guard.decrypt_data(priv_pem, encrypted_data.to_vec(), padding){
            Ok(msg) => {
                assert_eq!(mp3, msg);
            },
            Err(_err) => {
                assert!(false);
            },
        }
    }    

    #[test]
    fn test_encrypt_decrypt_no_padding() {
        let priv_pem = get_priv_pem();
        let pub_pem = get_pub_pem();
        let message_sent: Vec<u8> = String::from("_test123!# ").into_bytes();   
        let padding = Padding::NONE;

        struct Guard {};
        impl DaaSSecurityGaurd for Guard{};
        let guard = Guard {};

        let encrypted_data = guard.encrypt_data(pub_pem, message_sent.clone(), padding).unwrap();

        match guard.decrypt_data(priv_pem, encrypted_data.to_vec(), padding){
            Ok(msg) => {
                assert_eq!(message_sent, msg);
            },
            Err(_err) => {
                assert!(false);
            },
        }
    }
}
