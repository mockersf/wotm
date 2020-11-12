use bevy::prelude::*;
use tracing::info;

use super::*;

pub fn setup(
    commands: &mut Commands,
    (game_screen, _game, screen): (Res<crate::GameScreen>, Res<Game>, Res<Screen>),
    asset_server: Res<AssetServer>,
    mut nine_patches: ResMut<Assets<bevy_ninepatch::NinePatchBuilder<()>>>,
    mut asset_handles: ResMut<crate::AssetHandles>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading");

        let inner_content = commands
            .spawn(NodeComponents::default())
            .current_entity()
            .unwrap();
        let panel_style = Style {
            position_type: PositionType::Absolute,
            position: Rect::<Val> {
                right: Val::Px(0.),
                left: Val::Undefined,
                bottom: Val::Px(0.),
                top: Val::Px(0.),
            },
            margin: Rect::all(Val::Px(0.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Px(400.), Val::Undefined),
            align_content: AlignContent::Stretch,
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        };

        let panel_handles = asset_handles.get_panel_handle(&asset_server, &mut nine_patches);
        commands
            .spawn(bevy_ninepatch::NinePatchComponents {
                style: panel_style.clone(),
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    panel_handles.1,
                    panel_handles.0,
                    inner_content,
                ),
                ..Default::default()
            })
            .with(ScreenTag)
            .current_entity()
            .unwrap();
    }
}

