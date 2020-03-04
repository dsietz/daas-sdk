extern crate daas;
extern crate actix_web;

use daas::service::listener::{DaaSListener, DaaSListenerService};
use daas::service::extractor::{Base64Author};
use pbd::dua::middleware::actix::*;
use pbd::dtc::middleware::actix::*;
use actix_web::{web, App, HttpServer};

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
                web::resource(&DaaSListener::get_service_path()).route(web::post().to(DaaSListener::index::<Base64Author>))
            )
        )
    .bind("localhost:8088")
    .unwrap()
    .run()
    .unwrap();
}