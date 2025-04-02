use bevy::prelude::*;
use ferrum_config::ConfigPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Ferrum Client".to_string(),
                resolution: (1920, 1080).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(ConfigPlugin {
            config_path: "config.toml".into(),
        })
        .run();
}
