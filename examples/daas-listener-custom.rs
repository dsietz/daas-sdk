#[macro_use]  extern crate daas;
extern crate actix_web;

use daas::service::listener::{DaaSListener, DaaSListenerService};
use pbd::dua::middleware::actix::*;
use pbd::dtc::middleware::actix::*;
use actix_web::{web, App, HttpServer};

/// Build our own Author Extractor
use serde::{Serialize, Deserialize};
use actix_web::{FromRequest, HttpRequest};
use daas::macros;
use daas::service::extractor::{AuthorExtractor, LocalError};
use daas::errors::MissingAuthorError;

/// Use macros to crate our structure
author_struct!(MyAuthor);

impl AuthorExtractor for MyAuthor {
    fn extract_author(&mut self, req: &HttpRequest, _payload: &mut actix_web::dev::Payload) -> Result<String, MissingAuthorError> {
        Ok("Knot, Tellin".to_string())
    }
    
    /// Use macros to write the default functions
    author_fn_get_name!();
    author_fn_new!();
    author_fn_set_name!();
}

/// Use macros to write the implmentation of the FromRequest trait
author_from_request!(MyAuthor);

fn main() {
    std::env::set_var("RUST_LOG", "warn");
    env_logger::init();
    
    HttpServer::new(
        || App::new()
            .wrap(DUAEnforcer::default())
            .wrap(DTCEnforcer::default())
            .service(
                web::resource(&DaaSListener::get_service_health_path()).route(web::get().to(DaaSListener::health))
            )
            .service(
                web::resource(&DaaSListener::get_service_path()).route(web::post().to(DaaSListener::index::<MyAuthor>))
            )
        )
    .bind("localhost:8088")
    .unwrap()
    .run()
    .unwrap();
}