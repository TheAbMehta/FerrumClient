use crate::title_screen::GameState;
use bevy::prelude::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudState>()
            .add_systems(OnEnter(GameState::InGame), setup_hud)
            .add_systems(
                Update,
                (update_debug_text, update_hotbar_selection, toggle_debug)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource)]
pub struct HudState {
    pub health: f32,
    pub hunger: f32,
    pub xp_level: u32,
    pub xp_progress: f32,
    pub selected_slot: usize,
    pub show_debug: bool,
    pub fps: f32,
    pub position: [f64; 3],
    pub chunk_count: usize,
}

impl Default for HudState {
    fn default() -> Self {
        Self {
            health: 20.0,
            hunger: 20.0,
            xp_level: 0,
            xp_progress: 0.0,
            selected_slot: 0,
            show_debug: false,
            fps: 0.0,
            position: [0.0, 0.0, 0.0],
            chunk_count: 0,
        }
    }
}

#[derive(Component)]
struct HudCamera;

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct HotbarSlot(usize);

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct HungerBar;

#[derive(Component)]
struct XpBar;

#[derive(Component)]
struct DebugOverlay;

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        HudCamera,
        Camera {
            order: 1,
            ..default()
        },
    ));

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .with_children(|parent| {
            // Crosshair - center of screen
            parent.spawn((
                Text::new("+"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    ..default()
                },
                Crosshair,
            ));

            // Bottom HUD container
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(120.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                })
                .with_children(|bottom| {
                    // Health and Hunger container
                    bottom
                        .spawn(Node {
                            width: Val::Px(432.0),
                            height: Val::Px(20.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        })
                        .with_children(|stats| {
                            // Health bar (left)
                            stats.spawn((
                                Text::new("❤❤❤❤❤❤❤❤❤❤"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.1, 0.1)),
                                HealthBar,
                            ));

                            // Hunger bar (right)
                            stats.spawn((
                                Text::new("🍗🍗🍗🍗🍗🍗🍗🍗🍗🍗"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.5, 0.2)),
                                HungerBar,
                            ));
                        });

                    // XP bar
                    bottom.spawn((
                        Node {
                            width: Val::Px(432.0),
                            height: Val::Px(4.0),
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                        XpBar,
                    ));

                    // Hotbar container
                    bottom
                        .spawn(Node {
                            width: Val::Px(432.0),
                            height: Val::Px(48.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        })
                        .with_children(|hotbar| {
                            for i in 0..9 {
                                hotbar.spawn((
                                    Node {
                                        width: Val::Px(48.0),
                                        height: Val::Px(48.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                                    BorderColor::all(if i == 0 {
                                        Color::WHITE
                                    } else {
                                        Color::srgba(0.4, 0.4, 0.4, 0.8)
                                    }),
                                    HotbarSlot(i),
                                ));
                            }
                        });
                });

            // Debug overlay (F3) - top left
            parent.spawn((
                Text::new("Ferrum Client v0.1.0\nFPS: 0\nXYZ: 0.0 / 0.0 / 0.0\nChunks: 0/0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                Visibility::Hidden,
                DebugOverlay,
            ));
        });
}

fn toggle_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut hud_state: ResMut<HudState>,
    mut query: Query<&mut Visibility, With<DebugOverlay>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        hud_state.show_debug = !hud_state.show_debug;
        for mut visibility in &mut query {
            *visibility = if hud_state.show_debug {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

fn update_debug_text(
    time: Res<Time>,
    mut hud_state: ResMut<HudState>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut text_query: Query<&mut Text, With<DebugOverlay>>,
) {
    let delta = time.delta_secs();
    if delta > 0.0 {
        hud_state.fps = 1.0 / delta;
    }

    if let Ok(transform) = camera_query.single() {
        hud_state.position = [
            transform.translation.x as f64,
            transform.translation.y as f64,
            transform.translation.z as f64,
        ];
    }

    if hud_state.show_debug {
        for mut text in &mut text_query {
            **text = format!(
                "Ferrum Client v0.1.0\nFPS: {:.0}\nXYZ: {:.1} / {:.1} / {:.1}\nChunks: {}/{}",
                hud_state.fps,
                hud_state.position[0],
                hud_state.position[1],
                hud_state.position[2],
                hud_state.chunk_count,
                hud_state.chunk_count
            );
        }
    }
}

fn update_hotbar_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut hud_state: ResMut<HudState>,
    mut query: Query<(&HotbarSlot, &mut BorderColor)>,
) {
    let mut new_slot = None;

    if keys.just_pressed(KeyCode::Digit1) {
        new_slot = Some(0);
    } else if keys.just_pressed(KeyCode::Digit2) {
        new_slot = Some(1);
    } else if keys.just_pressed(KeyCode::Digit3) {
        new_slot = Some(2);
    } else if keys.just_pressed(KeyCode::Digit4) {
        new_slot = Some(3);
    } else if keys.just_pressed(KeyCode::Digit5) {
        new_slot = Some(4);
    } else if keys.just_pressed(KeyCode::Digit6) {
        new_slot = Some(5);
    } else if keys.just_pressed(KeyCode::Digit7) {
        new_slot = Some(6);
    } else if keys.just_pressed(KeyCode::Digit8) {
        new_slot = Some(7);
    } else if keys.just_pressed(KeyCode::Digit9) {
        new_slot = Some(8);
    }

    if let Some(slot) = new_slot {
        hud_state.selected_slot = slot;

        for (hotbar_slot, mut border_color) in &mut query {
            *border_color = if hotbar_slot.0 == slot {
                BorderColor::all(Color::WHITE)
            } else {
                BorderColor::all(Color::srgba(0.4, 0.4, 0.4, 0.8))
            };
        }
    }
}
