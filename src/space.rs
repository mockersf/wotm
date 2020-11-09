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
pub enum Rotation {
    Free,
    Fixed(f32),
}

#[derive(Clone, Copy)]
pub struct Orbiter {
    pub speed: f32,
    pub offset: f32,
    direction: RotationDirection,
    pub distance: f32,
    pub around: Entity,
    pub rotation: Rotation,
}

impl Orbiter {
    pub fn every(speed: f32, around: Entity, distance: f32) -> Self {
        Self {
            speed,
            offset: rand::thread_rng().gen_range(0., 2. * std::f32::consts::PI),
            direction: RotationDirection::CounterClockwise,
            distance,
            around,
            rotation: Rotation::Free,
        }
    }

    pub fn self_rotate(mut self, speed: f32) -> Self {
        self.rotation = Rotation::Fixed(speed);

        self
    }
}

pub struct SpawnShip {
    every: Timer,
    scale: f32,
}

impl SpawnShip {
    pub fn every(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, true);
        timer.elapsed = 3. * duration / 4.;
        Self {
            every: timer,
            scale: 1.,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;

        self
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawn_ship.system())
            .add_system(orbite_around.system());
    }
}

fn spawn_ship(
    commands: &mut Commands,
    time: Res<Time>,
    asset_handles: Res<crate::AssetHandles>,
    mut query: Query<(&mut SpawnShip, &GlobalTransform, Entity)>,
) {
    for (mut spawn, global_transform, entity) in query.iter_mut() {
        let game_handles = asset_handles.get_game_handles_unsafe();
        spawn.every.tick(time.delta_seconds);
        if spawn.every.just_finished {
            let ship = game_handles.ships.choose(&mut rand::thread_rng()).unwrap();
            let orbiter = Orbiter::every(
                rand::thread_rng().gen_range(0.5, 1.),
                entity,
                spawn.scale * 50.,
            );

            let mut translation = global_transform.translation.clone();
            translation.set_z(crate::Z_SHIP);

            commands
                .spawn(SpriteComponents {
                    transform: Transform {
                        translation,
                        scale: Vec3::splat(spawn.scale * 0.05),
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
                .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(
                    spawn.scale * 5.,
                ))
                .with(orbiter)
                .with(Ship);
        }
    }
}

pub fn target_position(
    seconds: f32,
    orbiter: &crate::space::Orbiter,
) -> bevy_rapier2d::rapier::math::Vector<f32> {
    let target_x = (seconds * orbiter.speed + orbiter.offset).cos();
    let target_y = (seconds * orbiter.speed + orbiter.offset).sin();
    bevy_rapier2d::rapier::math::Vector::new(target_x, target_y) * orbiter.distance
}

fn orbite_around(
    time: Res<Time>,
    mut bodies: ResMut<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    orbiters: Query<(
        &bevy_rapier2d::physics::RigidBodyHandleComponent,
        &crate::space::Orbiter,
    )>,
    centers: Query<&GlobalTransform>,
) {
    for (rigid_body, orbiter) in orbiters.iter() {
        let mut body = bodies.get_mut(rigid_body.handle()).unwrap();

        let center_transform = centers.get(orbiter.around).unwrap();

        let (linvel, rot) = crate::space::go_from_to_rapier(
            body.position.translation.vector
                - bevy_rapier2d::rapier::math::Vector::new(
                    center_transform.translation.x(),
                    center_transform.translation.y(),
                ),
            target_position(time.seconds_since_startup as f32, orbiter),
        );
        body.linvel = linvel * orbiter.speed * orbiter.distance;
        body.angvel = 0.;
        match orbiter.rotation {
            Rotation::Free => {
                body.position.rotation = bevy_rapier2d::na::UnitComplex::from_angle(rot);
            }
            Rotation::Fixed(speed) => {
                body.position.rotation = bevy_rapier2d::na::UnitComplex::from_angle(
                    speed * time.seconds_since_startup as f32,
                );
            }
        }
    }
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
