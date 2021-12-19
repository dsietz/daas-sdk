extern crate env_logger;
extern crate futures;
extern crate log;
extern crate pbd;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;
extern crate base64;
extern crate openssl;
extern crate rand;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate serde_json;
extern crate tokio;

use async_trait::async_trait;
use futures::future::{err, ok, Ready};
use log::*;
use std::env;
use std::time::SystemTime;

pub const DELIMITER: &'static str = "~";

#[macro_use]
pub mod macros;
pub mod doc;
pub mod errors;
pub mod eventing;
pub mod service;
pub mod storage;
