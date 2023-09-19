use crate::{
  errors::Result,
  settings::{Device, SETTINGS},
  wol,
};

use actix_web::{get, post, web, HttpResponse, Responder};
use pnet::datalink::interfaces;
use serde::{Deserialize, Serialize};
use surge_ping;

#[get("/all")]
async fn all() -> Result<impl Responder> {
  Ok(HttpResponse::Ok().json(&SETTINGS.read()?.devices))
}

#[post("/save")]
async fn save(data: web::Json<Vec<Device>>) -> Result<impl Responder> {
  let settings = &mut SETTINGS.write()?;
  settings.devices = data.clone();
  settings.save()?;

  Ok(HttpResponse::Ok().json(&settings.devices))
}

#[post("/wake")]
async fn wake(data: web::Json<wol::WakeData>) -> Result<impl Responder> {
  wol::wake(&data)?;
  Ok(HttpResponse::Ok().json(&data))
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DeviceStatus {
  Online,
  Offline,
}

#[get("/status/{ip}")]
async fn status(ip: web::Path<String>) -> Result<impl Responder> {
  let payload = [0; 8];
  let device = ip.parse()?;

  let device_status = surge_ping::ping(device, &payload)
    .await
    .map(|_| DeviceStatus::Online)
    .unwrap_or(DeviceStatus::Offline);

  Ok(HttpResponse::Ok().json(device_status))
}

#[derive(Debug, Serialize, Deserialize)]
struct Interface {
  name: String,
  mac: String,
  ips: Vec<String>,
  chosen: bool,
}

#[get("/interface")]
async fn get_interfaces() -> Result<impl Responder> {
  let chosen = &SETTINGS.read()?.chosen_interface;

  let interfaces: Vec<Interface> = interfaces()
    .into_iter()
    .filter(|iface| iface.is_up() && !iface.is_loopback())
    .map(|iface| Interface {
      name: iface.name.clone(),
      mac: iface
        .mac
        .map_or_else(|| String::from("none"), |m| m.to_string()),
      ips: iface
        .ips
        .iter()
        .filter(|i| i.is_ipv4())
        .map(|ip| ip.to_string())
        .collect(),
      chosen: chosen.as_ref().is_some_and(|c| c.eq(&iface.name)),
    })
    .filter(|bean| !bean.ips.is_empty())
    .collect();

  Ok(HttpResponse::Ok().json(interfaces))
}

#[post("/interface/{interface}")]
async fn chosen_interface(interface: web::Path<String>) -> Result<impl Responder> {
  let settings = &mut SETTINGS.write()?;
  settings.chosen_interface = Some(interface.clone());
  settings.save()?;

  Ok(HttpResponse::Ok().json(&settings.chosen_interface))
}
