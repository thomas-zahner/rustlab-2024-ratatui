use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub const DEFAULT_PORT: u16 = 42069;

#[derive(Parser)]
pub struct Args {
    #[arg(short, long, default_value_t = DEFAULT_IP)]
    ip: IpAddr,

    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

impl Args {
    pub fn parse_socket_addr() -> SocketAddr {
        let cli = Self::parse();
        SocketAddr::new(cli.ip, cli.port)
    }
}
