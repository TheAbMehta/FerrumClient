mod network;

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
        .add_systems(Startup, (auto_start_pumpkin, connect_to_server).chain())
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

fn connect_to_server(config: Res<Config>) {
    info!("Connecting to Minecraft server at {}...", config.server.address);
    
    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            let addr = match network::perform_handshake(
                config.server.address.clone(),
                25565,
            ).await {
                Ok(addr) => {
                    info!("Handshake successful, resolved address: {}", addr);
                    addr
                }
                Err(e) => {
                    warn!("Handshake failed: {}. Skipping server connection.", e);
                    return;
                }
            };
            
            match network::perform_login(addr).await {
                Ok(_) => {
                    info!("Successfully connected and logged in to server!");
                }
                Err(e) => {
                    warn!("Login failed: {}. Continuing without server connection.", e);
                }
            }
        });
}
