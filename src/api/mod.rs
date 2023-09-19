mod arp_call;
mod auth;
mod device;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
  cfg
    .service(
      web::scope("/device")
        .service(device::all)
        .service(device::save)
        .service(device::wake)
        .service(device::status)
        .service(device::get_interfaces)
        .service(device::chosen_interface)
        .service(arp_call::arp),
    )
    .service(web::scope("/auth").service(auth::get).service(auth::save));
}
