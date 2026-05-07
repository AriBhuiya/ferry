use std::net::SocketAddr;
use crate::transport::{factory, TransportClient};
use anyhow::Result;

pub struct Client{
    transport_client: Option<Box<dyn TransportClient<Conn=()>>>
}

impl Client {
    pub fn new(is_tcp:bool)->Client {
        if is_tcp{
            panic!("Not supported yet!")
        }
        Client{
            transport_client: None
        }
    }

    pub fn connect(mut self, ip_address:&str, port: u16, name: Option<&str>) -> Result<()>{
        let mut transport_client = factory::make_quic_client();
        let socket_addr : SocketAddr = format!("{ip_address}:{port}").parse().expect("df");
        // TODO: Need to clarify what to do with name. I am thinking should clients even have name?
        let name = name.unwrap_or("Default-Client");
        // self.transport_client = Some(Box(transport_client)); // wrong
        let rt = tokio::runtime::Runtime::new()?;
        let transport = rt.block_on(transport_client.connect(socket_addr, name))?;

        // TODO: Replace with better logging
        println!("connected to to {ip_address}:{port}");
        Ok(())
    }
}
