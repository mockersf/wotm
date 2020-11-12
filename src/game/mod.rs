use bevy::prelude::*;
use rand::{seq::SliceRandom, Rng};
use tracing::info;

const CURRENT_SCREEN: crate::Screen = crate::Screen::Game;

pub mod ui;

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
            .add_event::<ui::InteractionEvent>()
            .add_system(keyboard_input_system.system())
            .add_system(ui::setup.system())
            .add_system(ui::interaction.system())
            .add_system_to_stage(bevy::app::stage::PRE_UPDATE, ui::focus_system.system())
            .add_system(setup_game.system())
            .add_system(setup_finish.system())
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down.system());
    }
}

fn setup_game(
    commands: &mut Commands,
    (game_screen, _game, screen): (Res<crate::GameScreen>, Res<Game>, Res<Screen>),
    time: Res<Time>,
    asset_handles: Res<crate::AssetHandles>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");

        let game_handles = asset_handles.get_game_handles_unsafe();

        let planet = game_handles
            .planets
            .choose(&mut rand::thread_rng())
            .unwrap();

        let shift_left = -200.;

        commands
            .spawn(SpriteComponents {
                transform: Transform {
                    scale: Vec3::splat(0.10),
                    translation: Vec3::new(shift_left, 0., crate::Z_PLANET),
                    ..Default::default()
                },
                material: planet.clone(),
                ..Default::default()
            })
            .with(
                bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic()
                    .position(bevy_rapier2d::na::Isometry2::translation(shift_left, 0.))
                    .angvel(rand::thread_rng().gen_range(-1., 1.) * 0.2),
            )
            .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true))
            .with(ScreenTag);
        let planet = commands.current_entity().unwrap();

        let nb_moon = rand::thread_rng().gen_range(1, 4);

        for i in 0..nb_moon {
            let self_rotation = rand::thread_rng().gen_range(-1., 1.) * std::f32::consts::FRAC_PI_4;
            let orbiter = crate::space::Orbiter::every(
                rand::thread_rng().gen_range(0.01, 0.05),
                planet,
                if rand::thread_rng().gen_bool(0.5) {
                    crate::space::RotationDirection::Clockwise
                } else {
                    crate::space::RotationDirection::CounterClockwise
                },
                (i as f32 + 1.) * (300. / nb_moon as f32) + rand::thread_rng().gen_range(-20., 20.),
            )
            .self_rotate();
            let start_position =
                crate::space::target_position(time.seconds_since_startup as f32, &orbiter);

            commands
                .spawn(SpriteComponents {
                    transform: Transform {
                        scale: Vec3::splat(0.10),
                        translation: Vec3::new(
                            start_position.x + shift_left,
                            start_position.y,
                            crate::Z_MOON,
                        ),
                        ..Default::default()
                    },
                    material: game_handles
                        .orbiters
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .clone_weak(),
                    ..Default::default()
                })
                .with(orbiter)
                .with(
                    bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic()
                        .angvel(self_rotation)
                        .position(bevy_rapier2d::na::Isometry2::translation(
                            start_position.x + shift_left,
                            start_position.y,
                        )),
                )
                .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true))
                .with(crate::space::SpawnShip::every(
                    5.,
                    if self_rotation < 0. {
                        crate::space::RotationDirection::Clockwise
                    } else {
                        crate::space::RotationDirection::CounterClockwise
                    },
                ))
                .with_children(|p| {
                    p.spawn((crate::space::SpawnShipProgress,));
                })
                .with(ui::Interaction::None)
                .with(ui::InteractionBox {
                    size: Vec2::new(30., 30.),
                })
                .with(ScreenTag);
        }
    }
}

fn setup_finish(
    (game_screen, _game, mut screen): (Res<crate::GameScreen>, Res<Game>, ResMut<Screen>),
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");

        screen.loaded = true;
        screen.first_load = false;
    }
}

fn tear_down(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    query: Query<Entity, With<ScreenTag>>,
    ship_query: Query<Entity, With<crate::space::Ship>>,
) {
    if game_screen.current_screen != CURRENT_SCREEN && screen.loaded {
        info!("tear down");

        for entity in ship_query.iter() {
            commands.despawn_recursive(entity);
        }

        for entity in query.iter() {
            commands.despawn_recursive(entity);
        }

        screen.loaded = false;
    }
}

fn keyboard_input_system(
    mut game_screen: ResMut<crate::GameScreen>,
    screen: ResMut<Screen>,
    keyboard_input: Res<Input<KeyCode>>,
    mut wnds: ResMut<Windows>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && screen.loaded {
        if keyboard_input.just_released(KeyCode::Escape) {
            game_screen.current_screen = crate::Screen::Menu;
        } else if keyboard_input.just_released(KeyCode::F) {
            let window = wnds.get_primary_mut().unwrap();
            match window.mode() {
                bevy::window::WindowMode::Windowed => {
                    window.set_mode(bevy::window::WindowMode::BorderlessFullscreen)
                }
                _ => window.set_mode(bevy::window::WindowMode::Windowed),
            }
        }
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
    pub score: u32,
    pub selected: Option<Entity>,
    pub ratio: Ratio,
    pub targeted: Option<Entity>,
}

pub enum Ratio {
    ThreeQuarter,
    Half,
    OneQuarter,
    All,
}

impl Default for Ratio {
    fn default() -> Self {
        Ratio::ThreeQuarter
    }
}

impl Ratio {
    pub fn next(&mut self) {
        *self = match self {
            Ratio::ThreeQuarter => Ratio::Half,
            Ratio::Half => Ratio::OneQuarter,
            Ratio::OneQuarter => Ratio::All,
            Ratio::All => Ratio::ThreeQuarter,
        }
    }
}

#[derive(PartialEq)]
pub enum GameEvents {}

pub enum InterestingEvent {}
