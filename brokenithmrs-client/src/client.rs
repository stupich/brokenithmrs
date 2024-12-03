use std::net::UdpSocket;

pub fn send_keys(ip: String, key: char, stat: bool) {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind to address");
    if stat {
        socket.send_to(
            format!("{} true", key.to_string()).as_bytes(),
            format!("{}:6969", ip),
        );
        return;
    } else {
        socket.send_to(
            format!("{} false", key.to_string()).as_bytes(),
            format!("{}:6969", ip),
        );
        return;
    }
}
