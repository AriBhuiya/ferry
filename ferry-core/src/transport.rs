mod tcp;
mod quic;

use anyhow;

/// Transport when there is a connection established
#[async_trait::async_trait]
pub trait Transport {
    async fn send_data(&mut self, data: &[u8]) -> anyhow::Result<()>;
    async fn receive_data(&mut self,) -> anyhow::Result<Vec<u8>>;
    async fn close(&mut self) -> anyhow::Result<()>;
}

/// Transport layer for establishing a connection
#[async_trait::async_trait]
pub trait TransportClient{
    type Conn: Transport + Send;
    async fn connect(&mut self) -> anyhow::Result<Self::Conn>;
}

/// Transport layer for listening for a connection
#[async_trait::async_trait]
pub trait TransportServer {
    type Conn: Transport;
    async fn listen(&mut self,) -> anyhow::Result<()>;
    async fn accept(&mut self,) -> anyhow::Result<Self::Conn>;
}