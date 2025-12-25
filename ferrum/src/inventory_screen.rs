use crate::title_screen::GameState;
use bevy::prelude::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InventoryState>()
            .add_systems(OnEnter(GameState::InGame), setup_inventory_screen)
            .add_systems(
                Update,
                (
                    toggle_inventory,
                    handle_slot_interaction,
                    update_inventory_display,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource)]
pub struct InventoryState {
    pub is_open: bool,
    pub slots: [Option<ItemStack>; 36],
    pub armor: [Option<ItemStack>; 4],
    pub offhand: Option<ItemStack>,
    pub crafting: [Option<ItemStack>; 4],
    pub crafting_result: Option<ItemStack>,
    pub cursor_item: Option<ItemStack>,
}

#[derive(Clone, Debug)]
pub struct ItemStack {
    pub item_id: u16,
    pub count: u8,
    pub name: String,
}

impl Default for InventoryState {
    fn default() -> Self {
        let mut state = Self {
            is_open: false,
            slots: std::array::from_fn(|_| None),
            armor: std::array::from_fn(|_| None),
            offhand: None,
            crafting: std::array::from_fn(|_| None),
            crafting_result: None,
            cursor_item: None,
        };
        state.slots[0] = Some(ItemStack {
            item_id: 1,
            count: 64,
            name: "Stone".into(),
        });
        state.slots[1] = Some(ItemStack {
            item_id: 4,
            count: 64,
            name: "Cobblestone".into(),
        });
        state.slots[2] = Some(ItemStack {
            item_id: 3,
            count: 64,
            name: "Dirt".into(),
        });
        state.slots[3] = Some(ItemStack {
            item_id: 17,
            count: 64,
            name: "Oak Log".into(),
        });
        state.slots[4] = Some(ItemStack {
            item_id: 264,
            count: 1,
            name: "Diamond Sword".into(),
        });
        state.slots[5] = Some(ItemStack {
            item_id: 257,
            count: 1,
            name: "Iron Pickaxe".into(),
        });
        state
    }
}

#[derive(Component)]
struct InventoryUI;

