use super::*;
use std::{thread};
use std::time::Duration;
use kafka::client::KafkaClient;
use kafka::producer::{Producer, Record, RequiredAcks};
use kafka::error::{ErrorKind, KafkaCode};

pub static KAFKA_BROKERS: &str = "localhost:9092";

pub fn produce_message<'a, 'b>(data: &'a [u8], topic: &'b str, brokers: Vec<String>) -> Result<(), kafka::error::ErrorKind> {
    let mut client = KafkaClient::new(brokers);

    let mut attempt = 0;
    loop {
        attempt += 1;
        let _ = client.load_metadata(&[topic])?;
        if client.topics().partitions(topic).map(|p| p.len()).unwrap_or(0) > 0 { // <-- HERE
            break;
        } else if attempt > 2 { // try up to 3 times
            // return some error
            return Err(ErrorKind::Kafka(KafkaCode::UnknownTopicOrPartition));
        }
        thread::sleep(Duration::from_secs(1));
    }

    let mut producer =
        Producer::from_client(client)
             .with_ack_timeout(Duration::from_secs(1))
             .with_required_acks(RequiredAcks::One)
             .create()?;

    producer.send(&Record{
        topic: topic,
        partition: -1,
        key: (),
        value: data,
    })?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[test]
    fn test_send_message() {
        match produce_message("Hello Kafka...".as_bytes(), "testTopic", vec!(KAFKA_BROKERS.to_string())) {
                Ok(_v) => {
                    assert!(true);
                },
                Err(e) => {
                    println!("Failed to send message to {}: {:?}", KAFKA_BROKERS.to_string(), e);
                    assert!(false);
                }
        }
    }
}