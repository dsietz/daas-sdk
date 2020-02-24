extern crate daas;
extern crate pbd;

use daas::doc::{DaaSDoc};
use pbd::dua::{DUA};
use pbd::dtc::{Tracker};

fn main() {
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();

    println!("Ready to build your DaaS document ...");

    let mut src_name = String::new();
    println!("What is the source name?");
    let p1 = std::io::stdin().read_line(&mut src_name).unwrap();

    let mut src_uid = String::new();
    println!("What is the source unique identifier (ID)?");
    let p2 = std::io::stdin().read_line(&mut src_uid).unwrap();

    let mut cat = String::new();
    println!("What is the category?");
    let p3 = std::io::stdin().read_line(&mut cat).unwrap();

    let mut subcat = String::new();
    println!("What is the subcategory?");
    let p4 = std::io::stdin().read_line(&mut subcat).unwrap();

    let mut auth = String::new();
    println!("Who is the author?");
    let p5 = std::io::stdin().read_line(&mut auth).unwrap();

    let mut usg_agree = String::new();
    println!("What is the name of the usage agreement, (e.g.: For Billing Purpose)?");
    let p5 = std::io::stdin().read_line(&mut usg_agree).unwrap();

    let mut usg_agree_uri = String::new();
    println!("Where is the usage agreement located, (e.g.: www.dua.org/billing.pdf)?");
    let p5 = std::io::stdin().read_line(&mut usg_agree_uri).unwrap();

    let mut duas = Vec::new();
    duas.push(DUA::new(usg_agree, usg_agree_uri, 1582559823));
    let dtc = Tracker::new(DaaSDoc::make_id(cat.clone(), subcat.clone(), src_name.clone(), src_uid.parse::<usize>().unwrap()));

    let mut data = String::new();
    println!("Enter some date ...");
    let p5 = std::io::stdin().read_line(&mut data).unwrap();

    let mut doc = DaaSDoc::new(src_name, src_uid.parse::<usize>().unwrap(), cat, subcat, auth, duas, dtc, data.as_bytes().to_vec());

    println!("{}", doc.serialize())
}