use bevy::prelude::*;
use tracing::info;

const CURRENT_SCREEN: crate::Screen = crate::Screen::Game;

struct ScreenTag;

pub struct Screen {
    loaded: bool,
    first_load: bool,
}
impl Default for Screen {
    fn default() -> Self {
        Screen {
            loaded: false,
            first_load: true,
        }
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Screen::default())
            .init_resource::<Game>()
            .add_event::<GameEvents>()
            .add_event::<InterestingEvent>()
            .add_system(setup.system())
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down.system());
    }
}

fn setup(
    // mut commands: Commands,
    (game_screen, _game, mut screen): (Res<crate::GameScreen>, Res<Game>, ResMut<Screen>),
    _wnds: Res<Windows>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");

        // let _ratio = wnds.get_primary().unwrap().width() as f32 / BOARD_X as f32 / TILE_SIZE as f32;

        screen.loaded = true;
        screen.first_load = false;
    }
}

fn tear_down(
    mut commands: Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    query: Query<With<ScreenTag, Entity>>,
) {
    if game_screen.current_screen != CURRENT_SCREEN && screen.loaded {
        info!("tear down");

        for entity in query.iter() {
            commands.despawn_recursive(entity);
        }

        screen.loaded = false;
    }
}

struct Player {}

#[derive(Clone, Copy)]
enum FacingDirection {}

impl Default for Player {
    fn default() -> Self {
        Player {}
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GameState {
    Play,
    // Pause(Entity),
    // Death,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Play
    }
}

#[derive(Default)]
pub struct Game {
    // player: Player,
    pub round: u16,
    pub score: u32,
    // state: GameState,
}

#[derive(PartialEq)]
pub enum GameEvents {
    // Lost,
// Pause,
// NewHighscore,
}

pub enum InterestingEvent {}
