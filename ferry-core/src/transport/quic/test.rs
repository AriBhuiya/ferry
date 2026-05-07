#[cfg(test)]
mod test {
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio::time::sleep;
    use crate::transport::quic::client::QuicClient;
    use crate::transport::quic::server::{generate_self_signed_cert, make_server_config, QuicServer};
    use crate::transport::{TransportClient, TransportServer};
    use crate::transport::Transport; // or the correct path to your trait

    #[tokio::test(flavor = "multi_thread")]
    async fn quic_roundtrip_real_stack() -> anyhow::Result<()> {
        let server_cfg = make_server_config().expect("Failed to make server config");

        let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let mut server = QuicServer::new(bind_addr, server_cfg);
        server.bind()?;

        let server_addr = server
            .endpoint
            .as_ref()
            .unwrap()
            .local_addr()
            .expect("server addr");

        let server_task = tokio::spawn(async move {
            let mut conn = server.accept().await?;

            let data = conn.receive_data().await?;
            conn.send_data(&data).await?;

            sleep(Duration::from_millis(50)).await;

            Ok::<(), anyhow::Error>(())
        });

        let mut client = QuicClient::new();
        let mut conn = client.connect(server_addr, "localhost").await?;

        let msg = b"hello quic";
        conn.send_data(msg).await?;
        let echoed = conn.receive_data().await?;
        println!("{}", String::from_utf8(echoed.clone())?);
        assert_eq!(echoed, msg);

        drop(conn);

        match server_task.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                let s = format!("{e}");
                if s.contains("closed by peer: 0") || s.contains("connection closed") {
                    Ok(())
                } else {
                    Err(e)
                }
            }
            Err(join_err) => Err(join_err.into()),
        }
    }










}