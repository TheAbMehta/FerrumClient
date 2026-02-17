use bevy::app::AppExit;
use bevy::prelude::*;

pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::TitleScreen), setup_title_screen)
            .add_systems(OnExit(GameState::TitleScreen), cleanup_title_screen)
            .add_systems(OnEnter(GameState::Loading), setup_loading_screen)
            .add_systems(OnExit(GameState::Loading), cleanup_loading_screen)
            .add_systems(
                Update,
                (tick_title_ready, handle_title_buttons)
                    .chain()
                    .run_if(in_state(GameState::TitleScreen)),
            )
            .add_systems(
                Update,
                transition_to_ingame.run_if(in_state(GameState::Loading)),
            );
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    TitleScreen,
    Loading,
    InGame,
    Dead,
}

#[derive(Component)]
struct TitleScreenRoot;

#[derive(Component)]
struct LoadingScreenRoot;

#[derive(Component)]
enum TitleButton {
    Singleplayer,
    Multiplayer,
    Settings,
    Quit,
}

#[derive(Resource)]
struct TitleScreenReady(bool, f32);

const BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.08);
const PANEL_BG: Color = Color::srgb(0.12, 0.12, 0.15);
const BUTTON_NORMAL: Color = Color::srgb(0.18, 0.18, 0.22);
const BUTTON_HOVER: Color = Color::srgb(0.28, 0.28, 0.32);
const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.35, 0.40);
const TEXT_PRIMARY: Color = Color::srgb(0.95, 0.95, 0.98);
const TEXT_ACCENT: Color = Color::srgb(1.0, 0.85, 0.0);
const BORDER_COLOR: Color = Color::srgb(0.4, 0.4, 0.45);

fn setup_title_screen(mut commands: Commands) {
    commands.insert_resource(TitleScreenReady(false, 0.0));

    commands.spawn((Camera2d, TitleScreenRoot));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            TitleScreenRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("FERRUM"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(TEXT_ACCENT),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("A Minecraft Client"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(32.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Blazingly fast!"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(TEXT_ACCENT),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(48.0)),
                    ..default()
                },
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|buttons| {
                    buttons
                        .spawn((
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(BUTTON_NORMAL),
                            BorderColor::all(BORDER_COLOR),
                            Button,
                            TitleButton::Singleplayer,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Singleplayer"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });

                    buttons
                        .spawn((
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(BUTTON_NORMAL),
                            BorderColor::all(BORDER_COLOR),
                            Button,
                            TitleButton::Multiplayer,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Multiplayer"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });

                    buttons
                        .spawn((
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(BUTTON_NORMAL),
                            BorderColor::all(BORDER_COLOR),
                            Button,
                            TitleButton::Settings,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Settings"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });

                    buttons
                        .spawn((
                            Node {
                                width: Val::Px(400.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(BUTTON_NORMAL),
                            BorderColor::all(BORDER_COLOR),
                            Button,
                            TitleButton::Quit,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Quit Game"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });
                });

            parent.spawn((
                Text::new("Ferrum v0.1.0"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(16.0),
                    left: Val::Px(16.0),
                    ..default()
                },
            ));
        });
}

fn cleanup_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn setup_loading_screen(mut commands: Commands) {
    // Spawn a camera for the loading screen UI
    commands.spawn((Camera2d, LoadingScreenRoot));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            LoadingScreenRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading world..."),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

fn cleanup_loading_screen(mut commands: Commands, query: Query<Entity, With<LoadingScreenRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn tick_title_ready(time: Res<Time>, mut ready: ResMut<TitleScreenReady>) {
    if !ready.0 {
        ready.1 += time.delta_secs();
        if ready.1 >= 0.5 {
            ready.0 = true;
        }
    }
}

fn handle_title_buttons(
    ready: Res<TitleScreenReady>,
    mut interaction_query: Query<
        (&Interaction, &TitleButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !ready.0 {
        return;
    }
    for (interaction, button, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(BUTTON_PRESSED);

                match button {
                    TitleButton::Singleplayer => {
                        next_state.set(GameState::Loading);
                    }
                    TitleButton::Multiplayer => {
                        info!("Multiplayer: Coming Soon!");
                    }
                    TitleButton::Settings => {
                        info!("Settings: Coming Soon!");
                    }
                    TitleButton::Quit => {
                        app_exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(BUTTON_HOVER);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(BUTTON_NORMAL);
            }
        }
    }
}

fn transition_to_ingame(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}
