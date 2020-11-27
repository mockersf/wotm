use bevy::prelude::*;
use rand::{prelude::IteratorRandom, seq::SliceRandom, Rng};
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
            .add_system(keyboard_input_system)
            .add_system(ui::ship_count)
            .add_system(ui::setup)
            .add_system(ui::interaction)
            .add_system(ui::ui_update)
            .add_system(ui::ui_update_on_interaction_event)
            .add_system(ui::orders)
            .add_system(ui::change_ratio_ui)
            .add_system_to_stage(bevy::app::stage::PRE_UPDATE, ui::focus_system)
            .add_system(setup_game)
            .add_system(setup_finish)
            .add_system(change_owner)
            .add_system(planet_defense)
            .add_system(asteroid_belt)
            .add_system(asteroid)
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down);
    }
}

pub struct Planet {
    pub name: String,
}

pub struct Moon {
    pub index: i32,
    pub planet: Entity,
}

fn setup_game(
    commands: &mut Commands,
    (game_screen, mut game, screen): (Res<crate::GameScreen>, ResMut<Game>, Res<Screen>),
    config: Res<crate::Config>,
    time: Res<Time>,
    asset_handles: Res<crate::AssetHandles>,
    mut events: ResMut<Events<ui::InteractionEvent>>,
) {
    game.elapsed += time.delta_seconds;
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");
        game.elapsed = 0.;

        let game_handles = asset_handles.get_game_handles_unsafe();

        let planet = game_handles
            .planets
            .choose(&mut rand::thread_rng())
            .unwrap();

        let shift_left = -200.;

        commands.spawn(SpriteBundle {
            transform: Transform {
                scale: Vec3::splat(0.10),
                translation: Vec3::new(shift_left, 0., crate::Z_PLANET),
                ..Default::default()
            },
            material: planet.0.clone(),
            ..Default::default()
        });
        let planet_entity = commands.current_entity().unwrap();
        commands
            .with(
                bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic()
                    .position(bevy_rapier2d::na::Isometry2::translation(shift_left, 0.))
                    .angvel(rand::thread_rng().gen_range(-1., 1.) * 0.2)
                    .user_data(planet_entity.to_bits() as u128),
            )
            .with(
                bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(
                    planet.1 as f32 / 10. * 9. / 10.,
                )
                .sensor(true),
            )
            .with(Planet {
                name: asset_handles
                    .get_planet_names()
                    .choose(&mut rand::thread_rng())
                    .unwrap()
                    .to_string(),
            })
            .with(ui::Interaction::None)
            .with(ui::InteractionBox {
                radius: planet.1 as f32 / 10. + 5.,
            })
            .with(OwnedBy::Neutral)
            .with(PlanetFleet::new(&config))
            .with(AsteroidBelt::new(&config))
            .with(ScreenTag);
        let planet = commands.current_entity().unwrap();

        let nb_moon = 3; //rand::thread_rng().gen_range(2, 4);

        let player_start_moon = rand::thread_rng().gen_range(0, nb_moon);

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
                (i as f32 + 1.) * (300. / nb_moon as f32) + rand::thread_rng().gen_range(0., 30.),
            )
            .self_rotate();
            let start_position =
                crate::space::target_orbiting_position(time.seconds_since_startup as f32, &orbiter);

            commands
                .spawn(SpriteBundle {
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
                .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true));
            let rot = if self_rotation < 0. {
                crate::space::RotationDirection::Clockwise
            } else {
                crate::space::RotationDirection::CounterClockwise
            };
            if player_start_moon == i {
                commands.with(crate::space::SpawnShipType::Basic.to_components(rot));
            } else {
                commands.with(crate::space::SpawnShipType::Neutral.to_components(rot));
            }
            let entity = commands.current_entity().unwrap();
            commands.with(
                bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic()
                    .angvel(self_rotation)
                    .position(bevy_rapier2d::na::Isometry2::translation(
                        start_position.x + shift_left,
                        start_position.y,
                    ))
                    .user_data(entity.to_bits() as u128),
            );
            commands
                .with_children(|p| {
                    p.spawn((crate::space::SpawnShipProgress,));
                })
                .with(ui::Interaction::None)
                .with(ui::InteractionBox { radius: 30. })
                .with(Moon {
                    index: i + 1,
                    planet,
                })
                .with(if player_start_moon == i {
                    OwnedBy::Player(0)
                } else {
                    OwnedBy::Neutral
                })
                .with(ScreenTag);
            if player_start_moon == i {
                events.send(ui::InteractionEvent::Clicked(Some(
                    commands.current_entity().unwrap(),
                )));
            }
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OwnedBy {
    Neutral,
    Player(usize),
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
    pub elapsed: f32,
    pub ship_counts: std::collections::HashMap<Entity, std::collections::HashMap<OwnedBy, usize>>,
}

