use bevy::prelude::*;
use tracing::info;

use super::*;

pub struct UiSelected;
pub struct UiHighlighted;
pub struct UiTime;

pub struct UiGameInteractionBlock;

pub fn setup(
    commands: &mut Commands,
    (game_screen, _game, screen): (Res<crate::GameScreen>, Res<Game>, Res<Screen>),
    asset_server: Res<AssetServer>,
    mut nine_patches: ResMut<Assets<bevy_ninepatch::NinePatchBuilder<()>>>,
    mut asset_handles: ResMut<crate::AssetHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading");

        let material_none = materials.add(Color::NONE.into());

        let font = asset_handles.get_font_sub_handle(&asset_server);

        commands
            .spawn(TextBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        left: Val::Px(10.),
                        top: Val::Px(10.),
                        ..Default::default()
                    },
                    size: Size {
                        height: Val::Px(30.),
                        ..Default::default()
                    },
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
                text: Text {
                    font: font.clone(),
                    style: TextStyle {
                        color: crate::ui::ColorScheme::TEXT_DARK,
                        font_size: 30.,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            })
            .with(UiTime)
            .with(ScreenTag);

        let inner_content = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                    ..Default::default()
                },
                draw: Draw {
                    is_transparent: true,
                    ..Default::default()
                },
                material: material_none.clone(),
                ..Default::default()
            })
            .with(bevy::ui::FocusPolicy::Block)
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::ColumnReverse,
                            size: Size::new(Val::Percent(100.), Val::Percent(50.)),
                            ..Default::default()
                        },
                        draw: Draw {
                            is_transparent: true,
                            ..Default::default()
                        },
                        material: material_none.clone(),
                        ..Default::default()
                    })
                    .with(UiSelected);
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::ColumnReverse,
                            size: Size::new(Val::Percent(100.), Val::Percent(50.)),
                            ..Default::default()
                        },
                        draw: Draw {
                            is_transparent: true,
                            ..Default::default()
                        },
                        material: material_none.clone(),
                        ..Default::default()
                    })
                    .with(UiHighlighted);
            })
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
            .spawn(bevy_ninepatch::NinePatchBundle {
                style: panel_style.clone(),
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    panel_handles.1,
                    panel_handles.0,
                    inner_content,
                ),
                ..Default::default()
            })
            .with(ScreenTag)
            .with(UiGameInteractionBlock)
            .current_entity()
            .unwrap();
    }
}

