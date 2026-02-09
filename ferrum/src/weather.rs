use bevy::prelude::*;
use rand::Rng;

/// Plugin for weather rendering (rain, snow, thunder)
pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherState>()
            .add_systems(Update, (update_rain_particles, update_snow_particles, cleanup_weather_particles));
    }
}

/// Current weather conditions
#[derive(Resource, Clone, Copy, PartialEq)]
pub struct WeatherState {
    pub weather_type: WeatherType,
    pub intensity: f32, // 0.0 to 1.0
    pub thunder: bool,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            weather_type: WeatherType::Clear,
            intensity: 0.0,
            thunder: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum WeatherType {
    Clear,
    Rain,
    Snow,
}

/// Component marking a weather particle (rain or snow)
#[derive(Component)]
pub struct WeatherParticle {
    pub velocity: Vec3,
    pub age: f32,
    pub max_age: f32,
}

/// System that spawns and updates rain particles
fn update_rain_particles(
    weather: Res<WeatherState>,
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut rain_particles: Query<(&mut Transform, &mut WeatherParticle), Without<Camera3d>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if weather.weather_type != WeatherType::Rain {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let player_pos = camera_transform.translation;

    // Update existing rain particles
    for (mut transform, mut particle) in &mut rain_particles {
        particle.age += time.delta_secs();

        // Move particle down
        transform.translation += particle.velocity * time.delta_secs();

        // Despawn if too old or hit ground
        if particle.age > particle.max_age || transform.translation.y < 64.0 {
            particle.age = particle.max_age + 1.0; // Mark for cleanup
        }
    }

    // Spawn new rain particles around player
    let spawn_rate = (weather.intensity * 30.0) as usize; // Particles per second

    for _ in 0..spawn_rate {
        let mut rng = rand::thread_rng();

        // Random position in a circle around player
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(5.0..20.0);

        let spawn_pos = Vec3::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + rng.gen_range(10.0..30.0), // Spawn above player
            player_pos.z + angle.sin() * distance,
        );

        commands.spawn((
            Mesh3d(meshes.add(Capsule3d::new(0.02, 0.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.5, 0.5, 0.7, 0.4),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(spawn_pos),
            WeatherParticle {
                velocity: Vec3::new(0.0, -15.0, 0.0), // Fall speed
                age: 0.0,
                max_age: 3.0,
            },
        ));
    }
}

/// System that spawns and updates snow particles
fn update_snow_particles(
    weather: Res<WeatherState>,
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut snow_particles: Query<(&mut Transform, &mut WeatherParticle), Without<Camera3d>>,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if weather.weather_type != WeatherType::Snow {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let player_pos = camera_transform.translation;

    // Update existing snow particles
    for (mut transform, mut particle) in &mut snow_particles {
        particle.age += time.delta_secs();

        // Move particle down with slight horizontal drift
        transform.translation += particle.velocity * time.delta_secs();

        // Add gentle swaying motion
        let sway = (particle.age * 2.0).sin() * 0.5;
        transform.translation.x += sway * time.delta_secs();

        // Despawn if too old or hit ground
        if particle.age > particle.max_age || transform.translation.y < 64.0 {
            particle.age = particle.max_age + 1.0; // Mark for cleanup
        }
    }

    // Spawn new snow particles around player
    let spawn_rate = (weather.intensity * 20.0) as usize; // Slower than rain

    for _ in 0..spawn_rate {
        let mut rng = rand::thread_rng();

        // Random position in a circle around player
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(5.0..20.0);

        let spawn_pos = Vec3::new(
            player_pos.x + angle.cos() * distance,
            player_pos.y + rng.gen_range(10.0..30.0),
            player_pos.z + angle.sin() * distance,
        );

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 1.0),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(spawn_pos),
            WeatherParticle {
                velocity: Vec3::new(0.0, -5.0, 0.0), // Slower fall than rain
                age: 0.0,
                max_age: 4.0,
            },
        ));
    }
}

/// System that removes old weather particles
fn cleanup_weather_particles(
    mut commands: Commands,
    particles: Query<(Entity, &WeatherParticle)>,
) {
    for (entity, particle) in &particles {
        if particle.age > particle.max_age {
            commands.entity(entity).despawn();
        }
    }
}

/// Public API to change weather
impl WeatherState {
    pub fn set_rain(&mut self, intensity: f32) {
        self.weather_type = WeatherType::Rain;
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    pub fn set_snow(&mut self, intensity: f32) {
        self.weather_type = WeatherType::Snow;
        self.intensity = intensity.clamp(0.0, 1.0);
    }

    pub fn clear(&mut self) {
        self.weather_type = WeatherType::Clear;
        self.intensity = 0.0;
        self.thunder = false;
    }
}
