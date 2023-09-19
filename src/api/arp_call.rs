use crate::{errors::Result, settings::SETTINGS};
use actix_web::{post, web};
use pnet::datalink::{channel, interfaces, Channel, MacAddr, NetworkInterface};
use std::net::{IpAddr, Ipv4Addr};

use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, ArpPacket, MutableArpPacket};
use pnet::packet::ethernet::EtherTypes;
use pnet::packet::ethernet::MutableEthernetPacket;
use pnet::packet::{MutablePacket, Packet};
use serde::{Deserialize, Serialize};

struct ArpCall {
  interface: NetworkInterface,
  target_ip: Ipv4Addr,
}

impl ArpCall {
  fn new(interface: NetworkInterface, target_ip: Ipv4Addr) -> Self {
    Self {
      interface,
      target_ip,
    }
  }

  fn get_mac(&mut self) -> Result<MacAddr, String> {
    let source_ip = self
      .interface
      .ips
      .iter()
      .find(|ip| ip.is_ipv4())
      .map(|ip| match ip.ip() {
        IpAddr::V4(ip) => ip,
        _ => unreachable!(),
      })
      .ok_or("No IPv4 address found")?;

    let mac = self.interface.mac.ok_or("No MAC address found")?;

    if source_ip == self.target_ip {
      // FIXME: if chosen other interface, but using host ip, will causing loop
      return Ok(mac);
    }

    let (mut sender, mut receiver) = match channel(&self.interface, Default::default()) {
      Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
      _ => return Err("Unknown channel type".to_string()),
    };

    let mut ethernet_buffer = [0u8; 42];
    let mut ethernet_packet = MutableEthernetPacket::new(&mut ethernet_buffer).ok_or("Error")?;

    ethernet_packet.set_destination(MacAddr::broadcast());
    ethernet_packet.set_source(mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);

    let mut arp_buffer = [0u8; 28];
    let mut arp_packet = MutableArpPacket::new(&mut arp_buffer).ok_or("Error")?;

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(mac);
    arp_packet.set_sender_proto_addr(source_ip);
    arp_packet.set_target_hw_addr(MacAddr::zero());
    arp_packet.set_target_proto_addr(self.target_ip);

    ethernet_packet.set_payload(arp_packet.packet_mut());

    sender.send_to(ethernet_packet.packet(), None);

    println!("Sent ARP request");

    loop {
      let buf = receiver.next().map_err(|e| format!("error {}", e))?;
      let arp_packet =
        ArpPacket::new(&buf[MutableEthernetPacket::minimum_packet_size()..]).ok_or("Error")?;

      dbg!(&arp_packet);

      if arp_packet.get_sender_proto_addr() == self.target_ip
        && arp_packet.get_target_hw_addr() == mac
      {
        println!("Received reply");
        return Ok(arp_packet.get_sender_hw_addr());
      }
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct ArpCallRequest {
  ip: String,
}

#[post("/arp")]
async fn arp(data: web::Json<ArpCallRequest>) -> Result<String> {
  // ping
  surge_ping::ping(IpAddr::V4(data.ip.parse()?), &[0; 8]).await?;

  let interface = &SETTINGS
    .read()?
    .chosen_interface
    .clone()
    .ok_or("No interface chosen")?;

  let interface = interfaces()
    .into_iter()
    .find(|iface| iface.name.eq(interface))
    .ok_or("No interface found")?;

  let target_ip = data.ip.parse()?;

  let addr = ArpCall::new(interface, target_ip).get_mac()?;

  Ok(addr.to_string())
}