pub fn timer(game: Res<Game>, mut timer: Query<&mut Text, With<UiTime>>) {
    for mut timer in timer.iter_mut() {
        let secs = game.elapsed.floor() as i32;
        let ms = ((game.elapsed - secs as f32) * 1000.) as i32;
        let m = secs / 60;
        let secs = secs % 60;
        timer.value = format!("{:02}:{:02}.{}", m, secs, ms);
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
    pub radius: f32,
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
    block_query: Query<(&GlobalTransform, &Node), With<UiGameInteractionBlock>>,
    mut node_query: Query<(
        Entity,
        &InteractionBox,
        &GlobalTransform,
        Option<&mut Interaction>,
    )>,
) {
    if let Some(cursor_moved) = state.cursor_moved_event_reader.latest(&cursor_moved_events) {
        state.cursor_position = Vec2::new(
            cursor_moved.position.x - wnds.get_primary().unwrap().width() as f32 / 2.,
            cursor_moved.position.y - wnds.get_primary().unwrap().height() as f32 / 2.,
        );
    }
    if let Some(touch) = touches_input.get_pressed(0) {
        state.cursor_position = Vec2::new(
            touch.position.x - wnds.get_primary().unwrap().width() as f32 / 2.,
            touch.position.y - wnds.get_primary().unwrap().height() as f32 / 2.,
        );
    }
    for (global_transform, node) in block_query.iter() {
        let position = global_transform.translation;
        let ui_position = position.truncate();
        let extents = node.size / 2.0;
        let min = ui_position - extents;
        let max = ui_position + extents;
        if (min.x..max.x)
            .contains(&(state.cursor_position.x + wnds.get_primary().unwrap().width() as f32 / 2.))
            && (min.y..max.y).contains(
                &(state.cursor_position.y + wnds.get_primary().unwrap().height() as f32 / 2.),
            )
        {
            return;
        }
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
                // if the current cursor position is within the bounds of the node, consider it for clicking
                if global_transform
                    .translation
                    .truncate()
                    .distance(state.cursor_position)
                    < node.radius
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

pub fn interaction(
    commands: &mut Commands,
    mut game: ResMut<Game>,
    mut asset_handles: ResMut<crate::AssetHandles>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    bodies: Res<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    (mut event_reader, events): (
        Local<EventReader<InteractionEvent>>,
        Res<Events<InteractionEvent>>,
    ),
    query_body: Query<(
        &bevy_rapier2d::physics::RigidBodyHandleComponent,
        &InteractionBox,
        &crate::game::OwnedBy,
    )>,
    query_children: Query<&Children>,
    query_interacted: Query<Entity, With<Interacted>>,
) {
    for event in event_reader.iter(&events) {
        let (color_selected, entity) = match event {
            InteractionEvent::Clicked(Some(e)) => {
                if let Some(selected) = game.selected {
                    if selected != *e {
                        if let Ok(children) = query_children.get(selected) {
                            for entity in children
                                .iter()
                                .filter(|entity| query_interacted.get(**entity).is_ok())
                            {
                                commands.despawn_recursive(*entity);
                            }
                        }
                        game.ratio = super::Ratio::default();
                    } else {
                        game.ratio.next()
                    }
                } else {
                    game.ratio = super::Ratio::default();
                }
                game.selected = Some(*e);
                (true, e)
            }
            InteractionEvent::Hovered(Some(e)) => {
                if let Some(selected) = game.selected {
                    if *e == selected {
                        continue;
                    }
                }
                if let Some(children) = game.targeted.and_then(|e| query_children.get(e).ok()) {
                    if let Some(entity) = children
                        .iter()
                        .find(|entity| query_interacted.get(**entity).is_ok())
                    {
                        commands.despawn_recursive(*entity);
                    }
                }
                game.targeted = Some(*e);
                (false, e)
            }
            InteractionEvent::Clicked(None) => {
                if let Some(selected) = game.selected {
                    if let Ok(children) = query_children.get(selected) {
                        for entity in children
                            .iter()
                            .filter(|entity| query_interacted.get(**entity).is_ok())
                        {
                            commands.despawn_recursive(*entity);
                        }
                    }
                }
                game.selected = None;
                continue;
            }
            InteractionEvent::Hovered(None) => {
                // do not remove if we stop being hover the one being selected
                if let Some(selected) = game.selected {
                    if let Some(highlighted) = game.targeted {
                        if selected == highlighted {
                            continue;
                        }
                    }
                }
                if let Some(children) = game.targeted.and_then(|e| query_children.get(e).ok()) {
                    if let Some(entity) = children
                        .iter()
                        .find(|entity| query_interacted.get(**entity).is_ok())
                    {
                        commands.despawn_recursive(*entity);
                    }
                }
                game.targeted = None;
                continue;
            }
        };

        if let Ok((rigid_body, interaction_box, owner)) = query_body.get(*entity) {
            let color = match (color_selected, owner) {
                (true, OwnedBy::Player(0)) => asset_handles.get_color_selected_self(&mut materials),
                (true, _) => asset_handles.get_color_selected_other(&mut materials),
                (false, OwnedBy::Player(0)) => {
                    asset_handles.get_color_highlighted_self(&mut materials)
                }
                (false, _) => asset_handles.get_color_highlighted_other(&mut materials),
            };
            let body = bodies.get(rigid_body.handle()).unwrap();

            let radius = interaction_box.radius * 10. - 20.;
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

pub enum UiElement {
    Owner,
    ShipCount,
    Status,
    SelectedRatio,
    SelectedCount,
}

pub struct Panel(Entity);

pub fn ui_update_on_interaction_event(
    commands: &mut Commands,
    game: Res<Game>,
    mut asset_handles: ResMut<crate::AssetHandles>,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    (mut event_reader, events): (
        Local<EventReader<InteractionEvent>>,
        Res<Events<InteractionEvent>>,
    ),
    query_planet: Query<&Planet>,
    query_moon: Query<&Moon>,
    query_ui_selected: Query<(Entity, Option<&Children>), With<UiSelected>>,
    query_ui_highlighted: Query<(Entity, Option<&Children>), With<UiHighlighted>>,
) {
    let font = asset_handles.get_font_main_handle(&assets);
    let color_none = materials.add(Color::NONE.into());
    for event in event_reader.iter(&events) {
        let (moon_entity, ui_target_entity, children, main) = match event {
            InteractionEvent::Clicked(None) => {
                if let Some((ui_entity, children)) = query_ui_selected.iter().next() {
                    (None, ui_entity, children, true)
                } else {
                    continue;
                }
            }
            InteractionEvent::Clicked(Some(moon_entity)) => {
                if let Some((_ui_entity, children)) = query_ui_highlighted.iter().next() {
                    if let Some(children) = children {
                        for child in children.iter() {
                            commands.despawn_recursive(*child);
                        }
                    }
                }
                let (ui_entity, children) = query_ui_selected.iter().next().unwrap();
                (Some(moon_entity), ui_entity, children, true)
            }
            InteractionEvent::Hovered(None) => {
                if let Some((ui_entity, children)) = query_ui_highlighted.iter().next() {
                    (None, ui_entity, children, false)
                } else {
                    continue;
                }
            }
            InteractionEvent::Hovered(Some(moon_entity)) => {
                let (ui_entity, children) = query_ui_highlighted.iter().next().unwrap();
                if let Some(selected) = game.selected {
                    if *moon_entity == selected {
                        (None, ui_entity, children, false)
                    } else {
                        (Some(moon_entity), ui_entity, children, false)
                    }
                } else {
                    (Some(moon_entity), ui_entity, children, false)
                }
            }
        };
        if let Some(children) = children {
            for child in children.iter() {
                commands.despawn_recursive(*child);
            }
        }

        if let Some(moon_entity) = moon_entity {
            let name = if let Ok(planet) = query_planet.get(*moon_entity) {
                planet.name.clone()
            } else {
                let moon = query_moon.get(*moon_entity).unwrap();
                let planet = query_planet.get(moon.planet).unwrap();
                format!("{} {}", planet.name, roman::to(moon.index).unwrap())
            };
            let ui_name = commands
                .spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(30.),
                            ..Default::default()
                        },
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    text: Text {
                        value: name,
                        font: font.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT_DARK,
                            font_size: 30.,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                })
                .with(Panel(ui_target_entity))
                .current_entity()
                .unwrap();

            let ui_owner = commands
                .spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(15.),
                            ..Default::default()
                        },
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    text: Text {
                        font: font.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT_DARK,
                            font_size: 15.,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Panel(ui_target_entity))
                .with(UiElement::Owner)
                .current_entity()
                .unwrap();

            let ui_ships_orbiting_count = commands
                .spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(25.),
                            ..Default::default()
                        },
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    text: Text {
                        font: font.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT_DARK,
                            font_size: 25.,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Panel(ui_target_entity))
                .with(UiElement::ShipCount)
                .current_entity()
                .unwrap();
            let ui_under_attack = commands
                .spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(25.),
                            ..Default::default()
                        },
                        align_self: AlignSelf::Center,
                        ..Default::default()
                    },
                    text: Text {
                        font: font.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT_HIGHLIGHT,
                            font_size: 25.,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(Panel(ui_target_entity))
                .with(UiElement::Status)
                .current_entity()
                .unwrap();

            commands.push_children(
                ui_target_entity,
                &[ui_name, ui_owner, ui_ships_orbiting_count, ui_under_attack],
            );
            if main {
                let ui_ship_selection = commands
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size {
                                height: Val::Px(25.),
                                ..Default::default()
                            },
                            align_self: AlignSelf::Center,
                            ..Default::default()
                        },
                        draw: Draw {
                            is_transparent: true,
                            ..Default::default()
                        },
                        material: color_none.clone(),
                        ..Default::default()
                    })
                    .with(Panel(ui_target_entity))
                    .with(UiElement::SelectedRatio)
                    .current_entity()
                    .unwrap();

                let ui_ship_ratio_count = commands
                    .spawn(NodeBundle {
                        style: Style {
                            size: Size {
                                height: Val::Px(25.),
                                ..Default::default()
                            },
                            align_self: AlignSelf::Center,
                            ..Default::default()
                        },
                        draw: Draw {
                            is_transparent: true,
                            ..Default::default()
                        },
                        material: color_none.clone(),
                        ..Default::default()
                    })
                    .current_entity()
                    .unwrap();
                commands.with_children(|ratio_count| {
                    ratio_count
                        .spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(17.),
                                    ..Default::default()
                                },
                                align_self: AlignSelf::Center,
                                ..Default::default()
                            },
                            text: Text {
                                font: font.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT_DARK,
                                    font_size: 17.,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with(Panel(ui_target_entity))
                        .with(UiElement::SelectedRatio);
                    ratio_count
                        .spawn(TextBundle {
                            style: Style {
                                size: Size {
                                    height: Val::Px(17.),
                                    ..Default::default()
                                },
                                align_self: AlignSelf::Center,
                                ..Default::default()
                            },
                            text: Text {
                                font: font.clone(),
                                style: TextStyle {
                                    color: crate::ui::ColorScheme::TEXT_DARK,
                                    font_size: 17.,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with(Panel(ui_target_entity))
                        .with(UiElement::SelectedCount);
                });
                commands.push_children(ui_target_entity, &[ui_ship_selection, ui_ship_ratio_count]);
            }
        }
    }
}

pub fn ship_count(
    mut game: ResMut<Game>,
    query_moon: Query<Entity, With<Moon>>,
    query_planet: Query<Entity, With<Planet>>,
    query_ships: Query<(&crate::space::Orbiter, &crate::game::OwnedBy), With<crate::space::Ship>>,
) {
    for moon_entity in query_moon.iter().chain(query_planet.iter()) {
        let ships_orbiting_count = query_ships
            .iter()
            .filter(|(orbiter, _)| orbiter.around == moon_entity)
            .fold(
                std::collections::HashMap::new(),
                |mut counts, (_, owned_by)| {
                    let count = counts.entry(owned_by.clone()).or_insert(0);
                    *count += 1;
                    counts
                },
            );
        game.ship_counts.insert(moon_entity, ships_orbiting_count);
    }
}

pub fn ui_update(
    commands: &mut Commands,
    game: Res<Game>,
    mut asset_handles: ResMut<crate::AssetHandles>,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query_ui_selected: Query<Entity, With<UiSelected>>,
    query_ui_highlighted: Query<Entity, With<UiHighlighted>>,
    query_owner: Query<&crate::game::OwnedBy>,
    mut ui_texts: Query<(&mut Text, &UiElement, &Panel)>,
    mut ui_nodes: Query<(Entity, Option<&mut Children>, &UiElement, &Panel), Without<Text>>,
) {
    for (moon_entity, ui_entity) in
        std::iter::once((game.selected, query_ui_selected.iter().next())).chain(std::iter::once((
            game.targeted,
            query_ui_highlighted.iter().next(),
        )))
    {
        if let Some(entity) = moon_entity {
            if let Some(ui_main) = ui_entity {
                let owner = query_owner.get(entity).unwrap();
                let ships_orbiting_count = game.ship_counts.get(&entity).unwrap();
                for (mut ui_text, element, panel) in ui_texts.iter_mut() {
                    if panel.0 != ui_main {
                        continue;
                    }
                    match element {
                        UiElement::Owner => {
                            ui_text.value = match owner {
                                crate::game::OwnedBy::Player(0) => "Owned by you".to_string(),
                                crate::game::OwnedBy::Player(_) => {
                                    "Owned by another player".to_string()
                                }
                                crate::game::OwnedBy::Neutral => "Free".to_string(),
                            };
                        }
                        UiElement::ShipCount => {
                            ui_text.value = match ships_orbiting_count.get(owner).unwrap_or(&0) {
                                0 => "no ship".to_string(),
                                1 => "1 ship".to_string(),
                                n => format!("{} ships", n),
                            };
                        }
                        UiElement::Status => {
                            if ships_orbiting_count.len() > 1 {
                                ui_text.value = "Under Attack".to_string();
                            } else {
                                ui_text.value = "".to_string();
                            };
                        }
                        UiElement::SelectedRatio => {
                            if let crate::game::OwnedBy::Player(0) = owner {
                                ui_text.value = format!("{} ", game.ratio);
                            } else {
                                ui_text.value = "".to_string();
                            }
                        }
                        UiElement::SelectedCount => {
                            if let crate::game::OwnedBy::Player(0) = owner {
                                ui_text.value = format!(
                                    " - {} ship selected",
                                    game.ratio.of(*ships_orbiting_count
                                        .get(owner)
                                        .unwrap_or(&(0 as usize)))
                                );
                            } else {
                                ui_text.value = "".to_string();
                            }
                        }
                    }
                }

                if let crate::game::OwnedBy::Player(0) = owner {
                    for (entity, children, element, panel) in ui_nodes.iter_mut() {
                        if panel.0 != ui_main {
                            continue;
                        }

                        if let UiElement::SelectedRatio = element {
                            let levels = asset_handles.get_ui_level(&assets, &mut materials);
                            if let Some(mut children) = children {
                                for child in children.iter() {
                                    commands.despawn_recursive(*child);
                                }
                                *children = Children::default();
                            }

                            let mut markers = vec![];
                            for i in 0..game.ratio.as_usize() {
                                commands
                                    .spawn(ImageBundle {
                                        material: levels.0.clone(),
                                        style: Style {
                                            margin: Rect {
                                                left: Val::Px(5.),
                                                right: Val::Px(5.),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        draw: Draw {
                                            is_transparent: true,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    })
                                    .with_bundle((
                                        Button,
                                        bevy::ui::Interaction::default(),
                                        bevy::ui::FocusPolicy::Block,
                                    ))
                                    .with(Ratio::from_usize(i + 1));
                                markers.push(commands.current_entity().unwrap());
                            }
                            for i in game.ratio.as_usize()..4 {
                                commands
                                    .spawn(ImageBundle {
                                        material: levels.1.clone(),
                                        style: Style {
                                            margin: Rect {
                                                left: Val::Px(5.),
                                                right: Val::Px(5.),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        draw: Draw {
                                            is_transparent: true,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    })
                                    .with_bundle((
                                        Button,
                                        bevy::ui::Interaction::default(),
                                        bevy::ui::FocusPolicy::Block,
                                    ))
                                    .with(Ratio::from_usize(i + 1));
                                markers.push(commands.current_entity().unwrap());
                            }
                            commands.insert_children(entity, 0, &markers);
                        }
                    }
                }
            }
        }
    }
}

pub fn orders(
    commands: &mut Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    game: Res<Game>,
    query_owner: Query<&crate::game::OwnedBy>,
    query_ships: Query<
        (Entity, &crate::space::Orbiter, &crate::game::OwnedBy),
        With<crate::space::Ship>,
    >,
) {
    if mouse_button_input.just_pressed(MouseButton::Right)
        && game.selected.is_some()
        && game.targeted.is_some()
    {
        let selected = game.selected.unwrap();
        let targeted = game.targeted.unwrap();
        let owner = query_owner.get(selected).unwrap();
        if *owner != crate::game::OwnedBy::Player(0) {
            return;
        }
        let ship_count = *game
            .ship_counts
            .get(&selected)
            .unwrap()
            .get(&crate::game::OwnedBy::Player(0))
            .unwrap_or(&0);

        query_ships
            .iter()
            .filter(|(_, orbiter, owned_by)| {
                orbiter.around == selected && **owned_by == crate::game::OwnedBy::Player(0)
            })
            .take(game.ratio.of(ship_count))
            .for_each(|(entity, _, _)| {
                commands.remove_one::<crate::space::Orbiter>(entity);
                commands.insert_one(
                    entity,
                    crate::space::MoveTowards {
                        speed: 2000.,
                        from: selected,
                        towards: targeted,
                    },
                );
            });
    }
}

pub fn change_ratio_ui(
    mut game: ResMut<Game>,
    mut interaction_query: Query<
        (&bevy::ui::Interaction, &Ratio),
        (With<Button>, Mutated<bevy::ui::Interaction>),
    >,
) {
    for (interaction, button_ratio) in interaction_query.iter_mut() {
        match *interaction {
            bevy::ui::Interaction::Clicked => game.ratio = *button_ratio,
            _ => (),
        }
    }
}
