// Library interface for ferrum
// This allows integration tests to access public modules

pub mod network;
pub mod player_controller;

// Re-export commonly used types
pub use player_controller::{GameMode, PlayerState};
