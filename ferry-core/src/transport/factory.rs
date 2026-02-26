use std::net::SocketAddr;
use quinn::ServerConfig;
use crate::transport::quic::server::make_server_config;
use crate::transport::TransportServer;
use super::quic;
use anyhow::Result;
pub(crate) enum TransportType {
    Quic,
    TCP
}

pub fn make_quic_server(ip_address:SocketAddr, server_config: Option<ServerConfig>)-> Result<impl TransportServer>{
    let server_config = match server_config {
        Some(cfg) => cfg,
        None => make_server_config()?,
    };
    let quic_server = quic::server::QuicServer::new(ip_address, server_config);
    Ok(quic_server)
}

pub fn make_tcp_server()-> Result<()>{
    anyhow::bail!("TCP Server is not supported yet")
}

