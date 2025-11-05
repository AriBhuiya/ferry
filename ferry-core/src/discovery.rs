mod advertisement;

use anyhow::Context;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use crate::discovery::advertisement::start_ferry_advertisement;
use std::thread;

pub struct FerryAnnouncement {
    daemon: ServiceDaemon,
    fullname: String,
}

impl Drop for FerryAnnouncement {
    fn drop(&mut self) {
        println!("Dropping FerryAnnouncement for {}", &self.fullname);
        let _ = self.daemon.unregister(&self.fullname);
    }
}
pub(in crate) fn register_for_discovery(server_name:&str, port:&u16) -> anyhow::Result<FerryAnnouncement> {
    let res = start_ferry_advertisement(server_name, *port, &[("port","1234")]);
    res
}