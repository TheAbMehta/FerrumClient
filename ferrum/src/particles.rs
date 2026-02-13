use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

/// Plugin for particle effects system
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_particles, cleanup_dead_particles));
    }
}

/// Component marking an entity as a particle
#[derive(Component)]
pub struct Particle {
    pub lifetime: Timer,
    pub velocity: Vec3,
    pub gravity: f32,
}

impl Particle {
    pub fn new(lifetime_secs: f32, velocity: Vec3, gravity: f32) -> Self {
        Self {
            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
            velocity,
            gravity,
        }
    }
}

/// Spawn block break particles at a given position
pub fn spawn_block_break_particles(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    block_color: Color,
    count: usize,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        // Random velocity for each particle
        let velocity = Vec3::new(
            rng.gen_range(-2.0..2.0),
            rng.gen_range(1.0..4.0),
            rng.gen_range(-2.0..2.0),
        );

        // Small random offset from break position
        let offset = Vec3::new(
            rng.gen_range(-0.3..0.3),
            rng.gen_range(-0.3..0.3),
            rng.gen_range(-0.3..0.3),
        );

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: block_color,
                unlit: false,
                ..default()
            })),
            Transform::from_translation(position + offset),
            Particle::new(0.5, velocity, 9.8),
        ));
    }
}

/// Spawn generic particles (explosions, effects, etc.)
pub fn spawn_particle_burst(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    color: Color,
    count: usize,
    speed: f32,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..count {
        let velocity = Vec3::new(
            rng.gen_range(-speed..speed),
            rng.gen_range(-speed..speed),
            rng.gen_range(-speed..speed),
        );

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(position),
            Particle::new(1.0, velocity, 5.0),
        ));
    }
}

/// System that updates particle positions and lifetimes
fn update_particles(time: Res<Time>, mut particles: Query<(&mut Transform, &mut Particle)>) {
    let delta = time.delta_secs();

    for (mut transform, mut particle) in &mut particles {
        // Update lifetime
        particle.lifetime.tick(time.delta());

        // Apply velocity
        transform.translation += particle.velocity * delta;

        // Apply gravity
        particle.velocity.y -= particle.gravity * delta;

        // Fade out as particle dies
        let life_ratio = particle.lifetime.fraction();
        // Particles fade faster near end of life
        if life_ratio > 0.7 {
            // Could apply alpha fading here if materials support it
        }
    }
}

/// System that removes particles whose lifetime has expired
fn cleanup_dead_particles(mut commands: Commands, particles: Query<(Entity, &Particle)>) {
    for (entity, particle) in &particles {
        if particle.lifetime.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
