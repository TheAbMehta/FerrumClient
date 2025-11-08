use super::{connect_and_play, ConnectionError};
use std::net::SocketAddr;

pub async fn perform_login(address: SocketAddr) -> Result<(), ConnectionError> {
    let address_str = address.to_string();
    connect_and_play(address_str).await.map(|_| ())
}
