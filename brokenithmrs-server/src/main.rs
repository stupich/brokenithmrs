use core::str;
use local_ip_address;
use std::{collections::HashMap, net::UdpSocket};

struct KeysHeld {
    keys: HashMap<String, bool>,
}
impl KeysHeld {
    fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }
}
fn main() -> std::io::Result<()> {
    {
        let ip = local_ip_address::local_ip();

        let socket = UdpSocket::bind(format!("{}:6969", ip.as_ref().unwrap()))?;

        println!("Listening on {:?}:6969", ip.unwrap().clone());
        let mut buf = [0; 15];
        let mut enigo = enigo::Enigo::new(&enigo::Settings::default()).unwrap();
        let mut keys = KeysHeld::new();
        loop {
            if let Ok(received) = socket.recv(&mut buf) {
                let data = &buf[..received];
                let chars = str::from_utf8(data).unwrap();
                let split: Vec<&str> = chars.split(" ").collect();
                if (split[1] == "true") {
                    let buf: Vec<char> = split[0].to_string().chars().collect();
                    winput::press(buf[0]);
                } else if split[1] == "false" {
                    let buf: Vec<char> = split[0].to_string().chars().collect();
                    winput::release(buf[0]);
                }
            };
        }
    }
}
