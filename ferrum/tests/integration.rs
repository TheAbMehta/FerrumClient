use bevy::prelude::*;
use ferrum_config::{Config, ConfigPlugin};

#[test]
fn test_config_loads_with_bevy_app() {
    // Create a minimal Bevy app with ConfigPlugin
    let mut app = App::new();

    // Add minimal plugins (no rendering needed for test)
    app.add_plugins(MinimalPlugins);

    // Add ConfigPlugin with default path
    app.add_plugins(ConfigPlugin {
        config_path: "config.toml".into(),
    });

    // Run one update cycle to initialize systems
    app.update();

    // Verify Config resource exists
    let config = app.world().get_resource::<Config>();
    assert!(config.is_some(), "Config resource should be loaded");

    // Verify default values are present
    let config = config.unwrap();
    assert!(
        config.client.render_distance > 0,
        "Render distance should be > 0"
    );
    assert!(
        config.client.fov >= 30.0 && config.client.fov <= 120.0,
        "FOV should be valid"
    );
}

#[test]
fn test_config_defaults_when_file_missing() {
    // Create app with non-existent config path
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(ConfigPlugin {
        config_path: "/nonexistent/config.toml".into(),
    });

    app.update();

    // Should still have Config resource with defaults
    let config = app.world().get_resource::<Config>();
    assert!(
        config.is_some(),
        "Config should use defaults when file missing"
    );

    let config = config.unwrap();
    assert_eq!(
        config.client.render_distance, 16,
        "Should use default render distance"
    );
    assert_eq!(config.client.fov, 70.0, "Should use default FOV");
    assert_eq!(
        config.server.auto_start, false,
        "Should use default auto_start"
    );
}
