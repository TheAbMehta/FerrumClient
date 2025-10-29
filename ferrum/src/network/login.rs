use super::{ConnectionError, MinecraftConnection};
use std::net::SocketAddr;

pub async fn perform_login(address: SocketAddr) -> Result<(), ConnectionError> {
    let conn = MinecraftConnection::new(address);
    conn.connect().await
}
