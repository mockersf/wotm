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
            .add_system(input_system.system())
            .add_system(setup.system())
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down.system());
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
            .spawn(NodeComponents {
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
                title_parent.spawn(TextComponents {
                    style: Style {
                        size: Size {
                            height: Val::Px(150. / 2.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: "War of the Moons".to_string(),
                        font,
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT,
                            font_size: 150.0 / 2.,
                        },
                    },
                    ..Default::default()
                });
                title_parent.spawn(TextComponents {
                    style: Style {
                        size: Size {
                            height: Val::Px(40. / 2.),
                            ..Default::default()
                        },
                        margin: Rect {
                            right: Val::Px(20. / 2.),
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
                            font_size: 40.0 / 2.,
                        },
                    },
                    ..Default::default()
                });
            })
            .with(ScreenTag);

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
