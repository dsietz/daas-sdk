extern crate daas;
extern crate actix_web;

use daas::service::listener::{Info, DaaSListener, DaaSListenerService};
use pbd::dua::middleware::actix::*;
use actix_web::{web, App, HttpResponse, HttpServer};

fn main() {
    HttpServer::new(
        || App::new()
            .wrap(DUAEnforcer::default())
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