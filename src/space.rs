use bevy::prelude::*;
use rand::{seq::SliceRandom, Rng};

pub struct Ship {
    pub hit_points: i32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
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
    pub direction: RotationDirection,
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

pub enum SpawnShipType {
    Neutral,
    Basic,
    // Small,
    // Large,
}

impl SpawnShipType {
    pub fn to_components(&self, rotation_direction: RotationDirection) -> SpawnShip {
        let base_delay = 5.;
        let base_hit_points = 2;
        match self {
            SpawnShipType::Neutral => SpawnShip {
                every: Timer::from_seconds(base_delay * 1.2, true),
                scale: 1.,
                rotation_direction,
                hit_points: base_hit_points,
            },
            SpawnShipType::Basic => SpawnShip {
                every: Timer::from_seconds(base_delay, true),
                scale: 1.,
                rotation_direction,
                hit_points: base_hit_points,
            },
            // SpawnShipType::Small => SpawnShip {
            //     every: Timer::from_seconds(base_delay / 2., true),
            //     scale: 1.,
            //     rotation_direction,
            //     hit_points: base_hit_points / 2,
            // },
            // SpawnShipType::Large => SpawnShip {
            //     every: Timer::from_seconds(base_delay * 2., true),
            //     scale: 1.,
            //     rotation_direction,
            //     hit_points: base_hit_points * 2,
            // },
        }
    }
}

#[derive(Debug)]
pub struct SpawnShip {
    pub every: Timer,
    pub scale: f32,
    pub rotation_direction: RotationDirection,
    pub hit_points: i32,
}

impl SpawnShip {
    pub fn every(duration: f32, rotation_direction: RotationDirection) -> Self {
        Self {
            every: Timer::from_seconds(duration, true),
            scale: 1.,
            rotation_direction,
            hit_points: 1,
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
            .add_system(move_towards)
            .add_system(ship_collision)
            .add_system(object_collision)
            .add_system(game_events)
            .add_system(explode)
            .add_system(manage_shield);
    }
}
pub struct SpawnShipProgress;

fn spawn_ship(
    commands: &mut Commands,
    time: Res<Time>,
    config: Res<crate::Config>,
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
            let ship = game_handles.ships[match owned_by {
                crate::game::OwnedBy::Player(i) => *i,
                crate::game::OwnedBy::Neutral => 2,
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

            let lucky_draw = match (
                owned_by,
                rand::thread_rng().gen_bool(config.bigger_player_ship_rate),
            ) {
                (crate::game::OwnedBy::Player(0), true) => config.bigger_player_ship_change,
                _ => 1.,
            };

            commands.spawn(SpriteBundle {
                transform: Transform {
                    translation,
                    scale: Vec3::splat(spawn.scale * 0.15 * (lucky_draw)),
                    ..Default::default()
                },
                material: ship.clone(),
                ..Default::default()
            });
            let entity = commands.current_entity().unwrap();
            commands
                .with(
                    bevy_rapier2d::rapier::dynamics::RigidBodyBuilder::new_dynamic()
                        .translation(
                            global_transform.translation.x,
                            global_transform.translation.y,
                        )
                        .user_data(entity.to_bits() as u128),
                )
                .with(bevy_rapier2d::rapier::geometry::ColliderBuilder::ball(
                    spawn.scale * 5. * (lucky_draw),
                ));
            commands.with(orbiter).with(owned_by.clone()).with(Ship {
                hit_points: (spawn.hit_points as f32 * lucky_draw).ceil() as i32,
            });
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
                        rand::thread_rng().gen_range(0.8, 1.2),
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

pub fn ship_collision(
    events: Res<bevy_rapier2d::physics::EventQueue>,
    bodies: Res<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    colliders: Res<bevy_rapier2d::rapier::geometry::ColliderSet>,
    mut game_events: ResMut<Events<crate::game::GameEvents>>,
    ship_owner: Query<(&crate::game::OwnedBy, &crate::space::Ship)>,
) {
    let mut removed = std::collections::HashSet::new();
    while let Ok(contact_event) = events.contact_events.pop() {
        match contact_event {
            bevy_rapier2d::rapier::ncollide::pipeline::narrow_phase::ContactEvent::Started(
                h1,
                h2,
            ) => {
                let entity1 = Entity::from_bits(
                    bodies
                        .get(colliders.get(h1).unwrap().parent())
                        .unwrap()
                        .user_data as u64,
                );
                if removed.contains(&entity1) {
                    continue;
                }
                let entity2 = Entity::from_bits(
                    bodies
                        .get(colliders.get(h2).unwrap().parent())
                        .unwrap()
                        .user_data as u64,
                );
                if removed.contains(&entity2) {
                    continue;
                }
                if let Ok((owner1, ship1)) = ship_owner.get(entity1) {
                    if let Ok((owner2, ship2)) = ship_owner.get(entity2) {
                        if owner1 != owner2 {
                            game_events.send(crate::game::GameEvents::ShipDamaged(
                                entity1,
                                1.min(ship2.hit_points),
                            ));
                            removed.insert(entity1);
                            game_events.send(crate::game::GameEvents::ShipDamaged(
                                entity2,
                                1.min(ship1.hit_points),
                            ));
                            removed.insert(entity2);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

pub fn game_events(
    commands: &mut Commands,
    mut game_screen: ResMut<crate::GameScreen>,
    (mut event_reader, events): (
        Local<EventReader<crate::game::GameEvents>>,
        Res<Events<crate::game::GameEvents>>,
    ),
    mut asset_handles: ResMut<crate::AssetHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ship_info: Query<(&mut Ship, &GlobalTransform), With<crate::space::Ship>>,
    mut query_moon: Query<(&crate::space::SpawnShip, &mut crate::game::OwnedBy)>,
    mut query_ships: Query<
        (&mut crate::space::Orbiter, &crate::game::OwnedBy),
        With<crate::space::Ship>,
    >,
    query_interaction_box: Query<&crate::game::ui::InteractionBox>,
) {
    for event in event_reader.iter(&events) {
        match event {
            crate::game::GameEvents::ShipDamaged(entity, damage) => {
                if let Ok((mut ship, gt)) = ship_info.get_mut(*entity) {
                    ship.hit_points -= damage;
                    if ship.hit_points <= 0 {
                        commands.despawn_recursive(*entity);
                        let explosion_handle =
                            asset_handles.get_game_handles_unsafe().explosion_handle;
                        commands
                            .spawn(SpriteSheetBundle {
                                sprite: TextureAtlasSprite {
                                    index: 0,
                                    ..Default::default()
                                },
                                texture_atlas: explosion_handle.clone(),
                                transform: (*gt).into(),
                                ..Default::default()
                            })
                            .with(Explosion {
                                timer: Timer::from_seconds(0.1, true),
                            });
                    }
                }
            }
            crate::game::GameEvents::MoonConquered(entity, new_owner) => {
                if let Ok((spawnship, mut owner)) = query_moon.get_mut(*entity) {
                    if let crate::game::OwnedBy::Player(0) = *new_owner {
                        commands.insert_one(
                            *entity,
                            SpawnShipType::Basic.to_components(spawnship.rotation_direction),
                        );
                    } else {
                        commands.insert_one(
                            *entity,
                            SpawnShipType::Neutral.to_components(spawnship.rotation_direction),
                        );
                    }
                    *owner = new_owner.clone();
                    query_ships
                        .iter_mut()
                        .filter(|(orbiter, owned_by)| {
                            orbiter.around == *entity && *owned_by == new_owner
                        })
                        .for_each(|(mut orbiter, _)| {
                            orbiter.direction = spawnship.rotation_direction
                        });
                }
            }
            crate::game::GameEvents::PlanetShield(entity) => {
                if let Ok(interaction_box) = query_interaction_box.get(*entity) {
                    let shield_color = asset_handles.get_color_spawning_enemy(&mut materials);
                    let radius = interaction_box.radius * 10. - 20.;

                    let mut builder = bevy_prototype_lyon::path::PathBuilder::new();
                    builder.move_to(bevy_prototype_lyon::prelude::point(0., radius));
                    builder.arc(
                        bevy_prototype_lyon::prelude::point(0., 0.),
                        radius,
                        radius,
                        2. * std::f32::consts::PI,
                        0.,
                    );
                    let path = builder.build();
                    let sprite = path.stroke(
                        shield_color,
                        &mut meshes,
                        Vec3::new(0.0, 0.0, 0.0),
                        &bevy_prototype_lyon::prelude::StrokeOptions::default()
                            .with_line_width(20.0)
                            .with_line_cap(bevy_prototype_lyon::prelude::LineCap::Round)
                            .with_line_join(bevy_prototype_lyon::prelude::LineJoin::Round),
                    );
                    let shield = commands
                        .spawn(sprite)
                        .with(Shield(Timer::from_seconds(0.5, false)))
                        .current_entity()
                        .unwrap();
                    commands.push_children(*entity, &[shield]);
                }
            }
            crate::game::GameEvents::PlanetConquered(_) => {
                game_screen.current_screen = crate::Screen::Win;
            }
        }
    }
}

pub struct Shield(Timer);

pub fn manage_shield(
    commands: &mut Commands,
    time: Res<Time>,
    mut query_shields: Query<(Entity, &mut Shield)>,
) {
    for (entity, mut shield) in query_shields.iter_mut() {
        shield.0.tick(time.delta_seconds);
        if shield.0.just_finished {
            commands.despawn_recursive(entity);
        }
    }
}

pub fn object_collision(
    events: Res<bevy_rapier2d::physics::EventQueue>,
    bodies: Res<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    colliders: Res<bevy_rapier2d::rapier::geometry::ColliderSet>,
    mut game_events: ResMut<Events<crate::game::GameEvents>>,
    ship_owner: Query<&crate::game::OwnedBy, With<crate::space::Ship>>,
    planet_owner: Query<&crate::game::OwnedBy, With<crate::game::Planet>>,
    moon_owner: Query<&crate::game::OwnedBy, With<crate::game::Moon>>,
    asteroid: Query<&crate::game::Asteroid>,
) {
    while let Ok(event) = events.proximity_events.pop() {
        let entity1 = Entity::from_bits(
            bodies
                .get(colliders.get(event.collider1).unwrap().parent())
                .unwrap()
                .user_data as u64,
        );
        let entity2 = Entity::from_bits(
            bodies
                .get(colliders.get(event.collider2).unwrap().parent())
                .unwrap()
                .user_data as u64,
        );
        if let bevy_rapier2d::rapier::ncollide::query::Proximity::Intersecting = event.new_status {
            let (ship, asteroid) =
                match (asteroid.get(entity1).is_ok(), asteroid.get(entity2).is_ok()) {
                    (true, _) => (entity2, true),
                    (_, true) => (entity1, true),
                    _ => (entity1, false),
                };
            if asteroid {
                game_events.send(crate::game::GameEvents::ShipDamaged(ship, 500));
            }
            let (ship, planet) = match (
                planet_owner.get(entity1).is_ok(),
                planet_owner.get(entity2).is_ok(),
            ) {
                (true, _) => (entity2, entity1),
                (_, true) => (entity1, entity2),
                _ => continue,
            };
            if let Ok(crate::game::OwnedBy::Player(0)) = ship_owner.get(ship) {
                if moon_owner
                    .iter()
                    .filter(|owner| **owner != crate::game::OwnedBy::Player(0))
                    .count()
                    != 0
                {
                    game_events.send(crate::game::GameEvents::ShipDamaged(ship, 500));
                    game_events.send(crate::game::GameEvents::PlanetShield(planet));
                } else {
                    game_events.send(crate::game::GameEvents::PlanetConquered(planet));
                }
            }
        }
    }
}

struct Explosion {
    timer: Timer,
}
fn explode(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(&mut Explosion, &mut TextureAtlasSprite, Entity)>,
) {
    for (mut explosion, mut atlas_sprite, entity) in query.iter_mut() {
        explosion.timer.tick(time.delta_seconds);
        if explosion.timer.just_finished {
            match atlas_sprite.index {
                6 => {
                    commands.despawn_recursive(entity);
                }
                n => atlas_sprite.index = n + 1,
            }
        }
    }
}
