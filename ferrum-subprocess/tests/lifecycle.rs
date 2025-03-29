use std::path::PathBuf;

/// Test helper: Create a mock Pumpkin server binary that prints "Done" message
/// This allows us to test the lifecycle without requiring actual Pumpkin server
#[cfg(test)]
fn create_mock_pumpkin_binary() -> PathBuf {
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let mock_script = r#"#!/bin/bash
echo "Starting Pumpkin server..."
echo "Done (0.123s)!"
# Keep running until we receive "stop" on stdin
while read -r line; do
    if [ "$line" = "stop" ]; then
        echo "Stopping server..."
        exit 0
    fi
done
"#;

    let temp_dir = std::env::temp_dir();
    let mock_path = temp_dir.join("mock_pumpkin");
    
    let mut file = fs::File::create(&mock_path).unwrap();
    file.write_all(mock_script.as_bytes()).unwrap();
    
    // Make executable
    let mut perms = fs::metadata(&mock_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_path, perms).unwrap();
    
    mock_path
}

#[tokio::test]
async fn test_pumpkin_server_start() {
    let mock_binary = create_mock_pumpkin_binary();
    
    let mut server = ferrum_subprocess::PumpkinServer::new(mock_binary);
    let result = server.start().await;
    assert!(result.is_ok(), "Server should start successfully");
    assert!(server.is_running(), "Server should be running after start");
    
    let _ = server.kill().await;
}

#[tokio::test]
async fn test_pumpkin_server_graceful_shutdown() {
    let mock_binary = create_mock_pumpkin_binary();
    
    let mut server = ferrum_subprocess::PumpkinServer::new(mock_binary);
    server.start().await.unwrap();
    
    let result = server.stop().await;
    assert!(result.is_ok(), "Server should stop gracefully");
    assert!(!server.is_running(), "Server should not be running after stop");
}

#[tokio::test]
async fn test_pumpkin_server_force_kill() {
    let mock_script = r#"#!/bin/bash
echo "Starting Pumpkin server..."
echo "Done (0.123s)!"
while true; do
    sleep 1
done
"#;

    let temp_dir = std::env::temp_dir();
    let mock_path = temp_dir.join(format!("mock_pumpkin_stubborn_{}", std::process::id()));
    
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    
    let mut file = fs::File::create(&mock_path).unwrap();
    file.write_all(mock_script.as_bytes()).unwrap();
    drop(file);
    
    let mut perms = fs::metadata(&mock_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_path, perms).unwrap();
    
    let mut server = ferrum_subprocess::PumpkinServer::new(mock_path.clone());
    server.start().await.unwrap();
    
    let result = server.stop().await;
    assert!(result.is_ok(), "Server should be force killed after timeout");
    assert!(!server.is_running(), "Server should not be running after force kill");
    
    let _ = fs::remove_file(mock_path);
}

#[tokio::test]
async fn test_pumpkin_server_crash_detection() {
    let mock_script = r#"#!/bin/bash
echo "Starting Pumpkin server..."
exit 1
"#;

    let temp_dir = std::env::temp_dir();
    let mock_path = temp_dir.join(format!("mock_pumpkin_crash_{}", std::process::id()));
    
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    
    let mut file = fs::File::create(&mock_path).unwrap();
    file.write_all(mock_script.as_bytes()).unwrap();
    drop(file);
    
    let mut perms = fs::metadata(&mock_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_path, perms).unwrap();
    
    let mut server = ferrum_subprocess::PumpkinServer::new(mock_path.clone());
    let result = server.start().await;
    assert!(result.is_err(), "Server start should fail when process crashes");
    
    let _ = fs::remove_file(mock_path);
}

#[tokio::test]
async fn test_pumpkin_server_startup_timeout() {
    let mock_script = r#"#!/bin/bash
echo "Starting Pumpkin server..."
sleep 100
"#;

    let temp_dir = std::env::temp_dir();
    let mock_path = temp_dir.join(format!("mock_pumpkin_hang_{}", std::process::id()));
    
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    
    let mut file = fs::File::create(&mock_path).unwrap();
    file.write_all(mock_script.as_bytes()).unwrap();
    drop(file);
    
    let mut perms = fs::metadata(&mock_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&mock_path, perms).unwrap();
    
    let mut server = ferrum_subprocess::PumpkinServer::new(mock_path.clone());
    let result = server.start().await;
    assert!(result.is_err(), "Server start should timeout if Done message never appears");
    
    let _ = server.kill().await;
    let _ = fs::remove_file(mock_path);
}
