use crate::discovery::FerryService;
use anyhow::Context;
use mdns_sd::{ScopedIp, ServiceDaemon, ServiceEvent};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

pub(crate) fn find_ferry_services(timeout: Duration) -> anyhow::Result<Vec<FerryService>> {
    const SERVICE_TYPE: &str = "_ferry._tcp.local.";

    let mdns = ServiceDaemon::new().context("start mDNS daemon")?;
    let rx = mdns.browse(SERVICE_TYPE).context("browse ferry services")?;

    let start = Instant::now();
    let mut found: Vec<FerryService> = Vec::new();

    while start.elapsed() < timeout {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                // Instance name = fullname without the ".<ty_domain>" suffix
                let instance = info
                    .get_fullname()
                    .strip_suffix(info.ty_domain.as_str())
                    .and_then(|s| s.strip_suffix('.')) // strip the dot before ty_domain
                    .unwrap_or_else(|| info.get_fullname())
                    .to_string();

                // TXT -> HashMap<String,String>
                let txt: std::collections::HashMap<String, String> = info
                    .get_properties()
                    .iter()
                    .map(|p| {
                        let key = p.key().to_string(); // <-- own the key
                        let val = p
                            .val()
                            .and_then(|v| std::str::from_utf8(v).ok())
                            .unwrap_or("")
                            .to_string();
                        (key, val)
                    })
                    .collect();

                let addrs = info.get_addresses().iter().cloned().collect::<Vec<_>>();
                let addrs = scoped_to_socket_addrs(&addrs, info.get_port());

                let svc = FerryService {
                    instance,
                    fullname: info.get_fullname().to_string(),
                    host: info.get_hostname().to_string(),
                    port: info.get_port(),
                    addrs,
                    txt,
                };
                upsert_service(&mut found, svc);
            }
            Ok(ServiceEvent::ServiceRemoved(_ty, fullname)) => {
                // Keep list tidy if something disappears
                found.retain(|s| s.fullname != fullname);
            }
            Ok(_other) => {}
            Err(_timeout) => {}
        }
    }

    let _ = mdns.shutdown();
    Ok(found)
}

fn scoped_to_socket_addrs(addrs: &[ScopedIp], port: u16) -> Vec<SocketAddr> {
    let mut out = Vec::with_capacity(addrs.len());
    for a in addrs {
        let ip = a.to_ip_addr();
        out.push(SocketAddr::new(ip, port));
    }
    out
}

fn upsert_service(found: &mut Vec<FerryService>, mut incoming: FerryService) {
    // Normalize key (DNS names are case-insensitive)
    let key = incoming.fullname.to_ascii_lowercase();

    if let Some(svc) = found
        .iter_mut()
        .find(|s| s.fullname.eq_ignore_ascii_case(&key))
    {
        // If SRV port changed, prefer the latest and reset endpoints
        if svc.port != incoming.port {
            svc.port = incoming.port;
            svc.addrs.clear();
        }

        // Keep latest host (in case of conflict renames)
        svc.host = incoming.host;

        // Merge TXT (incoming wins on key conflicts)
        for (k, v) in incoming.txt.drain() {
            svc.txt.insert(k, v);
        }

        // Merge & dedupe addresses
        let mut set: HashSet<SocketAddr> = svc.addrs.drain(..).collect();
        set.extend(incoming.addrs);
        svc.addrs = set.into_iter().collect();
    } else {
        // New service — sort its endpoints once and store
        found.push(incoming);
    }
}

