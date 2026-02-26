mod tcp;

use std::net::SocketAddr;
use crate::discovery::register_for_discovery;
use crate::utils;
use std::path::Path;
use anyhow::Context;
use crate::transport;
use crate::transport::TransportServer;

pub fn serve(is_tcp_mode: &bool, ip: &str, port: &u16, dir: &Path, name: Option<&str>) -> anyhow::Result<()> {
    let name: String = name
        .map(str::to_owned)
        .unwrap_or_else(utils::name_generator::get_random_name);

    let res = register_for_discovery(&name, port);
    match &res {
        Ok(_) => {
            println!("OK");
        }
        Err(err) => {
            println!("Error: {err}")
        }
    }
    let _ann = res?;

    println!(
        "Starting a ferry Server on {ip}:{port} for {} with name {name}",
        dir.display()
    );

    let bind_address:SocketAddr = format!("{ip}:{port}").parse().with_context(|| format!("IP address not valid {ip}"))?;

    let mut transport_server = match is_tcp_mode {
        false => transport::factory::make_quic_server(bind_address, None)?,
        true => anyhow::bail!("TCP Server is not supported yet")
    };

    let rt = tokio::runtime::Runtime::new()?;
    let fut: anyhow::Result<()> = rt.block_on(async {
        transport_server.bind()?;
            transport_server.accept().await?;
            Ok(())
    });
    fut?;

    Ok(())
}


