mod tcp;

use std::net::SocketAddr;
use crate::discovery::register_for_discovery;
use crate::utils;
use std::path::{Path, PathBuf};
use anyhow::Context;
use crate::transport;
use crate::transport::TransportServer;

pub struct Server{
    ip: String,
    port: u16,
    name: Option<String>,
    is_tcp_mode: bool,
    dir: PathBuf
}

impl Server{
    pub fn new(is_tcp_mode: bool, ip: String, port: u16, dir: &Path, name: Option<String>)->Server{
        Server{
            ip,
            port,
            name,
            is_tcp_mode,
            dir: dir.to_path_buf()
        }
    }
    pub fn serve(mut self) -> anyhow::Result<()> {
        let name: String = self.name.clone()
            .unwrap_or_else(utils::name_generator::get_random_name);

        let res = register_for_discovery(&name, &self.port);
        let _ann = res?;

        self.name = Some(name.clone());

        println!(
            "Starting a ferry Server on {}:{} for {} with name {}",
            self.ip,
            self.port,
            self.dir.display(),
            name
        );

        let bind_address:SocketAddr = format!("{}:{}",self.ip, self.port).parse().with_context(|| format!("IP address not valid {}", self.ip))?;

        let mut transport_server = match self.is_tcp_mode {
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
}



