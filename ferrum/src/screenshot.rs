use bevy::prelude::*;
use std::fs;

/// Plugin for capturing and saving screenshots
pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        // Create screenshots directory if it doesn't exist
        if let Err(e) = fs::create_dir_all("screenshots") {
            warn!("Failed to create screenshots directory: {}", e);
        }

        app.add_systems(Update, capture_screenshot);
    }
}

/// System that captures screenshot when F2 is pressed
fn capture_screenshot(input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::F2) {
        // Generate filename with timestamp
        let now = chrono::Local::now();
        let filename = format!("screenshots/{}.png", now.format("%Y-%m-%d_%H.%M.%S"));

        info!("Screenshot requested: {}", filename);
        // TODO: Implement actual screenshot capture using Bevy's screenshot API
        // The API location/structure changed in Bevy 0.18, needs investigation
    }
}
