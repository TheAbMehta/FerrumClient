use std::net::SocketAddr;

#[tokio::test]
async fn test_handshake_resolves_address() {
    let result = ferrum::network::handshake::perform_handshake(
        "127.0.0.1".to_string(),
        25565,
    ).await;

    assert!(result.is_ok());
    let addr = result.unwrap();
    assert_eq!(addr.port(), 25565);
}

// Note: MinecraftConnection is not exposed in the public API
// This test is commented out until the type is made public
// #[tokio::test]
// async fn test_connection_creation() {
//     let addr: SocketAddr = "127.0.0.1:25565".parse().unwrap();
//     let _conn = ferrum::network::connection::MinecraftConnection::new(addr);
// }
