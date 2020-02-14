extern crate daas;
extern crate actix_web;

use daas::service::listener::{DaaSListener, DaaSListenerService};
use pbd::dua::middleware::actix::*;
use pbd::dtc::middleware::actix::*;
use actix_web::{web, App, HttpServer};

fn main() {
    env_logger::init();
    
    HttpServer::new(
        || App::new()
            .wrap(DUAEnforcer::default())
            .wrap(DTCEnforcer::default())
            .service(
                web::resource(&DaaSListener::get_service_health_path()).route(web::get().to(DaaSListener::health))
            )
            .service(
                web::resource(&DaaSListener::get_service_path()).route(web::post().to(DaaSListener::index))
            )
        )
    .bind("localhost:8088")
    .unwrap()
    .run()
    .unwrap();
}