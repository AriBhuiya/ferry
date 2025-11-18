use std::net::SocketAddr;
use crate::transport::quic::connection::QuicTransport;
use crate::transport::TransportServer;
use anyhow::{anyhow, Result};
use quinn::ServerConfig;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

pub struct QuicServer {
    bind_addr: SocketAddr,
    server_config: ServerConfig,
    pub(crate) endpoint: Option<quinn::Endpoint>,
}

impl QuicServer {
    pub fn new(bind_addr: SocketAddr, server_config: ServerConfig) -> Self {
        Self {
            bind_addr,
            server_config,
            endpoint: None,
        }
    }
}

#[async_trait::async_trait]
impl TransportServer for QuicServer {
    type Conn = QuicTransport;

    async fn listen(&mut self) -> Result<()>{
        if self.endpoint.is_some(){
            return Ok(());
        }
        let endpoint = quinn::Endpoint::server(self.server_config.clone(), self.bind_addr)?;
        self.endpoint = Some(endpoint);

        Ok(())
    }

    async fn accept(&mut self) -> Result<Self::Conn> {
        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or_else(|| anyhow!("listen() must be called before accept()"))?;
        let incoming_opt = endpoint.accept().await;
        let incoming = incoming_opt.ok_or_else(|| anyhow!("endpoint closed"))?;
        let connection = incoming.await?;
        let (send, recv) = connection.accept_bi().await?;
        Ok(QuicTransport::new(connection, send, recv))
    }

}

pub fn generate_self_signed_cert() -> Result<(CertificateDer<'static>, PrivatePkcs8KeyDer<'static>)> {
    let cert = generate_simple_self_signed(vec!["localhost".to_string()])?;
    let cert_der = CertificateDer::from(cert.cert);                 // or serialize_der(), etc.
    let key_der = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
    Ok((cert_der, key_der))
}

pub fn make_server_config() -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let (cert, key) = generate_self_signed_cert()?;
    let server_config = ServerConfig::with_single_cert(vec![cert], rustls::pki_types::PrivateKeyDer::Pkcs8(key))?;
    Ok(server_config)
}
