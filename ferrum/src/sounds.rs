use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::f32::consts::PI;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundAssets>()
            .init_resource::<FootstepTimer>()
            .init_resource::<AmbientTimer>()
            .init_resource::<LastPlayerPosition>()
            .add_systems(Startup, setup_sounds)
            .add_systems(
                Update,
                (
                    play_break_sound,
                    play_place_sound,
                    play_footstep_sound,
                    play_ambient_sound,
                ),
            );
    }
}

/// Resource holding handles to all procedurally generated sound effects
#[derive(Resource, Default)]
struct SoundAssets {
    break_sound: Handle<AudioSource>,
    place_sound: Handle<AudioSource>,
    step_sound: Handle<AudioSource>,
    ambient_sound: Handle<AudioSource>,
}

/// Timer for footstep sounds (plays every 0.4 seconds when moving)
#[derive(Resource)]
struct FootstepTimer {
    timer: Timer,
}

impl Default for FootstepTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.4, TimerMode::Repeating),
        }
    }
}

/// Timer for ambient cave sounds (plays every 30-60 seconds)
#[derive(Resource)]
struct AmbientTimer {
    timer: Timer,
}

impl Default for AmbientTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(45.0, TimerMode::Repeating),
        }
    }
}

/// Track last player position to detect movement
#[derive(Resource, Default)]
struct LastPlayerPosition {
    position: Vec3,
}

// ============================================================================
// WAV Generation
// ============================================================================

/// Generate a valid WAV file from PCM samples
/// WAV format: 44-byte header + 16-bit signed PCM mono data at 44100Hz
fn generate_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len();
    let byte_rate = sample_rate * 2; // 16-bit mono = 2 bytes per sample
    let data_size = (num_samples * 2) as u32;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity((44 + num_samples * 2) as usize);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // audio format (1 = PCM)
    wav.extend_from_slice(&1u16.to_le_bytes()); // num channels (1 = mono)
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align (2 bytes)
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // Convert f32 samples to 16-bit PCM
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let pcm_sample = (clamped * 32767.0) as i16;
        wav.extend_from_slice(&pcm_sample.to_le_bytes());
    }

    wav
}

// ============================================================================
// Sound Synthesis Functions
// ============================================================================

/// Generate block break sound: white noise with exponential decay (0.15s)
/// Simulates crunching/breaking with bandpass-like filtering
fn generate_break_sound() -> Vec<f32> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION: f32 = 0.15;
    let num_samples = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    // Simple PRNG for noise generation (xorshift)
    let mut rng_state: u32 = 0x12345678;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Exponential decay envelope
        let envelope = (-t * 8.0).exp();

        // White noise using xorshift
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 17;
        rng_state ^= rng_state << 5;
        let noise = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

        // Simple bandpass approximation (200-2000Hz range)
        // Mix noise with slightly delayed noise to create filtering effect
        let filtered_noise = if i > 2 {
            noise * 0.7 + samples[i - 2] * 0.3
        } else {
            noise
        };

        samples.push(filtered_noise * envelope * 0.5);
    }

    samples
}

/// Generate block place sound: low frequency thud (0.1s)
/// 120Hz sine wave with fast decay + subtle noise
fn generate_place_sound() -> Vec<f32> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION: f32 = 0.1;
    const FREQUENCY: f32 = 120.0;
    let num_samples = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut rng_state: u32 = 0x87654321;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Fast exponential decay
        let envelope = (-t * 15.0).exp();

        // Low frequency sine wave (thud)
        let sine = (2.0 * PI * FREQUENCY * t).sin();

        // Add subtle noise for texture
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 17;
        rng_state ^= rng_state << 5;
        let noise = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

        samples.push((sine + noise * 0.1) * envelope * 0.4);
    }

    samples
}

/// Generate footstep sound: very short soft noise burst (0.05s)
fn generate_step_sound() -> Vec<f32> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION: f32 = 0.05;
    let num_samples = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let mut rng_state: u32 = 0xABCDEF01;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Very fast decay
        let envelope = (-t * 25.0).exp();

        // Noise
        rng_state ^= rng_state << 13;
        rng_state ^= rng_state >> 17;
        rng_state ^= rng_state << 5;
        let noise = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

        samples.push(noise * envelope * 0.3);
    }

    samples
}

