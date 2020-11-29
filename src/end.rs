use bevy::prelude::*;
use tracing::info;

const CURRENT_SCREEN: crate::Screen = crate::Screen::End;

struct ScreenTag;

struct Screen {
    loaded: bool,
}
impl Default for Screen {
    fn default() -> Self {
        Screen { loaded: false }
    }
}

#[derive(Default)]
pub struct GameStats {}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Screen::default())
            .init_resource::<GameStats>()
            .add_system(input_system)
            .add_system(setup)
            .add_system(update_stats)
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down);
    }
}

fn update_stats(
    mut _stats: ResMut<GameStats>,
    _game: Res<crate::game::Game>,
    (mut interesting_event_reader, interesting_events): (
        Local<EventReader<crate::game::InterestingEvent>>,
        ResMut<Events<crate::game::InterestingEvent>>,
    ),
) {
    for event in interesting_event_reader.iter(&interesting_events) {
        match event {
            _ => (),
        }
    }
}

fn setup(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    mut game: ResMut<crate::game::Game>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut asset_handles: ResMut<crate::AssetHandles>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");

        let font: Handle<Font> = asset_handles.get_font_main_handle(&asset_server);

        commands
            .spawn(NodeBundle {
                style: Style {
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with(ScreenTag)
            .with_children(|parent| {
                parent.spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(130.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: match game.state {
                            crate::game::GameState::Win => "You won".to_string(),
                            crate::game::GameState::Lose => "You lost".to_string(),
                            _ => "...".to_string(),
                        },
                        font: font.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT,
                            font_size: 130.,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                });
                parent.spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(60.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: format!("final score: {}", game.score as u32),
                        font,
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT,
                            font_size: 60.,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                });
            });

        *game = crate::game::Game::default();

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
            || keyboard_input.just_released(KeyCode::Space))
    {
        game_screen.current_screen = crate::Screen::Menu;
    }
}