// inline tests
#[cfg(test)]
mod tests {
    use crate::FerryService;
    use std::collections::{HashMap, HashSet};
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    fn v4(a: u8, b: u8, c: u8, d: u8, port: u16) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(a, b, c, d), port))
    }

    fn set_of(v: &[SocketAddr]) -> HashSet<SocketAddr> {
        v.iter().cloned().collect()
    }

    fn svc(
        fullname: &str,
        host: &str,
        port: u16,
        addrs: Vec<SocketAddr>,
        txt: &[(&str, &str)],
    ) -> FerryService {
        let mut map = HashMap::new();
        for (k, v) in txt {
            map.insert((*k).to_string(), (*v).to_string());
        }
        FerryService {
            instance: "x".into(),
            fullname: fullname.into(),
            host: host.into(),
            port,
            addrs,
            txt: map,
        }
    }

    #[test]
    fn upsert_inserts_when_not_present() {
        let mut found: Vec<FerryService> = vec![];
        let incoming = svc(
            "MySvc._tcp.local.",
            "host.local.",
            8080,
            vec![v4(192, 168, 1, 10, 8080)],
            &[("a", "1")],
        );

        super::upsert_service(&mut found, incoming.clone());

        assert_eq!(found.len(), 1);
        let s = &found[0];
        assert_eq!(s.fullname, "MySvc._tcp.local.");
        assert_eq!(s.host, "host.local.");
        assert_eq!(s.port, 8080);
        assert_eq!(set_of(&s.addrs), set_of(&[v4(192, 168, 1, 10, 8080)]));
        assert_eq!(s.txt.get("a").map(|s| s.as_str()), Some("1"));
    }

    #[test]
    fn upsert_merges_case_insensitive_without_port_change() {
        let a1 = v4(192, 168, 1, 10, 8080);
        let a2 = v4(192, 168, 1, 11, 8080);

        let mut found = vec![svc(
            "Service._tcp.LOCAL.",
            "old.local.",
            8080,
            vec![a1],
            &[("k0", "v0"), ("k1", "v1")],
        )];

        let incoming = svc(
            "service._tcp.local.",          // same name, different case
            "new.local.",                   // should overwrite host
            8080,                           // same port → do NOT clear addrs
            vec![a1, a2],                   // a1 duplicate n a2 new
            &[("k1", "NEW"), ("k2", "v2")], // k1 overwritten, k2 added
        );

        super::upsert_service(&mut found, incoming);

        assert_eq!(found.len(), 1);
        let s = &found[0];

        // host updated to latest
        assert_eq!(s.host, "new.local.");
        // port unchanged
        assert_eq!(s.port, 8080);
        // addresses are union-dedup
        assert_eq!(set_of(&s.addrs), set_of(&[a1, a2]));
        // TXT: existing retained, incoming wins on conflicts
        assert_eq!(s.txt.get("k0").map(|s| s.as_str()), Some("v0")); // retained
        assert_eq!(s.txt.get("k1").map(|s| s.as_str()), Some("NEW")); // overwritten
        assert_eq!(s.txt.get("k2").map(|s| s.as_str()), Some("v2")); // added
        // fullname stays as the original stored one
        assert_eq!(s.fullname, "Service._tcp.LOCAL.");
    }

    #[test]
    fn upsert_resets_addrs_when_port_changes_and_keeps_only_incoming_addrs() {
        let old = v4(192, 168, 1, 10, 8000);
        let kept_new = v4(10, 0, 0, 5, 9000);

        let mut found = vec![svc(
            "Svc._tcp.local.",
            "old.local.",
            8000,
            vec![old],
            &[("k", "old")],
        )];

        let incoming = svc(
            "svc._tcp.local.", // case-insensitive same
            "new.local.",
            9000, // PORT CHANGED clear previous endpoints
            vec![kept_new],
            &[("k", "new"), ("n", "1")],
        );

        super::upsert_service(&mut found, incoming);

        assert_eq!(found.len(), 1);
        let s = &found[0];

        // port updated
        assert_eq!(s.port, 9000);
        // old address removed; only incoming remains with new port
        assert_eq!(set_of(&s.addrs), set_of(&[kept_new]));
        // host updated
        assert_eq!(s.host, "new.local.");
        // TXT merged with incoming winning
        assert_eq!(s.txt.get("k").map(|s| s.as_str()), Some("new"));
        assert_eq!(s.txt.get("n").map(|s| s.as_str()), Some("1"));
    }

    #[test]
    fn upsert_does_not_merge_when_fullname_differs() {
        let mut found = vec![svc(
            "alpha._tcp.local.",
            "alpha.local.",
            7000,
            vec![v4(192, 168, 0, 2, 7000)],
            &[("a", "1")],
        )];

        let incoming = svc(
            "beta._tcp.local.", // different service
            "beta.local.",
            7001,
            vec![v4(192, 168, 0, 3, 7001)],
            &[("b", "2")],
        );

        super::upsert_service(&mut found, incoming);

        assert_eq!(found.len(), 2);
        let names: HashSet<_> = found.iter().map(|s| s.fullname.as_str()).collect();
        assert!(names.contains("alpha._tcp.local."));
        assert!(names.contains("beta._tcp.local."));
    }
}
