use bevy::prelude::*;
use ferrum_config::{Config, ConfigPlugin};
use ferrum_subprocess::PumpkinServer;
use std::path::PathBuf;

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
        .add_systems(Startup, auto_start_pumpkin)
        .run();
}

fn auto_start_pumpkin(config: Res<Config>) {
    if config.server.auto_start {
        info!("Auto-starting Pumpkin server...");
        
        let pumpkin_path = PathBuf::from("./pumpkin");
        
        tokio::runtime::Runtime::new()
            .expect("Failed to create tokio runtime")
            .block_on(async {
                let mut server = PumpkinServer::new(pumpkin_path);
                match server.start().await {
                    Ok(_) => info!("Pumpkin server started successfully"),
                    Err(e) => warn!("Failed to start Pumpkin server: {}. Continuing without local server.", e),
                }
                
                std::mem::forget(server);
            });
    }
}
