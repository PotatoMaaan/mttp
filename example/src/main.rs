use std::{net::SocketAddrV4, str::FromStr};

fn main() {
    mttp::start_server(std::net::SocketAddr::V4(
        SocketAddrV4::from_str("127.0.0.1:5000").unwrap(),
    ));
}
