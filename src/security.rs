use super::*;
use crate::errors::{BadKeyPairError};
use openssl::rsa::{Rsa, Padding};

fn generate_keypair() -> Result<(Vec<u8>,Vec<u8>),BadKeyPairError>{
    let rsa = Rsa::generate(2048).unwrap();

    let priv_key: Vec<u8> = rsa.private_key_to_pem().unwrap();
    let pub_key: Vec<u8> = rsa.public_key_to_pem().unwrap();

    Ok((priv_key, pub_key))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = generate_keypair();

        assert!(keypair.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let keypair = generate_keypair().unwrap();
        let priv_key = keypair.0;
        let pub_key = keypair.1;
        let message_sent: Vec<u8> = String::from("Don't tell any one!").into_bytes();
        let mut encrypted_data: Vec<u8>  = vec![0; 512];
        let padding = Padding::PKCS1;
        
        let sender = Rsa::public_key_from_pem(&pub_key).unwrap();
        sender.public_encrypt(&message_sent,encrypted_data.as_mut_slice(), padding); 

        let receiver = Rsa::private_key_from_pem(&priv_key).unwrap();
        let message_received: &mut [u8] = &mut [];
        let message = receiver.private_decrypt(&encrypted_data, message_received, padding);

        println!("Sent: {}", String::from_utf8(message_sent.to_vec()).unwrap());
        println!("Received: {}", String::from_utf8(message_received.to_vec()).unwrap());
        //assert_eq!(message_sent, message_received);
        assert!(false);
    }
}
