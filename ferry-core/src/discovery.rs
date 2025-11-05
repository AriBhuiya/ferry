mod advertisement;
mod find_services;
mod score_ip;

use crate::discovery::advertisement::start_ferry_advertisement;
use crate::discovery::find_services::find_ferry_services;
use mdns_sd::ServiceDaemon;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

pub struct FerryAnnouncement {
    daemon: ServiceDaemon,
    fullname: String,
}

#[derive(Debug, Clone)]
pub struct FerryService {
    pub instance: String,
    pub fullname: String,
    pub host: String,
    pub port: u16,
    pub addrs: Vec<SocketAddr>,
    pub txt: HashMap<String, String>,
}

impl FerryService {
    pub fn sort_addrs_by_preference(&mut self) {
        score_ip::sort_addrs_by_preference(self);
    }

    pub fn get_best_addr(&mut self) -> Option<SocketAddr> {
        score_ip::get_best_addr(self)
    }
}

impl Drop for FerryAnnouncement {
    fn drop(&mut self) {
        println!("Dropping FerryAnnouncement for {}", &self.fullname);
        let _ = self.daemon.unregister(&self.fullname);
    }
}
pub(crate) fn register_for_discovery(
    server_name: &str,
    port: &u16,
) -> anyhow::Result<FerryAnnouncement> {
    start_ferry_advertisement(server_name, *port, &[("port", "1234")])
}

pub fn discover_ferry_services(timeout: Duration) -> anyhow::Result<Vec<FerryService>> {
    let services = find_ferry_services(timeout)?;
    log::info!("Found {} services", services.len());
    Ok(services)
}
