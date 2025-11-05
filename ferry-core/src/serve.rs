use crate::discovery::register_for_discovery;
use crate::utils;
use std::net::TcpListener;
use std::path::Path;

pub fn serve(ip: &str, port: &u16, dir: &Path, name: Option<&str>) -> anyhow::Result<()> {
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

    let listener = TcpListener::bind((ip, *port))?;
    listener.set_nonblocking(false)?; // TODO: We will deal with non blocking and multi connection later
    listener.accept()?;
    Ok(())
}
