use crate::title_screen::GameState;
use bevy::prelude::*;

pub struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dead), setup_death_screen)
            .add_systems(OnExit(GameState::Dead), cleanup_death_screen)
            .add_systems(
                Update,
                handle_death_screen_input.run_if(in_state(GameState::Dead)),
            );
    }
}

#[derive(Component)]
struct DeathScreenUI;

#[derive(Component)]
struct RespawnButton;

#[derive(Component)]
struct TitleButton;

fn setup_death_screen(mut commands: Commands) {
    // Red overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.5)),
        DeathScreenUI,
    ));

    // Death message container
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
            DeathScreenUI,
        ))
        .with_children(|parent| {
            // "You died!" text
            parent.spawn((
                Text::new("You died!"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                Node {
                    margin: UiRect::all(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Score text (if applicable)
            parent.spawn((
                Text::new("Score: 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Respawn button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        margin: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
                    RespawnButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Respawn"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Title screen button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        margin: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                    TitleButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Title Screen"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn cleanup_death_screen(mut commands: Commands, query: Query<Entity, With<DeathScreenUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_death_screen_input(
    mut interaction_query: Query<
        (&Interaction, Option<&RespawnButton>, Option<&TitleButton>),
        Changed<Interaction>,
    >,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, respawn_button, title_button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            if respawn_button.is_some() {
                info!("Respawning player...");
                // TODO: Send respawn packet to server
                game_state.set(GameState::InGame);
            } else if title_button.is_some() {
                info!("Returning to title screen...");
                // TODO: Disconnect from server
                game_state.set(GameState::TitleScreen);
            }
        }
    }
}
