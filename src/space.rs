use bevy::prelude::*;
use rand::{seq::SliceRandom, Rng};

pub struct Ship;

#[derive(Clone, Copy)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

impl RotationDirection {
    pub fn to_factor(&self) -> f32 {
        match self {
            RotationDirection::Clockwise => -1.,
            RotationDirection::CounterClockwise => 1.,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Orbiter {
    pub speed: f32,
    pub offset: f32,
    direction: RotationDirection,
    pub distance: f32,
    pub center_x: f32,
    pub center_y: f32,
}

impl Orbiter {
    pub fn every(speed: f32, center_x: f32, center_y: f32, distance: f32) -> Self {
        Self {
            speed,
            offset: rand::thread_rng().gen_range(0., 2. * std::f32::consts::PI),
            direction: RotationDirection::CounterClockwise,
            distance,
            center_x,
            center_y,
        }
    }
}

pub struct SpawnShip {
    every: Timer,
}

impl SpawnShip {
    pub fn every(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, true);
        timer.elapsed = 3. * duration / 4.;
        Self { every: timer }
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawn_ship.system());
    }
}

fn spawn_ship(
    mut commands: Commands,
    time: Res<Time>,
    asset_handles: Res<crate::AssetHandles>,
    mut spawn: Mut<SpawnShip>,
    global_transform: &GlobalTransform,
) {
    let game_handles = asset_handles.get_game_handles_unsafe();
    spawn.every.tick(time.delta_seconds);
    if spawn.every.just_finished {
        let ship = game_handles.ships.choose(&mut rand::thread_rng()).unwrap();
        let orbiter = Orbiter::every(
            rand::thread_rng().gen_range(0.5, 1.),
            global_transform.translation.x(),
            global_transform.translation.y(),
            100.,
        );

        commands
            .spawn(SpriteComponents {
                transform: Transform {
                    translation: global_transform.translation,
                    scale: Vec3::splat(0.10),
                    ..Default::default()
                },
                material: ship.clone(),
                ..Default::default()
            })
            .with(
                bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic().translation(
                    global_transform.translation.x(),
                    global_transform.translation.y(),
                ),
            )
            .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(10.))
            .with(orbiter)
            .with(Ship);
    }
}

pub fn go_from_to(from: Vec2, to: Vec2) -> (Vec2, f32) {
    (to - from, from.angle_between(to))
}

use bevy_rapier2d::rapier::math::Vector;
pub fn go_from_to_rapier(from: Vector<f32>, to: Vector<f32>) -> (Vector<f32>, f32) {
    (
        (to - from).normalize(),
        if to.x < 0. {
            to.angle(&Vector::new(0., 1.))
        } else {
            -to.angle(&Vector::new(0., 1.))
        } - std::f32::consts::FRAC_PI_2,
    )
}
