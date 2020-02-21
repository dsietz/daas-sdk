extern crate daas;
extern crate kafka;
extern crate rusoto_core;

use std::io;
use rusoto_core::Region;
use kafka::consumer::{FetchOffset, GroupOffsetStorage};
use daas::service::processor::{DaasGenesisProcessor, DaaSGenesisProcessorService};
use daas::storage::s3::{S3BucketManager, S3BucketMngr};

fn get_bucket() -> S3BucketMngr {
    S3BucketMngr::new(Region::UsEast1, "iapp-daas-test-bucket".to_string())
}

fn main() {
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();
    let hosts = vec!("localhost:9092".to_string());
    
    let stopper = DaasGenesisProcessor::run(hosts, FetchOffset::Earliest, GroupOffsetStorage::Kafka, get_bucket());
    
    println!("Genesis processor is running ...");
    println!("Press [Enter] to stop the Genesis processor.");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_n) => {
            DaasGenesisProcessor::stop(stopper);
        }
        Err(error) => println!("error: {}", error),
    }    
}