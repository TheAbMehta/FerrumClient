use crate::title_screen::GameState;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayNightCycle>()
            .add_systems(OnEnter(GameState::InGame), setup_sky)
            .add_systems(
                Update,
                (
                    update_day_night,
                    update_sky_color,
                    update_directional_light,
                    update_ambient_light,
                    update_fog,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

/// Day/night cycle resource tracking time and celestial state
#[derive(Resource)]
pub struct DayNightCycle {
    /// Current time in Minecraft ticks (0.0 to 24000.0)
    /// 0 = sunrise (6:00 AM), 6000 = noon, 12000 = sunset, 18000 = midnight
    pub time: f32,

    /// Ticks per second (default: 20.0 = 20 minute full cycle like vanilla)
    pub speed: f32,

    /// Sun angle in radians (0 to TAU)
    pub sun_angle: f32,

    /// Ambient light intensity (0.0 to 1.0)
    pub ambient_light: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time: 0.0,   // Start at sunrise
            speed: 20.0, // 20 ticks/sec = 20 minute cycle
            sun_angle: 0.0,
            ambient_light: 0.5,
        }
    }
}

/// Marker component for the sky camera to apply fog
#[derive(Component)]
pub struct SkyCamera;

fn setup_sky(mut commands: Commands) {
    // Initialize clear color to day sky
    commands.insert_resource(ClearColor(Color::srgb(0.47, 0.65, 1.0)));

    // Initialize ambient light as a component on an entity
    commands.spawn(AmbientLight {
        color: Color::srgb(1.0, 1.0, 1.0),
        brightness: 500.0,
        affects_lightmapped_meshes: false,
    });
}

/// Update day/night cycle time and derived values
fn update_day_night(time: Res<Time>, mut cycle: ResMut<DayNightCycle>) {
    // Advance time
    cycle.time += cycle.speed * time.delta_secs();
    if cycle.time >= 24000.0 {
        cycle.time -= 24000.0;
    }

    // Calculate sun angle (0 at sunrise, PI at sunset)
    cycle.sun_angle = (cycle.time / 24000.0) * std::f32::consts::TAU;

    // Calculate ambient light intensity
    // Brightest at noon (6000), darkest at midnight (18000)
    let time_normalized = cycle.time / 24000.0;
    let sun_height = (time_normalized * std::f32::consts::TAU).sin();

    // Smooth curve: 1.0 at noon, 0.15 at midnight
    cycle.ambient_light = (sun_height * 0.425 + 0.575).max(0.15);
}

/// Update sky background color based on time of day
fn update_sky_color(cycle: Res<DayNightCycle>, mut clear_color: ResMut<ClearColor>) {
    let time = cycle.time;

    // Define color palette for different times
    let day_color = Color::srgb(0.47, 0.65, 1.0); // Light blue
    let sunset_color = Color::srgb(0.9, 0.5, 0.2); // Orange
    let night_color = Color::srgb(0.01, 0.01, 0.05); // Dark blue-black
    let sunrise_color = Color::srgb(0.95, 0.6, 0.3); // Warm orange

    let color = if time < 1000.0 {
        // Early sunrise (0-1000): night -> sunrise
        let t = time / 1000.0;
        lerp_color(night_color, sunrise_color, smoothstep(t))
    } else if time < 3000.0 {
        // Late sunrise (1000-3000): sunrise -> day
        let t = (time - 1000.0) / 2000.0;
        lerp_color(sunrise_color, day_color, smoothstep(t))
    } else if time < 9000.0 {
        // Day (3000-9000): full day color
        day_color
    } else if time < 11000.0 {
        // Early sunset (9000-11000): day -> sunset
        let t = (time - 9000.0) / 2000.0;
        lerp_color(day_color, sunset_color, smoothstep(t))
    } else if time < 13000.0 {
        // Late sunset (11000-13000): sunset -> night
        let t = (time - 11000.0) / 2000.0;
        lerp_color(sunset_color, night_color, smoothstep(t))
    } else {
        // Night (13000-24000): full night color
        night_color
    };

    clear_color.0 = color;
}

