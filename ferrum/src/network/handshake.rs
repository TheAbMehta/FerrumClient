use super::ConnectionError;

pub async fn perform_handshake(
    server_address: String,
    port: u16,
) -> Result<(), ConnectionError> {
    Ok(())
}
