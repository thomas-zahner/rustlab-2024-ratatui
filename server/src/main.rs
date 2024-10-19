use std::io;

use server::room::Rooms;
use server::user::Users;
use server::{args, connection, logger};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let addr = args::Args::parse_socket_addr();
    logger::init_logger();

    let server = TcpListener::bind(addr).await?;
    tracing::info!("Listening on {addr}");

    let users = Users::new();
    let rooms = Rooms::new();
    loop {
        let (tcp, addr) = server.accept().await?;
        let mut connection = connection::Connection::new(tcp, users.clone(), rooms.clone(), addr);
        tokio::spawn(async move {
            connection.handle().await;
        });
    }
}