#[derive(Component)]
struct InventorySlot {
    slot_type: SlotType,
    index: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum SlotType {
    MainInventory,
    Armor,
    Crafting,
    CraftingResult,
    Offhand,
}

#[derive(Component)]
struct SlotItemDisplay;

fn setup_inventory_screen(mut commands: Commands) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            Visibility::Hidden,
            InventoryUI,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(420.0),
                        height: Val::Px(380.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(16.0)),
                        row_gap: Val::Px(12.0),
                        border: UiRect::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.13)),
                    BorderColor::all(Color::srgb(0.08, 0.08, 0.09)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("INVENTORY"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.95, 0.95, 0.96)),
                        TextLayout::new_with_justify(Justify::Center),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(90.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        })
                        .with_children(|top| {
                            top.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|crafting_area| {
                                crafting_area
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        row_gap: Val::Px(2.0),
                                        ..default()
                                    })
                                    .with_children(|grid| {
                                        for row in 0..2 {
                                            grid.spawn(Node {
                                                flex_direction: FlexDirection::Row,
                                                column_gap: Val::Px(2.0),
                                                ..default()
                                            })
                                            .with_children(|row_container| {
                                                for col in 0..2 {
                                                    row_container
                                                        .spawn((
                                                            Button,
                                                            Node {
                                                                width: Val::Px(36.0),
                                                                height: Val::Px(36.0),
                                                                justify_content:
                                                                    JustifyContent::Center,
                                                                align_items: AlignItems::Center,
                                                                border: UiRect::all(Val::Px(2.0)),
                                                                ..default()
                                                            },
                                                            BackgroundColor(Color::srgb(
                                                                0.08, 0.08, 0.09,
                                                            )),
                                                            BorderColor::all(Color::srgb(
                                                                0.18, 0.18, 0.2,
                                                            )),
                                                            InventorySlot {
                                                                slot_type: SlotType::Crafting,
                                                                index: row * 2 + col,
                                                            },
                                                        ))
                                                        .with_children(|slot| {
                                                            slot.spawn((
                                                                Text::new(""),
                                                                TextFont {
                                                                    font_size: 10.0,
                                                                    ..default()
                                                                },
                                                                TextColor(Color::srgb(
                                                                    0.9, 0.9, 0.92,
                                                                )),
                                                                SlotItemDisplay,
                                                            ));
                                                        });
                                                }
                                            });
                                        }
                                    });

                                crafting_area
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        column_gap: Val::Px(4.0),
                                        ..default()
                                    })
                                    .with_children(|result_area| {
                                        result_area.spawn((
                                            Text::new("→"),
                                            TextFont {
                                                font_size: 20.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.5, 0.5, 0.52)),
                                        ));

                                        result_area
                                            .spawn((
                                                Button,
                                                Node {
                                                    width: Val::Px(36.0),
                                                    height: Val::Px(36.0),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    border: UiRect::all(Val::Px(2.0)),
                                                    ..default()
                                                },
                                                BackgroundColor(Color::srgb(0.08, 0.08, 0.09)),
                                                BorderColor::all(Color::srgb(0.18, 0.18, 0.2)),
                                                InventorySlot {
                                                    slot_type: SlotType::CraftingResult,
                                                    index: 0,
                                                },
                                            ))
                                            .with_children(|slot| {
                                                slot.spawn((
                                                    Text::new(""),
                                                    TextFont {
                                                        font_size: 10.0,
                                                        ..default()
                                                    },
                                                    TextColor(Color::srgb(0.9, 0.9, 0.92)),
                                                    SlotItemDisplay,
                                                ));
                                            });
                                    });
                            });

                            top.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(2.0),
                                ..default()
                            })
                            .with_children(|armor_area| {
                                for i in 0..4 {
                                    armor_area
                                        .spawn((
                                            Button,
                                            Node {
                                                width: Val::Px(36.0),
                                                height: Val::Px(36.0),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.08, 0.08, 0.09)),
                                            BorderColor::all(Color::srgb(0.18, 0.18, 0.2)),
                                            InventorySlot {
                                                slot_type: SlotType::Armor,
                                                index: i,
                                            },
                                        ))
                                        .with_children(|slot| {
                                            slot.spawn((
                                                Text::new(""),
                                                TextFont {
                                                    font_size: 10.0,
                                                    ..default()
                                                },
                                                TextColor(Color::srgb(0.9, 0.9, 0.92)),
                                                SlotItemDisplay,
                                            ));
                                        });
                                }
                            });
                        });

                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(2.0),
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        })
                        .with_children(|inventory| {
                            for row in 0..3 {
                                inventory
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(2.0),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    })
                                    .with_children(|row_container| {
                                        for col in 0..9 {
                                            row_container
                                                .spawn((
                                                    Button,
                                                    Node {
                                                        width: Val::Px(36.0),
                                                        height: Val::Px(36.0),
                                                        justify_content: JustifyContent::Center,
                                                        align_items: AlignItems::Center,
                                                        border: UiRect::all(Val::Px(2.0)),
                                                        ..default()
                                                    },
                                                    BackgroundColor(Color::srgb(0.08, 0.08, 0.09)),
                                                    BorderColor::all(Color::srgb(0.18, 0.18, 0.2)),
                                                    InventorySlot {
                                                        slot_type: SlotType::MainInventory,
                                                        index: row * 9 + col,
                                                    },
                                                ))
                                                .with_children(|slot| {
                                                    slot.spawn((
                                                        Text::new(""),
                                                        TextFont {
                                                            font_size: 10.0,
                                                            ..default()
                                                        },
                                                        TextColor(Color::srgb(0.9, 0.9, 0.92)),
                                                        SlotItemDisplay,
                                                    ));
                                                });
                                        }
                                    });
                            }
                        });

                    panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(2.0),
                            justify_content: JustifyContent::Center,
                            padding: UiRect::top(Val::Px(6.0)),
                            border: UiRect::top(Val::Px(2.0)),
                            ..default()
                        })
                        .with_child(BorderColor::all(Color::srgb(0.2, 0.2, 0.22)))
                        .with_children(|hotbar| {
                            for i in 0..9 {
                                hotbar
                                    .spawn((
                                        Button,
                                        Node {
                                            width: Val::Px(36.0),
                                            height: Val::Px(36.0),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(2.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.08, 0.08, 0.09)),
                                        BorderColor::all(Color::srgb(0.18, 0.18, 0.2)),
                                        InventorySlot {
                                            slot_type: SlotType::MainInventory,
                                            index: 27 + i,
                                        },
                                    ))
                                    .with_children(|slot| {
                                        slot.spawn((
                                            Text::new(""),
                                            TextFont {
                                                font_size: 10.0,
                                                ..default()
                                            },
                                            TextColor(Color::srgb(0.9, 0.9, 0.92)),
                                            SlotItemDisplay,
                                        ));
                                    });
                            }
                        });
                });
        });
}

