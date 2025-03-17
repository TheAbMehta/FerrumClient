use ferrum_config::{Config, ConfigError};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_parse_valid_config() {
    let toml_content = r#"
[client]
render_distance = 16
fov = 90.0
fps_limit = 144
vsync = true

[server]
address = "localhost:25565"
auto_start = true

[assets]
source = "mojang"
cache_dir = "~/.ferrum/cache"

[keybindings]
forward = "W"
back = "S"
left = "A"
right = "D"
jump = "Space"
sneak = "LShift"
sprint = "LControl"
inventory = "E"
drop = "Q"
chat = "T"
"#;

    let config = Config::from_str(toml_content).expect("Failed to parse valid config");

    assert_eq!(config.client.render_distance, 16);
    assert_eq!(config.client.fov, 90.0);
    assert_eq!(config.client.fps_limit, Some(144));
    assert_eq!(config.client.vsync, true);

    assert_eq!(config.server.address, "localhost:25565");
    assert_eq!(config.server.auto_start, true);

    assert_eq!(config.assets.source, "mojang");
    assert_eq!(config.assets.cache_dir, "~/.ferrum/cache");

    assert_eq!(config.keybindings.forward, "W");
    assert_eq!(config.keybindings.jump, "Space");
}

#[test]
fn test_invalid_render_distance() {
    let toml_content = r#"
[client]
render_distance = 0
fov = 90.0
"#;

    let result = Config::from_str(toml_content);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ConfigError::ValidationError(msg) => {
            assert!(msg.contains("render_distance"));
            assert!(msg.contains("greater than 0"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_invalid_fov() {
    let toml_content = r#"
[client]
render_distance = 16
fov = 200.0
"#;

    let result = Config::from_str(toml_content);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ConfigError::ValidationError(msg) => {
            assert!(msg.contains("fov"));
            assert!(msg.contains("30") || msg.contains("120"));
        }
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_missing_config_uses_defaults() {
    let toml_content = r#"
[client]
render_distance = 12
"#;

    let config = Config::from_str(toml_content).expect("Failed to parse config with defaults");

    // Should use defaults for missing fields
    assert_eq!(config.client.render_distance, 12);
    assert_eq!(config.client.fov, 70.0); // default
    assert_eq!(config.client.fps_limit, None); // default (unlimited)
    assert_eq!(config.client.vsync, false); // default

    assert_eq!(config.server.address, "localhost:25565"); // default
    assert_eq!(config.server.auto_start, false); // default
}

#[test]
fn test_load_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let toml_content = r#"
[client]
render_distance = 8
fov = 85.0
"#;

    fs::write(&config_path, toml_content).unwrap();

    let config = Config::load(&config_path).expect("Failed to load config from file");
    assert_eq!(config.client.render_distance, 8);
    assert_eq!(config.client.fov, 85.0);
}

#[test]
fn test_hot_reload_detection() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let initial_content = r#"
[client]
render_distance = 8
"#;

    fs::write(&config_path, initial_content).unwrap();

    let config = Config::load(&config_path).expect("Failed to load initial config");
    assert_eq!(config.client.render_distance, 8);

    // Simulate file change
    let updated_content = r#"
[client]
render_distance = 16
"#;

    fs::write(&config_path, updated_content).unwrap();

    // Verify we can reload
    let reloaded_config = Config::load(&config_path).expect("Failed to reload config");
    assert_eq!(reloaded_config.client.render_distance, 16);
}
