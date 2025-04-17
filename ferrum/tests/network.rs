use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_connection_to_server() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    tokio::spawn(async move {
        let (_socket, _addr) = listener.accept().await.unwrap();
    });
    
    let conn = ferrum::network::MinecraftConnection::new(addr);
    assert!(conn.connect().await.is_ok());
}

#[tokio::test]
async fn test_handshake_sequence() {
    let result = ferrum::network::perform_handshake(
        "localhost".to_string(),
        25565,
    ).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_sequence() {
    let result = ferrum::network::perform_login(
        "TestBot".to_string(),
    ).await;
    
    assert!(result.is_ok());
}
