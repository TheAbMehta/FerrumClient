use bevy::prelude::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("File watcher error: {0}")]
    WatcherError(#[from] notify::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct Config {
    #[serde(default)]
    pub client: ClientConfig,

    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub assets: AssetsConfig,

    #[serde(default)]
    pub keybindings: Keybindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(default = "default_render_distance")]
    pub render_distance: u32,

    #[serde(default = "default_fov")]
    pub fov: f32,

    #[serde(default)]
    pub fps_limit: Option<u32>,

    #[serde(default)]
    pub vsync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_address")]
    pub address: String,

    #[serde(default)]
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsConfig {
    #[serde(default = "default_asset_source")]
    pub source: String,

    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    #[serde(default = "default_forward")]
    pub forward: String,

    #[serde(default = "default_back")]
    pub back: String,

    #[serde(default = "default_left")]
    pub left: String,

    #[serde(default = "default_right")]
    pub right: String,

    #[serde(default = "default_jump")]
    pub jump: String,

    #[serde(default = "default_sneak")]
    pub sneak: String,

    #[serde(default = "default_sprint")]
    pub sprint: String,

    #[serde(default = "default_inventory")]
    pub inventory: String,

    #[serde(default = "default_drop")]
    pub drop: String,

    #[serde(default = "default_chat")]
    pub chat: String,
}

fn default_render_distance() -> u32 {
    16
}
fn default_fov() -> f32 {
    70.0
}
fn default_server_address() -> String {
    "localhost:25565".to_string()
}
fn default_asset_source() -> String {
    "mojang".to_string()
}
fn default_cache_dir() -> String {
    "~/.ferrum/cache".to_string()
}
fn default_forward() -> String {
    "W".to_string()
}
fn default_back() -> String {
    "S".to_string()
}
fn default_left() -> String {
    "A".to_string()
}
fn default_right() -> String {
    "D".to_string()
}
fn default_jump() -> String {
    "Space".to_string()
}
fn default_sneak() -> String {
    "LShift".to_string()
}
fn default_sprint() -> String {
    "LControl".to_string()
}
fn default_inventory() -> String {
    "E".to_string()
}
fn default_drop() -> String {
    "Q".to_string()
}
fn default_chat() -> String {
    "T".to_string()
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            render_distance: default_render_distance(),
            fov: default_fov(),
            fps_limit: None,
            vsync: false,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: default_server_address(),
            auto_start: false,
        }
    }
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            source: default_asset_source(),
            cache_dir: default_cache_dir(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            forward: default_forward(),
            back: default_back(),
            left: default_left(),
            right: default_right(),
            jump: default_jump(),
            sneak: default_sneak(),
            sprint: default_sprint(),
            inventory: default_inventory(),
            drop: default_drop(),
            chat: default_chat(),
        }
    }
}

impl Config {
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.client.render_distance == 0 {
            return Err(ConfigError::ValidationError(
                "render_distance must be greater than 0".to_string(),
            ));
        }

        if self.client.fov < 30.0 || self.client.fov > 120.0 {
            return Err(ConfigError::ValidationError(
                "fov must be between 30 and 120".to_string(),
            ));
        }

        if let Some(fps) = self.client.fps_limit {
            if fps == 0 {
                return Err(ConfigError::ValidationError(
                    "fps_limit must be greater than 0 or None for unlimited".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Resource, Clone)]
pub struct ConfigWatcher {
    pub config_path: PathBuf,
    receiver: Arc<Mutex<mpsc::Receiver<notify::Result<notify::Event>>>>,
    _watcher: Arc<Mutex<RecommendedWatcher>>,
}

impl ConfigWatcher {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self, ConfigError> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

        watcher.watch(config_path.as_ref(), RecursiveMode::NonRecursive)?;

        Ok(Self {
            config_path: config_path.as_ref().to_path_buf(),
            receiver: Arc::new(Mutex::new(rx)),
            _watcher: Arc::new(Mutex::new(watcher)),
        })
    }

    pub fn check_for_changes(&self) -> Option<notify::Event> {
        self.receiver
            .lock()
            .ok()?
            .try_recv()
            .ok()
            .and_then(|r| r.ok())
    }
}

pub fn hot_reload_system(mut config: ResMut<Config>, watcher: Res<ConfigWatcher>) {
    if let Some(_event) = watcher.check_for_changes() {
        match Config::load(&watcher.config_path) {
            Ok(new_config) => {
                *config = new_config;
                info!("Config reloaded from {:?}", watcher.config_path);
            }
            Err(e) => {
                error!("Failed to reload config: {}", e);
            }
        }
    }
}

pub struct ConfigPlugin {
    pub config_path: PathBuf,
}

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let config = Config::load(&self.config_path).unwrap_or_else(|e| {
            warn!(
                "Failed to load config from {:?}: {}. Using defaults.",
                self.config_path, e
            );
            Config {
                client: ClientConfig::default(),
                server: ServerConfig::default(),
                assets: AssetsConfig::default(),
                keybindings: Keybindings::default(),
            }
        });

        app.insert_resource(config);

        if let Ok(watcher) = ConfigWatcher::new(&self.config_path) {
            app.insert_resource(watcher);
            app.add_systems(Update, hot_reload_system);
        } else {
            warn!("Failed to create config watcher. Hot reload disabled.");
        }
    }
}
