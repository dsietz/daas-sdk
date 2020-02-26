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
use json::object::{Object};
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
    let mut rspns = JsonValue::new_array();

    // headers
    let hdr0 = header_value("Content-Type", "Content-Type", get_content_type(file_path).unwrap());
    let hdr1 = header_value("Data-Usage-Agreement", "Data-Usage-Agreement", &format!("[{}]", &dua[0].serialize()));
    let hdr2 = header_value("Data-Tracker-Chain", "Data-Tracker-Chain", &base64::encode(&tracker.serialize()));
    let hdr3 = header_value("Author", "Author", &base64::encode(auth));

    header.push(hdr0);
    header.push(hdr1);
    header.push(hdr2);
    header.push(hdr3);
    
    // putting it all together
    rqst.insert("method", to_jsonvalue("POST"));
    rqst.insert("header", header);
    rqst.insert("body", gen_body(file_path));
    rqst.insert("url", gen_uri(url.clone()));

    itm.insert("name", to_jsonvalue(&url.path()));
    itm.insert("name", to_jsonvalue(&url.path().to_string()));
    itm.insert("request",rqst);
    itm.insert("response", rspns);
    item.push(itm);

    collection.insert("info", gen_info(url)); 
    collection.insert("item",item);
    collection.insert("protocolProfileBehavior", JsonValue::new_object());

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
            body.insert("mode", to_jsonvalue("raw"));
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
            body.insert("raw", String::from_utf8(data).unwrap());
            body.insert("options",json::parse(r#"{"raw":{"language":"json"}}"#).unwrap());
        
            body
        },
        false => panic!("File {} doesn't exist!", file_path),
    }
}

fn gen_info(url: Url) -> JsonValue {
    let mut info = JsonValue::new_object();

    info.insert("_postman_id", to_jsonvalue("c34879bc-68b0-4d47-a475-2e54d0f9ffe4"));
    info.insert("name", to_jsonvalue(&url.as_str()));
    info.insert("schema", to_jsonvalue("https://schema.getpostman.com/json/collection/v2.1.0/collection.json"));
    
    info
}

fn gen_uri(url: Url) -> JsonValue {
    let mut uri = JsonValue::new_object();
    uri.insert("raw", to_jsonvalue(url.as_str()));
    uri.insert("protocol", to_jsonvalue("http"));
    let mut hosts = JsonValue::new_array();
    hosts.push(url.host_str().unwrap());
    uri.insert("host", hosts);
    uri.insert("port", url.port().unwrap());
    let mut params = JsonValue::new_array();
    
    for param in url.path().to_string().split('/').collect::<Vec<&str>>().iter() {
        params.push(to_jsonvalue(param));
    }
    
    uri.insert("path", params);

    uri
}

fn get_content_type(file_path: &str) -> Option<&str> {
    Path::new(file_path)
    .extension()
    .and_then(OsStr::to_str)
}

fn header_value(key: &str, name: &str, value: &str) -> JsonValue {
    let mut hdr = JsonValue::new_object();
    hdr.insert("key", to_jsonvalue(key));
    hdr.insert("name", to_jsonvalue(name));
    hdr.insert("value", to_jsonvalue(value));
    hdr.insert("type", to_jsonvalue("text"));

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