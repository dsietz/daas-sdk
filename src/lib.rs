
extern crate env_logger;
extern crate futures;
extern crate log;
extern crate pbd;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;
extern crate rand;
extern crate openssl;
extern crate actix_web;

use log::*;
use std::env;
use std::time::{SystemTime};

pub const DELIMITER: &'static str = "~";

#[macro_use] pub mod macros;
pub mod errors;
pub mod config;
pub mod doc;
pub mod eventing;
pub mod service;
pub mod storage;