pub fn connect(ip_address: &str, port: u16, server_name: Option<String>){
    let client = ferry_core::Client::new();
    let server_name: Option<&str> = server_name.as_deref();
    client.connect(ip_address, port, server_name)
}