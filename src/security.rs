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

        let padding = Padding::PKCS1;
        
        let sender = Rsa::public_key_from_pem(&pub_key).unwrap();
        let mut encrypted_data: Vec<u8> = vec![0; sender.size() as usize];
        sender.public_encrypt(&message_sent, encrypted_data.as_mut_slice(), padding).unwrap(); 

        let receiver = Rsa::private_key_from_pem(&priv_key).unwrap();
        let mut message_received: Vec<u8> = vec![0; sender.size() as usize];
        let message_size = receiver.private_decrypt(&encrypted_data, message_received.as_mut_slice(), padding).unwrap();

        //count how many control NUL characters there are
        let zero: u8 = 0;
        let mut c: usize = 0;
        let mut message_received_trim: Vec<u8> = Vec::new();
        
        for chr in message_received {
            if chr.is_ascii_control() && chr == zero {
                c = c + 1;
            } else {
                message_received_trim.push(chr);
            }
        }
        println!("There are {} zero control characters.", c);
        

        //let encrypted_data = encrypt_data_with_pubkey(&message_sent, pub_key).unwrap();
        //let message_received = decrypt_data_with_prikey(&encrypted_data, priv_key).unwrap();
        println!("Sent: {}", String::from_utf8(message_sent.to_vec()).unwrap());
        //println!("Encrypted Data: {:?}", encrypted_data);
        println!("Received: {}", String::from_utf8(message_received_trim.to_vec()).unwrap());
        assert_eq!(message_sent, message_received_trim);
    }
}
