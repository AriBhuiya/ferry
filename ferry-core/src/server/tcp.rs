use std::path::Path;
use std::net::TcpListener;
// Mock TCP server

// TODO: Boilerplate to remove later
pub(super) fn serve(ip: &str, port: &u16, dir: &Path)->anyhow::Result<()>{
    let listener = TcpListener::bind((ip, *port))?;
    listener.set_nonblocking(false)?; // TODO: We will deal with non blocking and multi connection later
    listener.accept()?;
    Ok(())
}