#[derive(Copy, Clone, Debug)]
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

impl std::fmt::Display for Ratio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ratio::All => write!(f, "100%"),
            Ratio::ThreeQuarter => write!(f, "75%"),
            Ratio::Half => write!(f, "50%"),
            Ratio::OneQuarter => write!(f, "25%"),
        }
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

    pub fn as_usize(&self) -> usize {
        match self {
            Ratio::All => 4,
            Ratio::ThreeQuarter => 3,
            Ratio::Half => 2,
            Ratio::OneQuarter => 1,
        }
    }
    pub fn from_usize(i: usize) -> Self {
        match i {
            1 => Ratio::OneQuarter,
            2 => Ratio::Half,
            3 => Ratio::ThreeQuarter,
            _ => Ratio::All,
        }
    }
    pub fn of(&self, i: usize) -> usize {
        if i == 0 {
            return 0;
        }
        1.max(
            (i as f32
                * match self {
                    Ratio::ThreeQuarter => 0.75,
                    Ratio::Half => 0.5,
                    Ratio::OneQuarter => 0.25,
                    Ratio::All => 1.,
                }) as usize,
        )
    }
}

#[derive(PartialEq)]
pub enum GameEvents {
    ShipDamaged(Entity, i32),
    PlanetShield(Entity),
    MoonConquered(Entity, OwnedBy),
    PlanetConquered(Entity),
}

pub enum InterestingEvent {}

fn change_owner(
    mut game_events: ResMut<Events<crate::game::GameEvents>>,
    game: Res<Game>,
    query_moon: Query<(Entity, &OwnedBy), With<crate::game::Moon>>,
) {
    for (entity, owner) in query_moon.iter() {
        let owners = game.ship_counts.get(&entity).unwrap().keys();

        if owners.len() == 1 {
            let new_owner = owners.into_iter().next().unwrap();
            if *owner != *new_owner {
                game_events.send(GameEvents::MoonConquered(entity, new_owner.clone()));
            }
        }
    }
}

pub struct PlanetFleet {
    timer: Timer,
    last_happened: f32,
    iteration: f32,
}
impl PlanetFleet {
    pub fn new(config: &crate::Config) -> Self {
        let mut timer = Timer::from_seconds(config.fleet_timer, true);
        timer.elapsed = -config.fleet_delay;
        Self {
            timer,
            last_happened: -config.fleet_delay,
            iteration: 0.,
        }
    }
}

pub fn planet_defense(
    commands: &mut Commands,
    time: Res<Time>,
    config: Res<crate::Config>,
    mut game_screen: ResMut<crate::GameScreen>,
    asset_handles: Res<crate::AssetHandles>,
    mut planet_fleet: Query<(Entity, &GlobalTransform, &mut PlanetFleet)>,
    moons: Query<(Entity, &OwnedBy), With<Moon>>,
) {
    for (planet, gt, mut fleet) in planet_fleet.iter_mut() {
        fleet.timer.tick(time.delta_seconds);
        fleet.last_happened += time.delta_seconds;
        if fleet.timer.just_finished {
            let mut override_chance = None;
            let mut override_min_health = None;
            let mut override_max_health = None;
            let neutral_moons = moons
                .iter()
                .filter(|(_, moon_owner)| **moon_owner == OwnedBy::Neutral)
                .count();
            if neutral_moons == 0 {
                override_chance = Some(config.fleet_chance * 2.);
                override_min_health = Some(0);
                override_max_health = Some(4);
            }

            if rand::thread_rng().gen_bool(override_chance.unwrap_or(config.fleet_chance) as f64) {
                let game_handles = asset_handles.get_game_handles_unsafe();

                let ship = game_handles.ships[2]
                    .choose(&mut rand::thread_rng())
                    .unwrap();
                let mut translation = gt.translation.clone();
                translation.z = crate::Z_SHIP;
                let player_moons = moons
                    .iter()
                    .filter(|(_, moon_owner)| **moon_owner == OwnedBy::Player(0))
                    .count();
                if player_moons == 0 {
                    game_screen.current_screen = crate::Screen::Lost;
                }
                let mut hit_points_to_spawn = ((config.fleet_chance
                    * (fleet.last_happened / fleet.timer.duration + fleet.iteration)
                    / 2.5) as i32
                    * player_moons as i32)
                    - 1;
                let mut i = -0.2;
                while hit_points_to_spawn > 0 {
                    let spawn_hit_points = 0.max(rand::thread_rng().gen_range(
                        override_min_health.unwrap_or(-4),
                        override_max_health.unwrap_or(2),
                    ));
                    let moon = moons.iter().choose(&mut rand::thread_rng()).unwrap();
                    let scale = (spawn_hit_points as f32 + 3.) / 4.;
                    commands.spawn(SpriteBundle {
                        transform: Transform {
                            translation,
                            scale: Vec3::splat(0.15 * scale),
                            ..Default::default()
                        },
                        material: ship.clone(),
                        ..Default::default()
                    });
                    let entity = commands.current_entity().unwrap();
                    commands
                        .with(
                            bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic()
                                .translation(gt.translation.x + i, gt.translation.y + i)
                                .user_data(entity.to_bits() as u128),
                        )
                        .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(
                            5. * scale,
                        ));
                    commands
                        .with(crate::space::MoveTowards {
                            speed: 2000.,
                            from: planet,
                            towards: moon.0,
                        })
                        .with(crate::game::OwnedBy::Neutral)
                        .with(crate::space::Ship {
                            hit_points: spawn_hit_points,
                        });
                    hit_points_to_spawn -= spawn_hit_points;
                    i += 0.1;
                }
                fleet.last_happened = 0.;
                fleet.iteration += 1.;
            }
        }
    }
}

