use crate::title_screen::GameState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatState>()
            .add_systems(OnEnter(GameState::InGame), setup_chat)
            .add_systems(
                Update,
                (
                    toggle_chat,
                    handle_chat_input,
                    update_chat_messages,
                    fade_old_messages,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource)]
pub struct ChatState {
    pub is_open: bool,
    pub input_text: String,
    pub messages: Vec<ChatMessage>,
    pub max_visible: usize,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            is_open: false,
            input_text: String::new(),
            messages: Vec::new(),
            max_visible: 10,
        }
    }
}

pub struct ChatMessage {
    pub text: String,
    pub timestamp: f64,
    pub color: Color,
}

#[derive(Component)]
struct ChatContainer;

#[derive(Component)]
struct ChatInputBar;

#[derive(Component)]
struct ChatMessageText(usize);

fn setup_chat(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            bottom: Val::Px(88.0),
            flex_direction: FlexDirection::ColumnReverse,
            row_gap: Val::Px(2.0),
            ..default()
        },
        ChatContainer,
        Visibility::Inherited,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::all(Val::Px(12.0)),
                border: UiRect::top(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.95)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.4)),
            ChatInputBar,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("> "),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.4)),
                Node { ..default() },
            ));
        });
}

fn toggle_chat(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut chat_state: ResMut<ChatState>,
    mut input_bar_query: Query<&mut Visibility, With<ChatInputBar>>,
    time: Res<Time>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) && !chat_state.is_open {
        chat_state.is_open = true;
        chat_state.input_text.clear();

        if let Some(mut visibility) = input_bar_query.iter_mut().next() {
            *visibility = Visibility::Inherited;
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) && chat_state.is_open {
        chat_state.is_open = false;
        chat_state.input_text.clear();

        if let Some(mut visibility) = input_bar_query.iter_mut().next() {
            *visibility = Visibility::Hidden;
        }
    }

    if keyboard.just_pressed(KeyCode::Enter) && chat_state.is_open {
        if !chat_state.input_text.is_empty() {
            let input_text = chat_state.input_text.clone();
            chat_state.messages.push(ChatMessage {
                text: input_text,
                timestamp: time.elapsed_secs_f64(),
                color: Color::srgb(0.95, 0.95, 0.95),
            });

            chat_state.input_text.clear();
        }

        chat_state.is_open = false;

        if let Some(mut visibility) = input_bar_query.iter_mut().next() {
            *visibility = Visibility::Hidden;
        }
    }
}

fn handle_chat_input(
    mut chat_state: ResMut<ChatState>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut input_bar_query: Query<&Children, With<ChatInputBar>>,
    mut text_query: Query<&mut Text>,
) {
    if !chat_state.is_open {
        return;
    }

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Character(ch) => {
                chat_state.input_text.push_str(ch.as_str());
            }
            Key::Backspace => {
                chat_state.input_text.pop();
            }
            Key::Space => {
                chat_state.input_text.push(' ');
            }
            _ => {}
        }

        if let Some(children) = input_bar_query.iter().next() {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = format!("> {}", chat_state.input_text);
                }
            }
        }
    }
}

fn update_chat_messages(
    mut commands: Commands,
    chat_state: Res<ChatState>,
    container_query: Query<Entity, With<ChatContainer>>,
    message_query: Query<Entity, With<ChatMessageText>>,
) {
    if !chat_state.is_changed() {
        return;
    }

    let Some(container) = container_query.iter().next() else {
        return;
    };

    for entity in message_query.iter() {
        commands.entity(entity).despawn();
    }

    let start_idx = chat_state
        .messages
        .len()
        .saturating_sub(chat_state.max_visible);

    for (idx, message) in chat_state.messages.iter().enumerate().skip(start_idx) {
        let message_entity = commands
            .spawn((
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    margin: UiRect::bottom(Val::Px(1.0)),
                    border: UiRect::left(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                BorderColor::all(Color::srgb(0.0, 1.0, 0.4)),
                ChatMessageText(idx),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(&message.text),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(message.color),
                ));
            })
            .id();

        commands.entity(container).add_child(message_entity);
    }
}

fn fade_old_messages(
    chat_state: Res<ChatState>,
    time: Res<Time>,
    mut message_query: Query<(&ChatMessageText, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut TextColor>,
) {
    let current_time = time.elapsed_secs_f64();

    for (message_idx, mut bg_color, children) in message_query.iter_mut() {
        if let Some(message) = chat_state.messages.get(message_idx.0) {
            let age = current_time - message.timestamp;

            if chat_state.is_open {
                bg_color.0.set_alpha(0.75);

                for child in children.iter() {
                    if let Ok(mut text_color) = text_query.get_mut(child) {
                        text_color.0.set_alpha(1.0);
                    }
                }
            } else if age > 10.0 {
                let fade_duration = 5.0;
                let fade_progress = ((age - 10.0) / fade_duration).clamp(0.0, 1.0);
                let alpha = 1.0 - fade_progress;

                bg_color.0.set_alpha(0.75 * alpha as f32);

                for child in children.iter() {
                    if let Ok(mut text_color) = text_query.get_mut(child) {
                        text_color.0.set_alpha(alpha as f32);
                    }
                }
            }
        }
    }
}
