use super::*;
use crate::errors::{BadKeyPairError};
use openssl::pkey::{Private, Public};
use openssl::rsa::{Rsa, Padding};
use std::cmp::max;

fn generate_keypair() -> Result<(Vec<u8>,Vec<u8>),BadKeyPairError>{
    let rsa = Rsa::generate(2048).unwrap();

    let priv_key: Vec<u8> = rsa.private_key_to_pem().unwrap();
    let pub_key: Vec<u8> = rsa.public_key_to_pem().unwrap();

    Ok((priv_key, pub_key))
}


#[cfg(test)]
mod tests {
    use super::*;

    fn pad_chunk_to_size(chunk: &[u8], desired_size: usize) -> Vec<u8> {
        let mut resized_vec = Vec::with_capacity(desired_size);
        for &element in chunk {
            resized_vec.push(element);
        }
        while resized_vec.len() < desired_size {
            resized_vec.push(0);
        }
        println!(
            "Desired Length = {}, Actual Length = {}",
            desired_size,
            resized_vec.len()
        );
        resized_vec
    }

    fn encrypt_data_with_pubkey(data: &[u8], pub_key: Vec<u8>) -> Result<Vec<u8>,usize> {
        let data_len = data.len();
        let public_rsa: Rsa<Public> = Rsa::public_key_from_pem(pub_key.as_slice()).unwrap();
        let buf_len = public_rsa.size() as usize;
        let mut buffer: Vec<u8> = vec![0; buf_len];
        let mut encrypted_data: Vec<u8> = Vec::with_capacity(data_len);
        println!("{}", public_rsa.size());
        for chunk in data.chunks(buf_len) {
            println!("Encrypting (len = {}): {:?}", chunk.len(), chunk);
            let chunk_mod;
            if chunk.len() < buf_len {
                chunk_mod = pad_chunk_to_size(chunk, buf_len);
            } else {
                chunk_mod = Vec::from(chunk);
            }
            let chunk_mod = chunk_mod.as_slice();
            println!("Encrypting (len = {}): {:?}", chunk_mod.len(), chunk_mod);
            let enc_len = public_rsa
                .public_encrypt(chunk_mod, buffer.as_mut_slice(), Padding::NONE)
                .expect("Error Encrypting");
            println!("Enc Data Len : {}", enc_len);
            encrypted_data.extend_from_slice(buffer.as_slice());
        }
        Ok(encrypted_data)
    }

    fn decrypt_data_with_prikey(enc_data: &[u8], priv_key: Vec<u8>) -> Result<Vec<u8>,usize> {
        let data_len = enc_data.len();
        let private_rsa: Rsa<Private> = Rsa::private_key_from_pem(priv_key.as_slice()).unwrap();
        let buf_len = private_rsa.size() as usize;
        let mut buffer: Vec<u8> = vec![0; buf_len];
        let mut decrypted_data: Vec<u8> = Vec::with_capacity (data_len);
        println!("{}", private_rsa.size());
        for chunk in enc_data.chunks(buf_len) {
            private_rsa.private_decrypt(chunk, &mut buffer, Padding::NONE).expect("Error Decrypting");;
            decrypted_data.extend_from_slice(buffer.as_slice());
        }
        Ok(decrypted_data)
    }

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
