use crate::title_screen::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(OnEnter(GameState::InGame), setup_menu)
            .add_systems(
                Update,
                (
                    toggle_menu,
                    handle_pause_buttons,
                    handle_settings_buttons,
                    update_button_visuals,
                    update_slider_values,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource)]
pub struct MenuState {
    pub is_open: bool,
    pub current_screen: MenuScreen,
    pub render_distance: u32,
    pub fov: f32,
    pub mouse_sensitivity: f32,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            is_open: false,
            current_screen: MenuScreen::Pause,
            render_distance: 8,
            fov: 70.0,
            mouse_sensitivity: 1.0,
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum MenuScreen {
    #[default]
    Pause,
    Settings,
}

// Component markers
#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct PauseMenuContainer;

#[derive(Component)]
struct SettingsMenuContainer;

#[derive(Component)]
enum MenuButton {
    Resume,
    Settings,
    Quit,
    SettingsDone,
}

#[derive(Component)]
enum SliderButton {
    RenderDistanceDown,
    RenderDistanceUp,
    FovDown,
    FovUp,
    SensitivityDown,
    SensitivityUp,
}

#[derive(Component)]
struct RenderDistanceText;

#[derive(Component)]
struct FovText;

#[derive(Component)]
struct SensitivityText;

// Color palette - Brutalist/Industrial theme
const BG_OVERLAY: Color = Color::srgba(0.05, 0.05, 0.08, 0.92);
const PANEL_BG: Color = Color::srgb(0.12, 0.12, 0.15);
const BUTTON_NORMAL: Color = Color::srgb(0.18, 0.18, 0.22);
const BUTTON_HOVER: Color = Color::srgb(0.28, 0.28, 0.32);
const BUTTON_PRESSED: Color = Color::srgb(0.35, 0.35, 0.40);
const TEXT_PRIMARY: Color = Color::srgb(0.95, 0.95, 0.98);
const TEXT_ACCENT: Color = Color::srgb(1.0, 0.85, 0.0);
const BORDER_COLOR: Color = Color::srgb(0.4, 0.4, 0.45);

fn setup_menu(mut commands: Commands) {
    // Root menu container (hidden by default)
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
            BackgroundColor(BG_OVERLAY),
            Visibility::Hidden,
            MenuRoot,
        ))
        .with_children(|parent| {
            // PAUSE MENU
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        row_gap: Val::Px(24.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                    BorderColor::all(BORDER_COLOR),
                    Visibility::Inherited,
                    PauseMenuContainer,
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("GAME MENU"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(TEXT_ACCENT),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        },
                    ));

                    // Resume button
                    panel
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
                            MenuButton::Resume,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("BACK TO GAME"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });

                    // Settings button
                    panel
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
                            MenuButton::Settings,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("OPTIONS..."),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });

                    // Quit button
                    panel
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
                            MenuButton::Quit,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("SAVE AND QUIT TO TITLE"),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                                TextLayout::new_with_justify(Justify::Center),
                            ));
                        });
                });

            // SETTINGS MENU
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        row_gap: Val::Px(20.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL_BG),
                    BorderColor::all(BORDER_COLOR),
                    Visibility::Hidden,
                    SettingsMenuContainer,
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("OPTIONS"),
                        TextFont {
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(TEXT_ACCENT),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        },
                    ));

                    // Render Distance slider
                    panel
                        .spawn(Node {
                            width: Val::Px(500.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|container| {
                            container.spawn((
                                Text::new("RENDER DISTANCE"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                            ));

                            container
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        Node {
                                            width: Val::Px(50.0),
                                            height: Val::Px(50.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_NORMAL),
                                        BorderColor::all(BORDER_COLOR),
                                        Button,
                                        SliderButton::RenderDistanceDown,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("−"),
                                            TextFont {
                                                font_size: 28.0,
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });

                                    row.spawn((
                                        Text::new("8"),
                                        TextFont {
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_ACCENT),
                                        TextLayout::new_with_justify(Justify::Center),
                                        Node {
                                            width: Val::Px(100.0),
                                            ..default()
                                        },
                                        RenderDistanceText,
                                    ));

                                    row.spawn((
                                        Node {
                                            width: Val::Px(50.0),
                                            height: Val::Px(50.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_NORMAL),
                                        BorderColor::all(BORDER_COLOR),
                                        Button,
                                        SliderButton::RenderDistanceUp,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("+"),
                                            TextFont {
                                                font_size: 28.0,
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });
                                });
                        });

                    // FOV slider
                    panel
                        .spawn(Node {
                            width: Val::Px(500.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|container| {
                            container.spawn((
                                Text::new("FOV"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                            ));

                            container
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        Node {
                                            width: Val::Px(50.0),
                                            height: Val::Px(50.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_NORMAL),
                                        BorderColor::all(BORDER_COLOR),
                                        Button,
                                        SliderButton::FovDown,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("−"),
                                            TextFont {
                                                font_size: 28.0,
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });

                                    row.spawn((
                                        Text::new("70"),
                                        TextFont {
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_ACCENT),
                                        TextLayout::new_with_justify(Justify::Center),
                                        Node {
                                            width: Val::Px(100.0),
                                            ..default()
                                        },
                                        FovText,
                                    ));

                                    row.spawn((
                                        Node {
                                            width: Val::Px(50.0),
                                            height: Val::Px(50.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_NORMAL),
                                        BorderColor::all(BORDER_COLOR),
                                        Button,
                                        SliderButton::FovUp,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("+"),
                                            TextFont {
                                                font_size: 28.0,
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });
                                });
                        });

                    // Mouse Sensitivity slider
                    panel
                        .spawn(Node {
                            width: Val::Px(500.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|container| {
                            container.spawn((
                                Text::new("MOUSE SENSITIVITY"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(TEXT_PRIMARY),
                            ));

                            container
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(12.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        Node {
                                            width: Val::Px(50.0),
                                            height: Val::Px(50.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_NORMAL),
                                        BorderColor::all(BORDER_COLOR),
                                        Button,
                                        SliderButton::SensitivityDown,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("−"),
                                            TextFont {
                                                font_size: 28.0,
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });

                                    row.spawn((
                                        Text::new("1.0"),
                                        TextFont {
                                            font_size: 24.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_ACCENT),
                                        TextLayout::new_with_justify(Justify::Center),
                                        Node {
                                            width: Val::Px(100.0),
                                            ..default()
                                        },
                                        SensitivityText,
                                    ));

                                    row.spawn((
                                        Node {
                                            width: Val::Px(50.0),
                                            height: Val::Px(50.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(BUTTON_NORMAL),
                                        BorderColor::all(BORDER_COLOR),
                                        Button,
                                        SliderButton::SensitivityUp,
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new("+"),
                                            TextFont {
                                                font_size: 28.0,
                                                ..default()
                                            },
                                            TextColor(TEXT_PRIMARY),
                                            TextLayout::new_with_justify(Justify::Center),
                                        ));
                                    });
                                });
                        });

                    // Done button
                    panel
                        .spawn(Node {
                            margin: UiRect::top(Val::Px(16.0)),
                            ..default()
                        })
                        .with_children(|container| {
                            container
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
                                    MenuButton::SettingsDone,
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("DONE"),
                                        TextFont {
                                            font_size: 20.0,
                                            ..default()
                                        },
                                        TextColor(TEXT_PRIMARY),
                                        TextLayout::new_with_justify(Justify::Center),
                                    ));
                                });
                        });
                });
        });
}

