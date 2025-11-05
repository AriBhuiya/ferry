use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, ContentArrangement, Table};
use std::net::SocketAddr;
use std::time::Duration;

pub(crate) fn discover(_is_all: bool, interval: u64) -> anyhow::Result<()> {
    let mut services = ferry_core::discover_ferry_services(Duration::from_millis(interval))?;
    for service in &mut services {
        service.sort_addrs_by_preference();
    }
    // sort services by name for display
    services.sort_by(|a, b| a.instance.cmp(&b.instance));
    match _is_all {
        true => display_all(&services),
        false => display(&services),
    }
    Ok(())
}

fn display(services: &[ferry_core::FerryService]) {
    let mut table = Table::new();
    println!("Discovered {} services", services.len());
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["NAME", "HOST", "ADDRESS", "PORT"]);

    for svc in services.iter() {
        let best = svc
            .addrs
            .first()
            .map(|ip| match ip {
                SocketAddr::V4(addr) => addr.ip().to_string(),
                SocketAddr::V6(addr) => addr.ip().to_string(),
            })
            .unwrap_or_else(|| "<no addr>".to_string());

        table.add_row(vec![
            Cell::new(&svc.instance),
            Cell::new(&svc.host),
            Cell::new(best),
            Cell::new(svc.port.to_string()),
        ]);
    }
    println!("{table}");
}

fn display_all(services: &[ferry_core::FerryService]) {
    let mut table = Table::new();
    println!("Discovered {} services", services.len());
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["NAME", "HOST", "ADDRESSES", "PORT"]);

    for svc in services.iter() {
        let addrs_str = if svc.addrs.is_empty() {
            "<no addr>".to_string()
        } else {
            svc.addrs
                .iter()
                .map(|sa| match sa {
                    SocketAddr::V4(a) => a.ip().to_string(),
                    SocketAddr::V6(a) => a.ip().to_string(),
                })
                .collect::<Vec<_>>()
                .join("\n") // newline => multiline cell
        };

        table.add_row(vec![
            Cell::new(&svc.instance),
            Cell::new(&svc.host),
            Cell::new(addrs_str),
            Cell::new(svc.port.to_string()),
        ]);
    }

    println!("{table}");
}
