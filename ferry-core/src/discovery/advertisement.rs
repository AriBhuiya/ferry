use crate::discovery::FerryAnnouncement;
use anyhow::Context;
use mdns_sd::{ServiceDaemon, ServiceInfo};

pub(super) fn start_ferry_advertisement(
    instance_name: &str,  // e.g. "ferry-server-1"
    port: u16,            // e.g. 42042
    txt: &[(&str, &str)], // e.g. &[("name","ferry-server-1"), ("uuid","..."), ("ver","1.2")]
) -> anyhow::Result<FerryAnnouncement> {
    // mDNS/ DNS-SD service type
    const SERVICE_TYPE: &str = "_ferry._tcp.local.";
    let fullname = format!("{instance_name}.{SERVICE_TYPE}");

    // Spawn the mDNS service daemon (handles UDP/5353, probing, goodbyes)
    let daemon = ServiceDaemon::new().context("start mDNS daemon")?;

    // TODO: Decie if using the default hostname is a better idea
    // let mut host = hostname::get().unwrap_or_default().to_string_lossy().into_owned();
    // if !host.ends_with(".local") { host.push_str(".local"); }
    // if !host.ends_with('.') { host.push('.'); }

    let host = format!("{instance_name}.ferry.local.");
    // Build TXT properties map
    let props: std::collections::HashMap<String, String> = txt
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    // Construct the DNS-SD record set (SRV + TXT, host will be current .local)
    // `supply interface IPs or use enable_addr_auto()
    let service_info = ServiceInfo::new(
        SERVICE_TYPE,  // "_ferry._tcp.local."
        instance_name, // "FERRY-SERVER-1._ferry._tcp.local."
        &host,         // host ("" lets the library use the current hostname.local)
        "",            // domain ("" -> ".local")
        port,
        props, // TXT
    )
    .context("build ServiceInfo")?
    .enable_addr_auto();

    // Register (announces PTR → SRV/TXT; handles conflict renames like " (2)")
    daemon
        .register(service_info)
        .context("register Ferry service")?;
    Ok(FerryAnnouncement { daemon, fullname })
}
