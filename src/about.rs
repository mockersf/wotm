use bevy::prelude::*;
use tracing::info;

const CURRENT_SCREEN: crate::Screen = crate::Screen::About;

struct ScreenTag;

struct Screen {
    loaded: bool,
}
impl Default for Screen {
    fn default() -> Self {
        Screen { loaded: false }
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Screen::default())
            .add_system(input_system)
            .add_system(setup)
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down);
    }
}

fn setup(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_handles: ResMut<crate::AssetHandles>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");

        let color_none = materials.add(Color::NONE.into());

        let font: Handle<Font> = asset_handles.get_font_main_handle(&asset_server);

        let font_sub: Handle<Font> = asset_handles.get_font_sub_handle(&asset_server);

        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect::<Val> {
                        left: Val::Percent(20.),
                        right: Val::Undefined,
                        bottom: Val::Undefined,
                        top: Val::Percent(25.),
                    },
                    size: Size::<Val> {
                        height: Val::Px(190.),
                        width: Val::Auto,
                    },
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                material: color_none.clone(),
                ..Default::default()
            })
            .with_children(|title_parent| {
                title_parent.spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(75.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: "War of the Moons".to_string(),
                        font,
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT,
                            font_size: 75.0,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                });
                title_parent.spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(20.0),
                            ..Default::default()
                        },
                        margin: Rect {
                            right: Val::Px(10.0),
                            ..Default::default()
                        },
                        align_self: AlignSelf::FlexEnd,
                        ..Default::default()
                    },
                    text: Text {
                        value: format!("v{}", env!("CARGO_PKG_VERSION")),
                        font: font_sub.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT_DIM,
                            font_size: 20.0,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                });
            })
            .with(ScreenTag);

        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect::<Val> {
                        left: Val::Percent(10.),
                        right: Val::Undefined,
                        bottom: Val::Undefined,
                        top: Val::Percent(40.),
                    },
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                material: color_none.clone(),
                ..Default::default()
            })
            .with(ScreenTag)
            .with_children(|instruction_parent| {
                instruction_parent.spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(35.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: "Lead the revolution! Free other moons and satellites! Take control of the planet!".to_string(),
                        font: font_sub.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT,
                            font_size: 35.0,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                });
                instruction_parent.spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(35.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: "But beware, the planet will fight back...".to_string(),
                        font: font_sub.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT,
                            font_size: 35.0,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                });
                instruction_parent
                    .spawn(NodeBundle {
                        style: Style {
                            position: Rect::<Val> {
                                left: Val::Px(25.0),
                                right: Val::Undefined,
                                bottom: Val::Undefined,
                                top: Val::Px(5.0),
                            },
                            flex_direction: FlexDirection::ColumnReverse,
                            ..Default::default()
                        },
                        material: color_none.clone(),
                        ..Default::default()
                    })
                    .with_children(|controls_parent| {
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "You start with one moon / satellite".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "Select a moon with a left mouse click. Your moons will have a blue circle when selected".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "Send ships from a satellite you control to any other with a right clic".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "Control the number of ships you send by clicking on the same moon, or on the green / shadow square".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "If only your ships orbit a moon, you have freed it and it will help you".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "Send a ship to the planet to win, but it will destroy all your ships if there are still occupied moons".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "Once you freed all moons, the planet will trigger its shield".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "The planet will send enormous fleet of ships to all moons, but they are mostly rubbish...".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                        controls_parent.spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(30.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: Text {
                                value: "Watch out for the asteroids!".to_string(),
                                font: font_sub.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT,
                                    font_size: 25.0,
                                    ..Default::default()
                                },
                            },
                            ..Default::default()
                        });
                    });
            });

        screen.loaded = true;
    }
}

fn tear_down(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    query: Query<Entity, With<ScreenTag>>,
) {
    if game_screen.current_screen != CURRENT_SCREEN && screen.loaded {
        info!("tear down");

        for entity in query.iter() {
            commands.despawn_recursive(entity);
        }

        screen.loaded = false;
    }
}

fn input_system(
    mut game_screen: ResMut<crate::GameScreen>,
    screen: Res<Screen>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if game_screen.current_screen == CURRENT_SCREEN
        && screen.loaded
        && (mouse_button_input.just_pressed(MouseButton::Left)
            || keyboard_input.just_released(KeyCode::Escape)
            || keyboard_input.just_released(KeyCode::Space)
            || keyboard_input.just_released(KeyCode::Return))
    {
        game_screen.current_screen = crate::Screen::Menu;
    }
}
