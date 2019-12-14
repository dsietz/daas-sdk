
extern crate env_logger;
extern crate futures;
extern crate log;
extern crate pbd;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

use log::*;
use std::env;
use std::time::{SystemTime};


#[macro_use] pub mod macros;
pub mod config;
pub mod doc;
pub mod storage;