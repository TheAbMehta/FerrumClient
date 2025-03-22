use ferrum_protocol::{ConnectionState, ConnectionStateError};

#[test]
fn test_connection_state_initial() {
    let state = ConnectionState::new();
    assert_eq!(state.current(), ferrum_protocol::ProtocolState::Handshake);
}

#[test]
fn test_connection_state_handshake_to_login() {
    let mut state = ConnectionState::new();
    state.transition_to_login().unwrap();
    assert_eq!(state.current(), ferrum_protocol::ProtocolState::Login);
}

#[test]
fn test_connection_state_handshake_to_status() {
    let mut state = ConnectionState::new();
    state.transition_to_status().unwrap();
    assert_eq!(state.current(), ferrum_protocol::ProtocolState::Status);
}

#[test]
fn test_connection_state_login_to_config() {
    let mut state = ConnectionState::new();
    state.transition_to_login().unwrap();
    state.transition_to_config().unwrap();
    assert_eq!(state.current(), ferrum_protocol::ProtocolState::Config);
}

#[test]
fn test_connection_state_config_to_play() {
    let mut state = ConnectionState::new();
    state.transition_to_login().unwrap();
    state.transition_to_config().unwrap();
    state.transition_to_play().unwrap();
    assert_eq!(state.current(), ferrum_protocol::ProtocolState::Play);
}

#[test]
fn test_connection_state_invalid_transition_handshake_to_config() {
    let mut state = ConnectionState::new();
    let result = state.transition_to_config();
    assert!(result.is_err());
    match result {
        Err(ConnectionStateError::InvalidTransition { from, to }) => {
            assert_eq!(from, ferrum_protocol::ProtocolState::Handshake);
            assert_eq!(to, ferrum_protocol::ProtocolState::Config);
        }
        _ => panic!("Expected InvalidTransition error"),
    }
}

#[test]
fn test_connection_state_invalid_transition_handshake_to_play() {
    let mut state = ConnectionState::new();
    let result = state.transition_to_play();
    assert!(result.is_err());
}

#[test]
fn test_connection_state_invalid_transition_login_to_play() {
    let mut state = ConnectionState::new();
    state.transition_to_login().unwrap();
    let result = state.transition_to_play();
    assert!(result.is_err());
}

#[test]
fn test_connection_state_invalid_transition_status_to_login() {
    let mut state = ConnectionState::new();
    state.transition_to_status().unwrap();
    let result = state.transition_to_login();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_packet_type_aliases_exist() {
    // This test verifies that type aliases compile and are accessible
    // We don't need to instantiate them, just verify they exist
    let _: Option<ferrum_protocol::ChunkDataPacket> = None;
    let _: Option<ferrum_protocol::LoginPacket> = None;
    let _: Option<ferrum_protocol::ConfigPacket> = None;
    let _: Option<ferrum_protocol::GamePacket> = None;
}
