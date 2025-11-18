use std::net::SocketAddr;
use std::sync::Arc;
use crate::transport::quic::connection::QuicTransport;
use crate::transport::TransportClient;
use anyhow::Result;
use quinn::ClientConfig;
use quinn::crypto::rustls::QuicClientConfig;
use crate::transport::quic::cert_utils::SkipServerVerification;

pub struct QuicClient {
    server_addr: SocketAddr,
    server_name: String, // for TLS? TODO: decide what to do with this
    endpoint: Option<quinn::Endpoint>,
}

impl QuicClient {
    pub fn new(server_addr: SocketAddr, server_name: &str) -> Self {
        Self {
            server_addr,
            server_name: server_name.to_string(),
            endpoint: None,
        }
    }
}

#[async_trait::async_trait]
impl TransportClient for QuicClient {
    type Conn = QuicTransport;

    async fn connect(&mut self) -> Result<Self::Conn> {
        if self.endpoint.is_none() {
            let bind_addr: SocketAddr = "[::]:0".parse().unwrap();

            let mut endpoint = quinn::Endpoint::client(bind_addr)?;
            let client_config = make_insecure_client_config()?;
            endpoint.set_default_client_config(client_config);
            self.endpoint = Some(endpoint);
        }

        let endpoint = self.endpoint.as_ref().expect("endpoint just set above");

        let connecting = endpoint.connect(self.server_addr, self.server_name.as_str())?;
        let connection = connecting.await?;

        let (send, recv) = connection.open_bi().await?;

        Ok(QuicTransport::new(connection, send, recv))
    }
}

fn make_insecure_client_config() -> std::result::Result<ClientConfig, quinn::crypto::rustls::NoInitialCipherSuite> {
    use rustls::ClientConfig as RustlsClientConfig;

    let crypto = RustlsClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let quic_crypto = QuicClientConfig::try_from(crypto)?;
    Ok(ClientConfig::new(Arc::new(quic_crypto)))
}

