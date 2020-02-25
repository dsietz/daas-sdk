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
use json::{JsonValue};
use json::object::{Object};
use url::Url;
use daas::doc::{DaaSDoc};
use pbd::dua::{DUA};
use pbd::dtc::{Tracker};

fn to_jsonvalue( val: &str) -> JsonValue {
    JsonValue::String(val.to_string())
}

fn header_value(key: &str, name: &str, value: &str) -> JsonValue {
    let mut hdr = JsonValue::new_object();
    hdr.insert("key", to_jsonvalue(key));
    hdr.insert("name", to_jsonvalue(name));
    hdr.insert("value", to_jsonvalue(value));
    hdr.insert("type", to_jsonvalue("text"));

    hdr
}

fn call(url: Url, mut dua: Vec<DUA>, tracker: Tracker, payload: Vec<u8>) -> Result<String, std::io::Error>{
    let mut collection = JsonValue::new_object();
    let mut info = JsonValue::new_object();
    let mut item = JsonValue::new_array();
    info.insert("_postman_id", to_jsonvalue("c34879bc-68b0-4d47-a475-2e54d0f9ffe4"));
    info.insert("name", to_jsonvalue(&url.as_str()));
	info.insert("schema", to_jsonvalue("https://schema.getpostman.com/json/collection/v2.1.0/collection.json"));

    let mut itm = JsonValue::new_object();
    itm.insert("name", to_jsonvalue(&url.path()));
    
    let mut rqst = JsonValue::new_object();

    // headers
    let mut header = JsonValue::new_array();
    let hdr0 = header_value("Content-Type", "Content-Type", "application/octet-stream");
    let hdr1 = header_value("Data-Usage-Agreement", "Data-Usage-Agreement", &format!("[{}]", &dua[0].serialize()));
    let hdr2 = header_value("Data-Tracker-Chain", "Data-Tracker-Chain", &base64::encode(&tracker.serialize()));

    header.push(hdr0);
    header.push(hdr1);
    header.push(hdr2);

    //body
    let mut body = JsonValue::new_object();
    //let mut options = JsonValue::new_object();
    //let mut raw = JsonValue::new_object();
    //raw.insert("language", to_jsonvalue("json"));
    //options.insert("raw", raw);
    body.insert("mode", to_jsonvalue("binary"));
    body.insert("raw", payload);
    //body.insert("options", options);

    //uri
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

    //response
    let mut rspns = JsonValue::new_array();
    
    // putting it all together
    rqst.insert("method", to_jsonvalue("POST"));
    rqst.insert("header", header);
    rqst.insert("body", body);
    rqst.insert("url", uri);

    itm.insert("name", to_jsonvalue(&url.path().to_string()));
    itm.insert("request",rqst);
    itm.insert("response", rspns);
    
    item.push(itm);
    collection.insert("info",info); 
    collection.insert("item",item);
    collection.insert("protocolProfileBehavior", JsonValue::new_object());

    let file_name = format!("postman-collection-{}.json", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    let mut file = File::create(file_name.clone()).unwrap();
    
    match file.write_all(&json::stringify_pretty(collection, 5).as_bytes()){
        Ok(_k) => Ok(format!("{}/{}",env::current_dir().unwrap().as_path().to_str().unwrap(),file_name)),
        Err(err) => Err(err), 
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();

    println!("Ready to build your DaaS document ...");

    let mut src_name = String::new();
    println!("What is the source name?");
    let _p1 = std::io::stdin().read_line(&mut src_name).unwrap();

    let mut src_uid = String::new();
    println!("What is the source unique identifier (ID)?");
    let _p2 = std::io::stdin().read_line(&mut src_uid).unwrap();
    let uid = match src_uid.trim().parse::<usize>() {
        Ok(id) => id,
        Err(err) => panic!("Invalid source unique identifier. {}", err),
    };

    let mut cat = String::new();
    println!("What is the category?");
    let _p3 = std::io::stdin().read_line(&mut cat).unwrap();

    let mut subcat = String::new();
    println!("What is the subcategory?");
    let _p4 = std::io::stdin().read_line(&mut subcat).unwrap();

    let mut auth = String::new();
    println!("Who is the author?");
    let _p5 = std::io::stdin().read_line(&mut auth).unwrap();

    let mut usg_agree = String::new();
    println!("What is the name of the usage agreement, (e.g.: For Billing Purpose)?");
    let _p6 = std::io::stdin().read_line(&mut usg_agree).unwrap();

    let mut usg_agree_uri = String::new();
    println!("Where is the usage agreement located, (e.g.: https://www.dua.org/billing.pdf)?");
    let _p7 = std::io::stdin().read_line(&mut usg_agree_uri).unwrap();
    let url = Url::parse(&usg_agree_uri).unwrap();

    let mut duas = Vec::new();
    duas.push(DUA::new(usg_agree.trim().to_string(), url.as_str().to_string(), 1582559823));

    // clean variables
    cat = cat.trim().to_string();
    subcat = subcat.trim().to_string();
    src_name = src_name.trim().to_string();

    let dtc = Tracker::new(DaaSDoc::make_id(cat.clone(), subcat.clone(), src_name.clone(), uid));

    let mut data = String::new();
    println!("Enter some date ...");
    let _p8 = std::io::stdin().read_line(&mut data).unwrap();

    let uri = Url::parse(&format!("http://localhost:8088/{}/{}/{}/{}", cat, subcat, src_name, uid));

    match call(uri.unwrap(), duas, dtc, data.as_bytes().to_vec()) {
        Ok(f) => println!("Postman collection ready at {}", f),
        Err(err) => panic!("{}", err),
    }
}