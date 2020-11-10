use bevy::prelude::*;
use rand::{seq::SliceRandom, Rng};

pub struct Ship;

#[derive(Clone, Copy, PartialEq)]
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
    Fixed,
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
    pub fn every(speed: f32, around: Entity, direction: RotationDirection, distance: f32) -> Self {
        Self {
            speed,
            offset: rand::thread_rng().gen_range(0., 2. * std::f32::consts::PI),
            direction,
            distance,
            around,
            rotation: Rotation::Free,
        }
    }

    pub fn self_rotate(mut self) -> Self {
        self.rotation = Rotation::Fixed;

        self
    }
}

pub struct SpawnShip {
    every: Timer,
    scale: f32,
    rotation_direction: RotationDirection,
}

impl SpawnShip {
    pub fn every(duration: f32, rotation_direction: RotationDirection) -> Self {
        let mut timer = Timer::from_seconds(duration, true);
        timer.elapsed = 3. * duration / 4.;
        Self {
            every: timer,
            scale: 1.,
            rotation_direction,
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
pub struct SpawnShipProgress;

fn spawn_ship(
    commands: &mut Commands,
    time: Res<Time>,
    asset_handles: Res<crate::AssetHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    bodies: Res<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    mut query: Query<(
        &mut SpawnShip,
        &GlobalTransform,
        Entity,
        &Children,
        Option<&bevy_rapier2d::physics::RigidBodyHandleComponent>,
    )>,
    progress_query: Query<With<SpawnShipProgress, Entity>>,
) {
    let red = materials.add(Color::rgb(0., 100.0, 0.0).into());

    for (mut spawn, global_transform, entity, children, rigid_body) in query.iter_mut() {
        let game_handles = asset_handles.get_game_handles_unsafe();
        spawn.every.tick(time.delta_seconds);

        if let Some(progress_entity) = children
            .iter()
            .find_map(|entity| progress_query.get(*entity).ok())
        {
            if let Some(rigid_body) = rigid_body {
                let body = bodies.get(rigid_body.handle()).unwrap();

                let angle = spawn.every.elapsed / spawn.every.duration * 2. * std::f32::consts::PI;

                let radius = 300.;
                let start_x =
                    (-body.position.rotation.angle() + std::f32::consts::FRAC_PI_2).cos() * radius;
                let start_y =
                    (-body.position.rotation.angle() + std::f32::consts::FRAC_PI_2).sin() * radius;
                let mut builder = bevy_prototype_lyon::path::PathBuilder::new();
                builder.move_to(bevy_prototype_lyon::prelude::point(start_x, start_y));
                builder.arc(
                    bevy_prototype_lyon::prelude::point(0., 0.),
                    radius,
                    radius,
                    angle,
                    0.,
                );
                let path = builder.build();
                let sprite = path.stroke(
                    red.clone(),
                    &mut meshes,
                    Vec3::new(0.0, 0.0, 0.0),
                    &bevy_prototype_lyon::prelude::StrokeOptions::default()
                        .with_line_width(10.0)
                        .with_line_cap(bevy_prototype_lyon::prelude::LineCap::Round)
                        .with_line_join(bevy_prototype_lyon::prelude::LineJoin::Round),
                );

                commands.insert(progress_entity, sprite);
            }
        }

        if spawn.every.just_finished {
            let ship = game_handles.ships.choose(&mut rand::thread_rng()).unwrap();
            let orbiter = Orbiter::every(
                rand::thread_rng().gen_range(0.5, 1.),
                entity,
                spawn.rotation_direction,
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
    let sign = if orbiter.direction == RotationDirection::Clockwise {
        1.
    } else {
        -1.
    };
    let target_x = (sign * seconds * orbiter.speed + orbiter.offset).cos();
    let target_y = (sign * seconds * orbiter.speed + orbiter.offset).sin();
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
        match orbiter.rotation {
            Rotation::Free => {
                body.angvel = 0.;
                body.position.rotation = bevy_rapier2d::na::UnitComplex::from_angle(rot);
            }
            Rotation::Fixed => (),
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
