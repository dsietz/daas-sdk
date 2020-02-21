extern crate daas;
extern crate kafka;

use std::io;
use std::thread;
use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use daas::service::processor::{DaaSProcessor, DaaSProcessorMessage, DaaSProcessorService};
use std::sync::mpsc::{channel};


fn main() {
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();

    // configuration settings
    let hosts = vec!("localhost:9092".to_string());
    let topic = "order.clothing".to_string();

    // 
    let (tx, rx) = channel();
    let consumer = Consumer::from_hosts(hosts)
                            .with_topic(topic.clone())
                            .with_fallback_offset(FetchOffset::Earliest)
                            .with_group(format!("{}-consumer", topic.clone()))
                            .with_offset_storage(GroupOffsetStorage::Kafka)
                            .create()
                            .unwrap();

    // start the processor
    let _handler = thread::spawn(move || {
        DaaSProcessor::start_listening(consumer, &rx, None, |msg: DaaSProcessorMessage, _none_var , _t: Option<&i8>|{
            //let doc = msg.doc;
            //let _id = doc._id;
            //let order = doc.;

            println!("Order Number {} ...", msg.doc._id);
            Ok(1)
        });
    });

    println!("Clothing Orders processor is running ...");
    println!("Press [Enter] to stop the Clothing Orders processor.");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_n) => {
            DaaSProcessor::stop_listening(&tx);
        }
        Err(error) => println!("error: {}", error),
    }    
}