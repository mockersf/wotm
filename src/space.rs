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
    pub fn opposite(&self) -> Self {
        match self {
            RotationDirection::Clockwise => RotationDirection::CounterClockwise,
            RotationDirection::CounterClockwise => RotationDirection::Clockwise,
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

const SHIP_ORBITING_DISTANCE: f32 = 50.;

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
    pub fn every_with_offset(
        speed: f32,
        around: Entity,
        direction: RotationDirection,
        distance: f32,
        offset: f32,
    ) -> Self {
        Self {
            speed,
            offset,
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

#[derive(Clone, Copy, Debug)]
pub struct MoveTowards {
    pub speed: f32,
    pub towards: Entity,
    pub from: Entity,
}

pub struct SpawnShip {
    every: Timer,
    scale: f32,
    rotation_direction: RotationDirection,
}

impl SpawnShip {
    pub fn every(duration: f32, rotation_direction: RotationDirection) -> Self {
        Self {
            every: Timer::from_seconds(duration, true),
            scale: 1.,
            rotation_direction,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;

        self
    }

    pub fn with_headstart(mut self) -> Self {
        self.every.elapsed = 3. * self.every.duration / 4.;

        self
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(spawn_ship)
            .add_system(orbite_around)
            .add_system(move_towards);
    }
}
pub struct SpawnShipProgress;

fn spawn_ship(
    commands: &mut Commands,
    time: Res<Time>,
    mut asset_handles: ResMut<crate::AssetHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    bodies: Res<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    mut query: Query<(
        &mut SpawnShip,
        &GlobalTransform,
        Entity,
        &Children,
        &crate::game::OwnedBy,
        Option<&bevy_rapier2d::physics::RigidBodyHandleComponent>,
    )>,
    progress_query: Query<Entity, With<SpawnShipProgress>>,
) {
    for (mut spawn, global_transform, entity, children, owned_by, rigid_body) in query.iter_mut() {
        let game_handles = asset_handles.get_game_handles_unsafe();
        spawn.every.tick(time.delta_seconds);

        if let Some(progress_entity) = children
            .iter()
            .find_map(|entity| progress_query.get(*entity).ok())
        {
            if let Some(rigid_body) = rigid_body {
                let color_spawn_progress = match owned_by {
                    crate::game::OwnedBy::Player(0) => {
                        asset_handles.get_color_spawning_self(&mut materials)
                    }
                    crate::game::OwnedBy::Player(_) => {
                        asset_handles.get_color_spawning_enemy(&mut materials)
                    }
                    crate::game::OwnedBy::Neutral => {
                        asset_handles.get_color_spawning_neutral(&mut materials)
                    }
                };
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
                    color_spawn_progress.clone(),
                    &mut meshes,
                    Vec3::new(0.0, 0.0, 0.0),
                    &bevy_prototype_lyon::prelude::StrokeOptions::default()
                        .with_line_width(20.)
                        .with_line_cap(bevy_prototype_lyon::prelude::LineCap::Round)
                        .with_line_join(bevy_prototype_lyon::prelude::LineJoin::Round),
                );

                commands.insert(progress_entity, sprite);
            }
        }

        if spawn.every.just_finished {
            if let crate::game::OwnedBy::Neutral = owned_by {
                spawn.every.duration *= 1.02;
            }
            let ship = game_handles.ships[match owned_by {
                crate::game::OwnedBy::Player(i) => *i,
                crate::game::OwnedBy::Neutral => 3,
            }]
            .choose(&mut rand::thread_rng())
            .unwrap();
            let orbiter = Orbiter::every(
                rand::thread_rng().gen_range(0.5, 1.),
                entity,
                spawn.rotation_direction,
                spawn.scale * SHIP_ORBITING_DISTANCE,
            );

            let mut translation = global_transform.translation.clone();
            translation.z = crate::Z_SHIP;

            commands
                .spawn(SpriteBundle {
                    transform: Transform {
                        translation,
                        scale: Vec3::splat(spawn.scale * 0.15),
                        ..Default::default()
                    },
                    material: ship.clone(),
                    ..Default::default()
                })
                .with(
                    bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic().translation(
                        global_transform.translation.x,
                        global_transform.translation.y,
                    ),
                )
                .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(
                    spawn.scale * 5.,
                ))
                .with(orbiter)
                .with(owned_by.clone())
                .with(Ship);
        }
    }
}

pub fn target_orbiting_position(
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
    orbiters: Query<(&bevy_rapier2d::physics::RigidBodyHandleComponent, &Orbiter)>,
    centers: Query<&GlobalTransform>,
) {
    for (rigid_body, orbiter) in orbiters.iter() {
        let mut body = bodies.get_mut(rigid_body.handle()).unwrap();

        let center_transform = centers.get(orbiter.around).unwrap();

        let (linvel, rot) = crate::space::go_from_to_rapier(
            body.position.translation.vector
                - bevy_rapier2d::rapier::math::Vector::new(
                    center_transform.translation.x,
                    center_transform.translation.y,
                ),
            target_orbiting_position(time.seconds_since_startup as f32, orbiter),
        );
        body.linvel = linvel * orbiter.speed * orbiter.distance;
        match orbiter.rotation {
            Rotation::Free => {
                body.angvel = 0.;
                body.position.rotation = match orbiter.direction {
                    RotationDirection::Clockwise => bevy_rapier2d::na::UnitComplex::from_angle(rot),
                    RotationDirection::CounterClockwise => {
                        bevy_rapier2d::na::UnitComplex::from_angle(rot + std::f32::consts::PI)
                    }
                };
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

fn move_towards(
    commands: &mut Commands,
    time: Res<Time>,
    mut bodies: ResMut<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    movers: Query<(
        Entity,
        &bevy_rapier2d::physics::RigidBodyHandleComponent,
        &MoveTowards,
        &crate::game::OwnedBy,
    )>,
    centers: Query<&GlobalTransform>,
    target_spawn: Query<(&SpawnShip, &crate::game::OwnedBy)>,
) {
    for (moving, rigid_body, towards, owned_by) in movers.iter() {
        let mut body = bodies.get_mut(rigid_body.handle()).unwrap();

        let target = centers.get(towards.towards).unwrap();
        let origin = centers.get(moving).unwrap();

        let (linvel, rot) = crate::space::go_from_to_rapier(
            bevy_rapier2d::rapier::math::Vector::new(0., 0.),
            bevy_rapier2d::rapier::math::Vector::new(
                target.translation.x - origin.translation.x,
                target.translation.y - origin.translation.y,
            ),
        );

        if let Ok((spawn, moon_owned_by)) = target_spawn.get(towards.towards) {
            if origin.translation.distance(target.translation)
                < spawn.scale * SHIP_ORBITING_DISTANCE
            {
                commands.remove_one::<MoveTowards>(moving);
                commands.insert_one(
                    moving,
                    Orbiter::every_with_offset(
                        rand::thread_rng().gen_range(0.5, 1.),
                        towards.towards,
                        if owned_by == moon_owned_by {
                            spawn.rotation_direction
                        } else {
                            spawn.rotation_direction.opposite()
                        },
                        spawn.scale * SHIP_ORBITING_DISTANCE,
                        rot,
                    ),
                );
            }
        }

        body.linvel = linvel * towards.speed * time.delta_seconds;
        body.position.rotation =
            bevy_rapier2d::na::UnitComplex::from_angle(rot - std::f32::consts::FRAC_PI_2);
    }
}
