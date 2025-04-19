use std::net::SocketAddr;

#[path = "../src/network/mod.rs"]
mod network;

#[tokio::test]
async fn test_handshake_resolves_address() {
    let result = network::perform_handshake(
        "127.0.0.1".to_string(),
        25565,
    ).await;
    
    assert!(result.is_ok());
    let addr = result.unwrap();
    assert_eq!(addr.port(), 25565);
}

#[tokio::test]
async fn test_connection_creation() {
    let addr: SocketAddr = "127.0.0.1:25565".parse().unwrap();
    let _conn = network::MinecraftConnection::new(addr);
}