pub struct AsteroidBelt {
    timer: Timer,
}
impl AsteroidBelt {
    pub fn new(config: &crate::Config) -> Self {
        Self {
            timer: Timer::from_seconds(config.asteroid_timer, true),
        }
    }
}

pub fn asteroid_belt(
    commands: &mut Commands,
    time: Res<Time>,
    game: Res<Game>,
    config: Res<crate::Config>,
    asset_handles: Res<crate::AssetHandles>,
    mut asteroids: Query<&mut AsteroidBelt>,
    moons: Query<&GlobalTransform, With<Moon>>,
) {
    for mut asteroid in asteroids.iter_mut() {
        asteroid.timer.tick(time.delta_seconds);
        if asteroid.timer.just_finished {
            if rand::thread_rng().gen_bool(config.asteroid_chance as f64) {
                let game_handles = asset_handles.get_game_handles_unsafe();
                let meteor = game_handles
                    .meteors
                    .choose(&mut rand::thread_rng())
                    .unwrap();
                let (start_x, start_y) = match rand::thread_rng().gen_range(0, 5) {
                    0 => (-700., rand::thread_rng().gen_range(-400., 400.)),
                    1 => (700., rand::thread_rng().gen_range(-400., 400.)),
                    2 => (rand::thread_rng().gen_range(-700., 700.), -400.),
                    _ => (rand::thread_rng().gen_range(-700., 700.), 400.),
                };

                let translation = Vec3::new(start_x, start_y, crate::Z_SHIP);

                let target = game
                    .ship_counts
                    .iter()
                    .map(|(entity, counts)| (entity, counts.iter().fold(0, |acc, (_, c)| acc + c)))
                    .max_by_key(|(_, c)| *c)
                    .unwrap()
                    .0;
                let target = moons.get(*target).unwrap();

                commands.spawn(SpriteBundle {
                    transform: Transform {
                        translation,
                        scale: Vec3::splat(0.4),
                        ..Default::default()
                    },
                    material: meteor.clone(),
                    ..Default::default()
                });
                let entity = commands.current_entity().unwrap();
                let a = bevy_rapier2d::rapier::math::Vector::new(
                    -start_x + target.translation.x,
                    -start_y + target.translation.y,
                )
                .normalize()
                    * rand::thread_rng().gen_range(175., 250.);
                commands
                    .with(
                        bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic()
                            .translation(translation.x, translation.y)
                            .user_data(entity.to_bits() as u128)
                            .angvel(rand::thread_rng().gen_range(-1., 1.))
                            .linvel(a.x, a.y),
                    )
                    .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.).sensor(true));
                commands
                    .with(Asteroid(Timer::from_seconds(40., false)))
                    .with(ScreenTag);
            }
        }
    }
}

pub struct Asteroid(Timer);

pub fn asteroid(
    commands: &mut Commands,
    time: Res<Time>,
    mut asteroids: Query<(Entity, &mut Asteroid)>,
) {
    for (entity, mut asteroid) in asteroids.iter_mut() {
        asteroid.0.tick(time.delta_seconds);
        if asteroid.0.just_finished {
            commands.despawn_recursive(entity);
        }
    }
}
