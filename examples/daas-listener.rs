extern crate actix_web;
extern crate daas;

use actix_web::{web, App, HttpServer};
use daas::service::extractor::Base64Author;
use daas::service::listener::{DaaSListener, DaaSListenerService};
use pbd::dtc::middleware::actix::*;
use pbd::dua::middleware::actix::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "warn");
    // set the environment variable for overwriting the default location for local storage
    //std::env::set_var("DAAS_LOCAL_STORAGE", "C:\\tmp");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(DUAEnforcer::default())
            .wrap(DTCEnforcer::default())
            .service(
                web::resource(&DaaSListener::get_service_health_path())
                    .route(web::get().to(DaaSListener::health)),
            )
            .service(
                web::resource(&DaaSListener::get_service_path())
                    .route(web::post().to(DaaSListener::index::<Base64Author>)),
            )
    })
    .bind("localhost:8088")
    .unwrap()
    .run()
    .await
}
