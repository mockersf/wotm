use bevy::prelude::*;
use rand::{seq::SliceRandom, Rng};
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
    commands: &mut Commands,
    (game_screen, _game, mut screen): (Res<crate::GameScreen>, Res<Game>, ResMut<Screen>),
    _wnds: Res<Windows>,
    asset_handles: Res<crate::AssetHandles>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");

        // let _ratio = wnds.get_primary().unwrap().width() as f32 / BOARD_X as f32 / TILE_SIZE as f32;

        let game_handles = asset_handles.get_game_handles_unsafe();

        let planet = game_handles
            .planets
            .choose(&mut rand::thread_rng())
            .unwrap();

        commands
            .spawn(SpriteComponents {
                transform: Transform {
                    scale: Vec3::splat(0.10),
                    ..Default::default()
                },
                material: planet.clone(),
                ..Default::default()
            })
            .with(bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic().angvel(0.1))
            .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true))
            .with(ScreenTag);
        let planet = commands.current_entity().unwrap();

        commands
            .spawn(SpriteComponents {
                transform: Transform {
                    scale: Vec3::splat(0.10),
                    ..Default::default()
                },
                material: game_handles
                    .orbiters
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone_weak(),
                ..Default::default()
            })
            .with(crate::space::Orbiter::every(
                rand::thread_rng().gen_range(0.01, 0.05),
                planet,
                300.,
            ))
            .with(bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic().angvel(0.1))
            .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true))
            .with(crate::space::SpawnShip::every(5.))
            .with(ScreenTag);

        commands
            .spawn(SpriteComponents {
                transform: Transform {
                    scale: Vec3::splat(0.10),
                    ..Default::default()
                },
                material: game_handles
                    .orbiters
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone_weak(),
                ..Default::default()
            })
            .with(crate::space::Orbiter::every(
                rand::thread_rng().gen_range(0.01, 0.05),
                planet,
                200.,
            ))
            .with(bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic().angvel(0.1))
            .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true))
            .with(crate::space::SpawnShip::every(5.))
            .with(ScreenTag);

        commands
            .spawn(SpriteComponents {
                transform: Transform {
                    scale: Vec3::splat(0.10),
                    ..Default::default()
                },
                material: game_handles
                    .orbiters
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .clone_weak(),
                ..Default::default()
            })
            .with(crate::space::Orbiter::every(
                rand::thread_rng().gen_range(0.01, 0.05),
                planet,
                100.,
            ))
            .with(bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic().angvel(0.1))
            .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true))
            .with(crate::space::SpawnShip::every(5.))
            .with(ScreenTag);

        screen.loaded = true;
        screen.first_load = false;
    }
}

fn tear_down(
    commands: &mut Commands,
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