#[derive(Debug)]
pub enum InteractionEvent {
    Clicked(Option<Entity>),
    Hovered(Option<Entity>),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Interaction {
    Clicked,
    Hovered,
    None,
}

pub struct InteractionBox {
    pub size: Vec2,
}

pub struct Interacted;

#[derive(Default)]
pub struct State {
    cursor_moved_event_reader: EventReader<CursorMoved>,
    cursor_position: Vec2,
    hovered_entity: Option<Entity>,
    clicked_entity: Option<Entity>,
}

pub fn focus_system(
    mut state: Local<State>,
    mouse_button_input: Res<Input<MouseButton>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    touches_input: Res<Touches>,
    wnds: Res<Windows>,
    mut events: ResMut<Events<InteractionEvent>>,
    mut node_query: Query<(
        Entity,
        &InteractionBox,
        &GlobalTransform,
        Option<&mut Interaction>,
    )>,
) {
    if let Some(cursor_moved) = state.cursor_moved_event_reader.latest(&cursor_moved_events) {
        state.cursor_position = Vec2::new(
            cursor_moved.position.x() - wnds.get_primary().unwrap().width() as f32 / 2.,
            cursor_moved.position.y() - wnds.get_primary().unwrap().height() as f32 / 2.,
        );
    }
    if let Some(touch) = touches_input.get_pressed(0) {
        state.cursor_position = Vec2::new(
            touch.position.x() - wnds.get_primary().unwrap().width() as f32 / 2.,
            touch.position.y() - wnds.get_primary().unwrap().height() as f32 / 2.,
        );
    }

    if mouse_button_input.just_released(MouseButton::Left) || touches_input.just_released(0) {
        for (_entity, _node, _global_transform, interaction) in node_query.iter_mut() {
            if let Some(mut interaction) = interaction {
                if *interaction == Interaction::Clicked {
                    *interaction = Interaction::None;
                }
            }
        }
    }

    let mouse_clicked =
        mouse_button_input.just_pressed(MouseButton::Left) || touches_input.just_released(0);
    let mut hovered_entity = None;
    let mut clicked_entity = None;

    {
        let moused_over_z_sorted_nodes = node_query
            .iter_mut()
            .filter_map(|(entity, node, global_transform, interaction)| {
                let position = global_transform.translation;
                let ui_position = position.truncate();
                let extents = node.size / 2.0;
                let min = ui_position - extents;
                let max = ui_position + extents;
                // if the current cursor position is within the bounds of the node, consider it for clicking
                if (min.x()..max.x()).contains(&state.cursor_position.x())
                    && (min.y()..max.y()).contains(&state.cursor_position.y())
                {
                    Some((entity, interaction))
                } else {
                    if let Some(mut interaction) = interaction {
                        if *interaction == Interaction::Hovered {
                            *interaction = Interaction::None;
                        }
                    }
                    None
                }
            })
            .collect::<Vec<_>>();

        for (entity, interaction) in moused_over_z_sorted_nodes {
            if let Some(mut interaction) = interaction {
                if mouse_clicked {
                    // only consider nodes with ClickState "clickable"
                    if *interaction != Interaction::Clicked {
                        *interaction = Interaction::Clicked;
                        clicked_entity = Some(entity);
                    }
                } else if *interaction == Interaction::None {
                    *interaction = Interaction::Hovered;
                }
            }

            hovered_entity = Some(entity);
        }
    }
    if mouse_clicked {
        state.clicked_entity = clicked_entity;
        events.send(InteractionEvent::Clicked(clicked_entity));
    }
    if state.hovered_entity != hovered_entity {
        events.send(InteractionEvent::Hovered(hovered_entity));
    }

    // if there is a new hovered entity, but an entity is currently hovered, unhover the old entity
    if let Some(new_hovered_entity) = hovered_entity {
        if let Some(old_hovered_entity) = state.hovered_entity {
            if new_hovered_entity != old_hovered_entity {
                if let Ok(mut interaction) =
                    node_query.get_component_mut::<Interaction>(old_hovered_entity)
                {
                    if *interaction == Interaction::Hovered {
                        *interaction = Interaction::None;
                    }
                }
                state.hovered_entity = None;
            }
        }
        state.hovered_entity = hovered_entity;
    }
    state.hovered_entity = hovered_entity;
}

#[derive(Default)]
pub struct InteractingWith {
    selected: Option<Entity>,
    highlighted: Option<Entity>,
}

pub fn interaction(
    commands: &mut Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    bodies: Res<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    (mut event_reader, events): (
        Local<EventReader<InteractionEvent>>,
        Res<Events<InteractionEvent>>,
    ),
    mut state: Local<InteractingWith>,
    query_body: Query<&bevy_rapier2d::physics::RigidBodyHandleComponent>,
    query_children: Query<&Children>,
    query_interacted: Query<Entity, With<Interacted>>,
) {
    let selected = materials
        .add(Color::rgb(0x45 as f32 / 255., 0xb6 as f32 / 255., 0xfe as f32 / 255.).into());
    let highlighted = materials
        .add(Color::rgb(0x1c as f32 / 255., 0x49 as f32 / 255., 0x66 as f32 / 255.).into());

    for event in event_reader.iter(&events) {
        let (color, entity) = match event {
            InteractionEvent::Clicked(Some(e)) => {
                if let Some(selected) = state.selected {
                    if selected != *e {
                        if let Ok(children) = query_children.get(selected) {
                            for entity in children
                                .iter()
                                .filter(|entity| query_interacted.get(**entity).is_ok())
                            {
                                commands.despawn_recursive(*entity);
                            }
                        }
                    }
                }
                state.selected = Some(*e);
                (selected.clone(), e)
            }
            InteractionEvent::Hovered(Some(e)) => {
                if let Some(selected) = state.selected {
                    if *e == selected {
                        continue;
                    }
                }
                state.highlighted = Some(*e);
                (highlighted.clone(), e)
            }
            InteractionEvent::Clicked(None) => {
                if let Some(selected) = state.selected {
                    if let Ok(children) = query_children.get(selected) {
                        for entity in children
                            .iter()
                            .filter(|entity| query_interacted.get(**entity).is_ok())
                        {
                            commands.despawn_recursive(*entity);
                        }
                    }
                }
                state.selected = None;
                continue;
            }
            InteractionEvent::Hovered(None) => {
                // do not remove if we stop being hover the one being selected
                if let Some(selected) = state.selected {
                    if let Some(highlighted) = state.highlighted {
                        if selected == highlighted {
                            continue;
                        }
                    }
                }
                if let Some(children) = state.highlighted.and_then(|e| query_children.get(e).ok()) {
                    if let Some(entity) = children
                        .iter()
                        .find(|entity| query_interacted.get(**entity).is_ok())
                    {
                        commands.despawn_recursive(*entity);
                    }
                }
                state.highlighted = None;
                continue;
            }
        };

        if let Ok(rigid_body) = query_body.get(*entity) {
            let body = bodies.get(rigid_body.handle()).unwrap();

            let radius = 280.;
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
                2. * std::f32::consts::PI,
                0.,
            );
            let path = builder.build();
            let sprite = path.stroke(
                color,
                &mut meshes,
                Vec3::new(0.0, 0.0, 0.0),
                &bevy_prototype_lyon::prelude::StrokeOptions::default()
                    .with_line_width(20.0)
                    .with_line_cap(bevy_prototype_lyon::prelude::LineCap::Round)
                    .with_line_join(bevy_prototype_lyon::prelude::LineJoin::Round),
            );

            let interacted = commands
                .spawn(sprite)
                .with(Interacted)
                .current_entity()
                .unwrap();
            commands.push_children(*entity, &[interacted]);
        }
    }
}