fn toggle_inventory(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory_state: ResMut<InventoryState>,
    mut ui_query: Query<&mut Visibility, With<InventoryUI>>,
    mut cursor_options: Single<&mut bevy::window::CursorOptions>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        inventory_state.is_open = !inventory_state.is_open;

        if let Some(mut visibility) = ui_query.iter_mut().next() {
            *visibility = if inventory_state.is_open {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }

        if inventory_state.is_open {
            cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
            cursor_options.visible = true;
        } else {
            cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
            cursor_options.visible = false;
        }
    }
}

fn handle_slot_interaction(
    mut interaction_query: Query<
        (&Interaction, &InventorySlot, &mut BorderColor),
        Changed<Interaction>,
    >,
    mut inventory_state: ResMut<InventoryState>,
) {
    for (interaction, slot, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Take cursor_item temporarily to avoid double borrow
                let mut cursor_item = inventory_state.cursor_item.take();

                let slot_item = match slot.slot_type {
                    SlotType::MainInventory => &mut inventory_state.slots[slot.index],
                    SlotType::Armor => &mut inventory_state.armor[slot.index],
                    SlotType::Crafting => &mut inventory_state.crafting[slot.index],
                    SlotType::CraftingResult => &mut inventory_state.crafting_result,
                    SlotType::Offhand => &mut inventory_state.offhand,
                };

                std::mem::swap(slot_item, &mut cursor_item);
                inventory_state.cursor_item = cursor_item;

                *border_color = BorderColor::all(Color::srgb(0.9, 0.85, 0.4));
            }
            Interaction::Hovered => {
                *border_color = BorderColor::all(Color::srgb(0.6, 0.6, 0.62));
            }
            Interaction::None => {
                *border_color = BorderColor::all(Color::srgb(0.18, 0.18, 0.2));
            }
        }
    }
}

fn update_inventory_display(
    inventory_state: Res<InventoryState>,
    slot_query: Query<(&InventorySlot, &Children)>,
    mut text_query: Query<&mut Text, With<SlotItemDisplay>>,
) {
    if !inventory_state.is_changed() {
        return;
    }

    for (slot, children) in &slot_query {
        let item = match slot.slot_type {
            SlotType::MainInventory => &inventory_state.slots[slot.index],
            SlotType::Armor => &inventory_state.armor[slot.index],
            SlotType::Crafting => &inventory_state.crafting[slot.index],
            SlotType::CraftingResult => &inventory_state.crafting_result,
            SlotType::Offhand => &inventory_state.offhand,
        };

        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if let Some(stack) = item {
                    let abbrev = stack.name.chars().take(3).collect::<String>();
                    **text = if stack.count > 1 {
                        format!("{}\n{}", abbrev.to_uppercase(), stack.count)
                    } else {
                        abbrev.to_uppercase()
                    };
                } else {
                    **text = String::new();
                }
            }
        }
    }
}
