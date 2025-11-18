use crate::transport::Transport;

pub struct QuicTransport{
    connection: quinn::Connection,
    send: quinn::SendStream,
    recv: quinn::RecvStream,
}

impl QuicTransport {
    pub fn new(connection: quinn::Connection, send: quinn::SendStream, recv: quinn::RecvStream) -> Self {
        Self { connection, send, recv }
    }
}

#[async_trait::async_trait]
impl Transport for QuicTransport{
    async fn send_data(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.send.write_all(data).await?;
        self.send.finish()?;
        Ok(())
    }

    async fn receive_data(&mut self) -> anyhow::Result<Vec<u8>> {
        let buf = self.recv.read_to_end(usize::MAX).await?;
        Ok(buf)
    }

    async fn close(&mut self) -> anyhow::Result<()> {
        self.connection.close(0u32.into(), b"");
        Ok(())
    }
}