fn toggle_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut menu_root: Query<&mut Visibility, With<MenuRoot>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        menu_state.is_open = !menu_state.is_open;

        if let Some(mut visibility) = menu_root.iter_mut().next() {
            if menu_state.is_open {
                *visibility = Visibility::Inherited;
                cursor_options.grab_mode = CursorGrabMode::None;
                cursor_options.visible = true;
                menu_state.current_screen = MenuScreen::Pause;
            } else {
                *visibility = Visibility::Hidden;
                cursor_options.grab_mode = CursorGrabMode::Locked;
                cursor_options.visible = false;
            }
        }
    }
}

fn handle_pause_buttons(
    mut interaction_query: Query<(&Interaction, &MenuButton), (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<MenuState>,
    mut pause_container: Query<&mut Visibility, With<PauseMenuContainer>>,
    mut settings_container: Query<
        &mut Visibility,
        (With<SettingsMenuContainer>, Without<PauseMenuContainer>),
    >,
    mut app_exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::Resume => {
                    menu_state.is_open = false;
                    // Menu will be hidden by toggle_menu system
                }
                MenuButton::Settings => {
                    menu_state.current_screen = MenuScreen::Settings;
                    if let Some(mut vis) = pause_container.iter_mut().next() {
                        *vis = Visibility::Hidden;
                    }
                    if let Some(mut vis) = settings_container.iter_mut().next() {
                        *vis = Visibility::Inherited;
                    }
                }
                MenuButton::Quit => {
                    app_exit.write(AppExit::Success);
                }
                _ => {}
            }
        }
    }
}

