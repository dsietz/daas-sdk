#[macro_use] extern crate daas;
extern crate pbd;
extern crate url;
extern crate reqwest;
extern crate base64;
#[macro_use] extern crate json;

use std::env;
use std::time::{SystemTime};
use std::fs::{File};
use std::io::prelude::*;
use std::path::Path;
use std::ffi::OsStr;
use json::{JsonValue};
use url::Url;
use daas::doc::{DaaSDoc};
use pbd::dua::{DUA};
use pbd::dtc::{Tracker};

fn call(url: Url, auth: &str, mut dua: Vec<DUA>, tracker: Tracker, file_path: &str) -> Result<String, std::io::Error>{
    let mut collection = JsonValue::new_object();
    let mut item = JsonValue::new_array();
    let mut itm = JsonValue::new_object();
    let mut rqst = JsonValue::new_object();
    let mut header = JsonValue::new_array();
    let rspns = JsonValue::new_array();

    // headers
    let hdr0 = header_value("Content-Type", "Content-Type", get_content_type(file_path).unwrap());
    let hdr1 = header_value("Data-Usage-Agreement", "Data-Usage-Agreement", &format!("[{}]", &dua[0].serialize()));
    let hdr2 = header_value("Data-Tracker-Chain", "Data-Tracker-Chain", &base64::encode(&tracker.serialize()));
    let hdr3 = header_value("Authorization", "Author", &format!("Basic {}", &base64::encode(auth)));

    header.push(hdr0).expect("Could not push header 0!");
    header.push(hdr1).expect("Could not push header 1!");
    header.push(hdr2).expect("Could not push header 2!");
    header.push(hdr3).expect("Could not push header 3!");
    
    // putting it all together
    rqst.insert("method", to_jsonvalue("POST")).expect("Could not insert method!");
    rqst.insert("header", header).expect("Could not insert header!");
    rqst.insert("body", gen_body(file_path)).expect("Could not insert body!");
    rqst.insert("url", gen_uri(url.clone())).expect("Could not insert url!");

    itm.insert("name", to_jsonvalue(&url.path().to_string())).expect("Could not insert name!");
    itm.insert("request",rqst).expect("Could not insert request!");
    itm.insert("response", rspns).expect("Could not insert response!");
    item.push(itm).expect("Could not push itm!");

    collection.insert("info", gen_info(url)).expect("Could not set info!"); 
    collection.insert("item",item).expect("Could not set item!");
    collection.insert("protocolProfileBehavior", JsonValue::new_object()).expect("Could not set protocolProfileBehavior!");

    let file_name = format!("postman-collection-{}.json", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    let mut file = File::create(file_name.clone()).unwrap();
    
    match file.write_all(&json::stringify_pretty(collection, 5).as_bytes()){
        Ok(_k) => Ok(format!("{}/{}",env::current_dir().unwrap().as_path().to_str().unwrap(),file_name)),
        Err(err) => Err(err), 
    }
}

fn gen_body(file_path: &str) -> JsonValue {
    let path = Path::new(file_path);

    match path.exists() {
        true => {
            let mut body = JsonValue::new_object();
            body.insert("mode", to_jsonvalue("raw")).expect("Could not set mode!");
            let mut upload_file = match File::open(path) {
                Ok(f) => f,
                Err(err) => {
                    panic!("Could not open the file! Error: {}", err);
                }, 
            };
            let mut data = Vec::new();
            match upload_file.read_to_end(&mut data){
                Ok(sz) => println!("{} bytes read ...", sz),
                Err(err) => {
                    panic!("Could not read the file! Error: {}", err);
                },
            };
            body.insert("raw", String::from_utf8(data).unwrap()).expect("Could not insert raw!");
            body.insert("options",json::parse(r#"{"raw":{"language":"json"}}"#).unwrap()).expect("Could not insert options!");
        
            body
        },
        false => panic!("File {} doesn't exist!", file_path),
    }
}

fn gen_info(url: Url) -> JsonValue {
    let mut info = JsonValue::new_object();

    info.insert("_postman_id", to_jsonvalue("c34879bc-68b0-4d47-a475-2e54d0f9ffe4")).expect("Could not set _postman_id!");
    info.insert("name", to_jsonvalue(&url.as_str())).expect("Could not set name!");
    info.insert("schema", to_jsonvalue("https://schema.getpostman.com/json/collection/v2.1.0/collection.json")).expect("Could not set schema!");
    
    info
}

fn gen_uri(url: Url) -> JsonValue {
    let mut uri = JsonValue::new_object();
    uri.insert("raw", to_jsonvalue(url.as_str())).expect("Could not set raw!");
    uri.insert("protocol", to_jsonvalue("http")).expect("Could not set protocol!");
    let mut hosts = JsonValue::new_array();
    hosts.push(url.host_str().unwrap()).expect("Could not push hosts!");
    uri.insert("host", hosts).expect("Could not set host!");
    uri.insert("port", url.port().unwrap()).expect("Could not set port!");
    let mut params = JsonValue::new_array();
    
    for param in url.path().to_string().split('/').collect::<Vec<&str>>().iter() {
        params.push(to_jsonvalue(param)).expect("Could not push parameters!");
    }
    
    uri.insert("path", params).expect("Could not set path!");

    uri
}

fn get_content_type(file_path: &str) -> Option<&str> {
    match Path::new(file_path).extension().and_then(OsStr::to_str).unwrap() {
        "json" => Some("application/json"),
        "txt" => Some("text/plain"),
        "xml" => Some("text/xml"),
        "html" => Some("text/html"),
        _ => None,
    }
}

fn header_value(key: &str, name: &str, value: &str) -> JsonValue {
    let mut hdr = JsonValue::new_object();
    hdr.insert("key", to_jsonvalue(key)).expect("Could not set header key!");
    hdr.insert("name", to_jsonvalue(name)).expect("Could not set header name!");
    hdr.insert("value", to_jsonvalue(value)).expect("Could not set header value!");
    hdr.insert("type", to_jsonvalue("text")).expect("Could not set header type!");

    hdr
}

fn input_from_user(msg: &str) -> String {
    println!("{}", msg);
    
    let mut input = String::new();
    let _reply = std::io::stdin().read_line(&mut input).unwrap();

    input.trim().to_string()
}

fn to_jsonvalue( val: &str) -> JsonValue {
    JsonValue::String(val.to_string())
}

fn main() {
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();

    println!("Ready to build your DaaS document ...");

    let src_name = input_from_user("What is the source name?");
    let src_uid = input_from_user("What is the source unique identifier (ID)?");
    let uid = match src_uid.parse::<usize>() {
        Ok(id) => id,
        Err(err) => panic!("Invalid source unique identifier. {}", err),
    };
    let cat = input_from_user("What is the category?");
    let subcat = input_from_user("What is the subcategory?");
    let auth = input_from_user("Who is the author?");
    let  usg_agree = input_from_user("What is the name of the usage agreement, (e.g.: For Billing Purpose)?");
    let  usg_agree_uri = input_from_user("Where is the usage agreement located, (e.g.: https://www.dua.org/billing.pdf)?");
    let url = Url::parse(&usg_agree_uri).unwrap();
    let mut duas = Vec::new();
    duas.push(DUA::new(usg_agree.trim().to_string(), url.as_str().to_string(), 1582559823));
    let dtc = Tracker::new(DaaSDoc::make_id(cat.clone(), subcat.clone(), src_name.clone(), uid));
    let file_path = input_from_user("Enter the file path to send, (e.g.: C:\\tmp\\hello.json)");
    let uri = Url::parse(&format!("http://localhost:8088/{}/{}/{}/{}", cat, subcat, src_name, uid));

    match call(uri.unwrap(), &auth, duas, dtc, &file_path) {
        Ok(f) => println!("Postman collection ready at {}", f),
        Err(err) => panic!("{}", err),
    }
}