/// Update directional light (sun/moon) based on time
fn update_directional_light(
    cycle: Res<DayNightCycle>,
    mut light_query: Query<(&mut DirectionalLight, &mut Transform)>,
) {
    for (mut light, mut transform) in light_query.iter_mut() {
        // Rotate light around X axis to simulate sun movement
        let angle = cycle.sun_angle - std::f32::consts::FRAC_PI_2;
        transform.rotation = Quat::from_rotation_x(angle);

        // Adjust light intensity based on time
        let time = cycle.time;
        let sun_height = (cycle.sun_angle).sin();

        if sun_height > 0.0 {
            // Daytime: bright warm light
            light.illuminance = 10000.0 + sun_height * 5000.0;

            // Warm color at sunrise/sunset, white at noon
            let sunset_factor = if time < 3000.0 || time > 9000.0 {
                1.0 - ((time - 6000.0).abs() - 3000.0).max(0.0) / 3000.0
            } else {
                0.0
            };

            light.color = Color::srgb(1.0, 0.95 + sunset_factor * 0.05, 0.9 + sunset_factor * 0.1);
        } else {
            // Nighttime: dim cool moonlight
            light.illuminance = 500.0 + sun_height.abs() * 500.0;
            light.color = Color::srgb(0.7, 0.8, 1.0); // Cool blue moonlight
        }
    }
}

/// Update ambient light based on time
fn update_ambient_light(cycle: Res<DayNightCycle>, mut ambient_query: Query<&mut AmbientLight>) {
    for mut ambient in ambient_query.iter_mut() {
        // Brightness: 200 at night, 1000 at day
        ambient.brightness = 200.0 + cycle.ambient_light * 800.0;

        // Color: slightly warm during day, neutral at night
        let warmth = (cycle.ambient_light - 0.15) / 0.85; // 0 at night, 1 at day
        ambient.color = Color::srgb(0.9 + warmth * 0.1, 0.85 + warmth * 0.15, 0.8 + warmth * 0.2);
    }
}

/// Update fog based on time and sky color
fn update_fog(
    cycle: Res<DayNightCycle>,
    clear_color: Res<ClearColor>,
    mut camera_query: Query<&mut DistanceFog, With<Camera3d>>,
) {
    for mut fog in camera_query.iter_mut() {
        // Match fog color to sky
        fog.color = clear_color.0;

        // Fog distance: assume 16 chunk render distance (16 * 16 = 256 blocks)
        // Start fog at 80% of render distance, end at 100%
        let render_distance = 256.0;
        fog.falloff = FogFalloff::Linear {
            start: render_distance * 0.8,
            end: render_distance,
        };

        // Directional light fog color matches sun/moon color
        let time = cycle.time;
        let sun_height = (cycle.sun_angle).sin();

        if sun_height > 0.0 {
            // Day: warm directional fog
            let sunset_factor = if time < 3000.0 || time > 9000.0 {
                1.0 - ((time - 6000.0).abs() - 3000.0).max(0.0) / 3000.0
            } else {
                0.0
            };

            fog.directional_light_color =
                Color::srgb(1.0, 0.95 + sunset_factor * 0.05, 0.9 + sunset_factor * 0.1);
            fog.directional_light_exponent = 8.0;
        } else {
            // Night: cool directional fog
            fog.directional_light_color = Color::srgb(0.7, 0.8, 1.0);
            fog.directional_light_exponent = 4.0;
        }
    }
}

/// Linear interpolation between two colors
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a_rgb = a.to_srgba();
    let b_rgb = b.to_srgba();

    Color::srgba(
        a_rgb.red + (b_rgb.red - a_rgb.red) * t,
        a_rgb.green + (b_rgb.green - a_rgb.green) * t,
        a_rgb.blue + (b_rgb.blue - a_rgb.blue) * t,
        a_rgb.alpha + (b_rgb.alpha - a_rgb.alpha) * t,
    )
}

/// Smoothstep interpolation for smoother transitions
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