fn handle_settings_buttons(
    mut interaction_query: Query<
        (&Interaction, Option<&MenuButton>, Option<&SliderButton>),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu_state: ResMut<MenuState>,
    mut pause_container: Query<&mut Visibility, With<PauseMenuContainer>>,
    mut settings_container: Query<
        &mut Visibility,
        (With<SettingsMenuContainer>, Without<PauseMenuContainer>),
    >,
) {
    for (interaction, menu_btn, slider_btn) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Handle menu buttons
            if let Some(MenuButton::SettingsDone) = menu_btn {
                menu_state.current_screen = MenuScreen::Pause;
                if let Some(mut vis) = settings_container.iter_mut().next() {
                    *vis = Visibility::Hidden;
                }
                if let Some(mut vis) = pause_container.iter_mut().next() {
                    *vis = Visibility::Inherited;
                }
            }

            // Handle slider buttons
            if let Some(slider) = slider_btn {
                match slider {
                    SliderButton::RenderDistanceDown => {
                        menu_state.render_distance = (menu_state.render_distance - 1).max(2);
                    }
                    SliderButton::RenderDistanceUp => {
                        menu_state.render_distance = (menu_state.render_distance + 1).min(32);
                    }
                    SliderButton::FovDown => {
                        menu_state.fov = (menu_state.fov - 5.0).max(30.0);
                    }
                    SliderButton::FovUp => {
                        menu_state.fov = (menu_state.fov + 5.0).min(110.0);
                    }
                    SliderButton::SensitivityDown => {
                        menu_state.mouse_sensitivity =
                            (menu_state.mouse_sensitivity - 0.1).max(0.1);
                    }
                    SliderButton::SensitivityUp => {
                        menu_state.mouse_sensitivity =
                            (menu_state.mouse_sensitivity + 0.1).min(5.0);
                    }
                }
            }
        }
    }
}

fn update_button_visuals(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(BUTTON_PRESSED);
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

fn update_slider_values(
    menu_state: Res<MenuState>,
    mut render_distance_text: Query<
        &mut Text,
        (
            With<RenderDistanceText>,
            Without<FovText>,
            Without<SensitivityText>,
        ),
    >,
    mut fov_text: Query<
        &mut Text,
        (
            With<FovText>,
            Without<RenderDistanceText>,
            Without<SensitivityText>,
        ),
    >,
    mut sensitivity_text: Query<
        &mut Text,
        (
            With<SensitivityText>,
            Without<RenderDistanceText>,
            Without<FovText>,
        ),
    >,
) {
    if menu_state.is_changed() {
        if let Some(mut text) = render_distance_text.iter_mut().next() {
            **text = menu_state.render_distance.to_string();
        }
        if let Some(mut text) = fov_text.iter_mut().next() {
            **text = format!("{:.0}", menu_state.fov);
        }
        if let Some(mut text) = sensitivity_text.iter_mut().next() {
            **text = format!("{:.1}", menu_state.mouse_sensitivity);
        }
    }
}
