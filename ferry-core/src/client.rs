pub struct Client{

}

impl Client{
    pub fn new()->Client{
        Client{}
    }

    pub fn connect(self, ip_address:&str, port: u16, name: Option<&str>){
        println!("Will connect at some point to {ip_address}:{port}")
    }
}
