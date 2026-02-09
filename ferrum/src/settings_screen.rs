use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use ferrum_config::Config;

pub struct SettingsScreenPlugin;

impl Plugin for SettingsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsScreenState>()
            .add_systems(Update, (toggle_settings_screen, handle_settings_input));
    }
}

#[derive(Resource, Default)]
struct SettingsScreenState {
    visible: bool,
}

#[derive(Component)]
struct SettingsScreenUI;

#[derive(Component)]
struct CloseButton;

#[derive(Component)]
enum SettingSlider {
    Fov,
    RenderDistance,
    MasterVolume,
}

fn toggle_settings_screen(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SettingsScreenState>,
    query: Query<Entity, With<SettingsScreenUI>>,
    config: Res<Config>,
) {
    // Toggle with Escape or a settings key
    if keyboard.just_pressed(KeyCode::F3) {
        state.visible = !state.visible;

        if state.visible {
            // Create settings screen
            spawn_settings_screen(&mut commands, &config);
        } else {
            // Destroy settings screen
            for entity in &query {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_settings_screen(commands: &mut Commands, config: &Config) {
    // Semi-transparent background
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            SettingsScreenUI,
        ))
        .with_children(|parent| {
            // Settings panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(600.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(30.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                ))
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // Video Settings Section
                    add_section_header(parent, "Video");

                    // FOV Setting
                    add_setting_row(
                        parent,
                        "Field of View",
                        &format!("{}", config.client.fov),
                        SettingSlider::Fov,
                    );

                    // Render Distance Setting
                    add_setting_row(
                        parent,
                        "Render Distance",
                        &format!("{} chunks", config.client.render_distance),
                        SettingSlider::RenderDistance,
                    );

                    // Audio Settings Section
                    add_section_header(parent, "Audio");

                    // Master Volume Setting
                    add_setting_row(
                        parent,
                        "Master Volume",
                        "100%",
                        SettingSlider::MasterVolume,
                    );

                    // Close button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(50.0),
                                margin: UiRect::top(Val::Px(30.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.5, 0.8)),
                            CloseButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Close"),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                        },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

fn add_section_header(parent: &mut ChildSpawnerCommands, title: &str) {
    parent.spawn((
        Text::new(title.to_string()),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.9, 1.0)),
        Node {
            margin: UiRect {
                top: Val::Px(20.0),
                bottom: Val::Px(15.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn add_setting_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: &str,
    _slider_type: SettingSlider,
) {
    parent
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(15.0)),
            ..default()
        },))
        .with_children(|parent| {
            // Label
            parent.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));

            // Value display
            parent.spawn((
                Text::new(value.to_string()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
            ));

            // TODO: Add slider component here
        });
}

fn handle_settings_input(
    mut interaction_query: Query<(&Interaction, Option<&CloseButton>), Changed<Interaction>>,
    mut state: ResMut<SettingsScreenState>,
    mut commands: Commands,
    query: Query<Entity, With<SettingsScreenUI>>,
) {
    for (interaction, close_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed && close_button.is_some() {
            state.visible = false;
            for entity in &query {
                commands.entity(entity).despawn();
            }
        }
    }
}