/// Generate ambient cave sound: 3s low frequency drone with subtle modulation
/// 60Hz base with slow 0.5Hz modulation
fn generate_ambient_cave() -> Vec<f32> {
    const SAMPLE_RATE: u32 = 44100;
    const DURATION: f32 = 3.0;
    const BASE_FREQ: f32 = 60.0;
    const MOD_FREQ: f32 = 0.5;
    const MOD_DEPTH: f32 = 10.0;
    let num_samples = (SAMPLE_RATE as f32 * DURATION) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;

        // Frequency modulation for eerie effect
        let modulation = (2.0 * PI * MOD_FREQ * t).sin() * MOD_DEPTH;
        let frequency = BASE_FREQ + modulation;

        // Low frequency drone
        let drone = (2.0 * PI * frequency * t).sin();

        // Gentle fade in/out envelope
        let fade_in = (t * 2.0).min(1.0);
        let fade_out = ((DURATION - t) * 2.0).min(1.0);
        let envelope = fade_in * fade_out;

        samples.push(drone * envelope * 0.1);
    }

    samples
}

// ============================================================================
// Systems
// ============================================================================

/// Setup system: Generate all sounds and store handles
fn setup_sounds(mut commands: Commands, mut audio_assets: ResMut<Assets<AudioSource>>) {
    info!("Generating procedural sound effects...");

    let break_samples = generate_break_sound();
    let break_wav = generate_wav(&break_samples, 44100);
    let break_source = AudioSource {
        bytes: break_wav.into(),
    };
    let break_handle = audio_assets.add(break_source);

    let place_samples = generate_place_sound();
    let place_wav = generate_wav(&place_samples, 44100);
    let place_source = AudioSource {
        bytes: place_wav.into(),
    };
    let place_handle = audio_assets.add(place_source);

    let step_samples = generate_step_sound();
    let step_wav = generate_wav(&step_samples, 44100);
    let step_source = AudioSource {
        bytes: step_wav.into(),
    };
    let step_handle = audio_assets.add(step_source);

    let ambient_samples = generate_ambient_cave();
    let ambient_wav = generate_wav(&ambient_samples, 44100);
    let ambient_source = AudioSource {
        bytes: ambient_wav.into(),
    };
    let ambient_handle = audio_assets.add(ambient_source);

    commands.insert_resource(SoundAssets {
        break_sound: break_handle,
        place_sound: place_handle,
        step_sound: step_handle,
        ambient_sound: ambient_handle,
    });

    info!("Sound effects generated successfully");
}

/// Play break sound when block break progress reaches 1.0
fn play_break_sound(
    mut commands: Commands,
    block_target: Res<crate::block_interact::BlockTarget>,
    sound_assets: Res<SoundAssets>,
) {
    if block_target.is_breaking && block_target.break_progress >= 1.0 {
        commands.spawn((
            AudioPlayer(sound_assets.break_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// Play place sound when right mouse button is clicked with a block target
fn play_place_sound(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    block_target: Res<crate::block_interact::BlockTarget>,
    sound_assets: Res<SoundAssets>,
) {
    if mouse_input.just_pressed(MouseButton::Right) {
        if block_target.targeted_block.is_some() {
            commands.spawn((
                AudioPlayer(sound_assets.place_sound.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

/// Play footstep sound when player is moving on ground
fn play_footstep_sound(
    mut commands: Commands,
    time: Res<Time>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut footstep_timer: ResMut<FootstepTimer>,
    mut last_position: ResMut<LastPlayerPosition>,
    sound_assets: Res<SoundAssets>,
) {
    let Some(camera_transform) = camera_query.iter().next() else {
        return;
    };

    let current_pos = camera_transform.translation;

    let xz_delta = Vec2::new(
        current_pos.x - last_position.position.x,
        current_pos.z - last_position.position.z,
    );
    let is_moving = xz_delta.length() > 0.1 * time.delta_secs();

    last_position.position = current_pos;

    if is_moving {
        footstep_timer.timer.tick(time.delta());

        if footstep_timer.timer.just_finished() {
            commands.spawn((
                AudioPlayer(sound_assets.step_sound.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.3)),
            ));
        }
    } else {
        footstep_timer.timer.reset();
    }
}

/// Play ambient cave sound periodically
fn play_ambient_sound(
    mut commands: Commands,
    time: Res<Time>,
    mut ambient_timer: ResMut<AmbientTimer>,
    sound_assets: Res<SoundAssets>,
) {
    ambient_timer.timer.tick(time.delta());

    if ambient_timer.timer.just_finished() {
        let next_duration = 30.0 + (time.elapsed_secs() % 30.0);
        ambient_timer
            .timer
            .set_duration(std::time::Duration::from_secs_f32(next_duration));

        commands.spawn((
            AudioPlayer(sound_assets.ambient_sound.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.15)),
        ));
    }
}
