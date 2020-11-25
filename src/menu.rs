use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

use tracing::info;

use bevy_easings::Ease;

const CURRENT_SCREEN: crate::Screen = crate::Screen::Menu;

struct ScreenTag;

struct Screen {
    loaded: bool,
    first_load: bool,
    menu_selected: Option<i32>,
}
impl Default for Screen {
    fn default() -> Self {
        Screen {
            loaded: false,
            first_load: true,
            menu_selected: None,
        }
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Screen::default())
            .add_system(keyboard_input_system)
            .add_system(setup)
            .add_system(button_system)
            .add_system(display_menu_item_selector)
            .add_system(rotate_on_self)
            .add_system(go_away)
            .add_system(despawn_gone_ships)
            .add_system(menu_ship_behaviour)
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down);
    }
}

#[derive(Clone, Copy)]
enum MenuButton {
    NewGame,
    About,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

impl Into<String> for MenuButton {
    fn into(self) -> String {
        match self {
            MenuButton::NewGame => "New Game".to_string(),
            MenuButton::About => "About".to_string(),
            #[cfg(not(target_arch = "wasm32"))]
            MenuButton::Quit => "Quit".to_string(),
        }
    }
}

const MENU_BUTTONS: &[MenuButton] = &[
    MenuButton::NewGame,
    MenuButton::About,
    #[cfg(not(target_arch = "wasm32"))]
    MenuButton::Quit,
];

struct RotateOnSelf {
    duration: f64,
    offset: f64,
    direction: crate::space::RotationDirection,
}

impl RotateOnSelf {
    fn every(secs: f64) -> Self {
        Self {
            duration: secs,
            offset: rand::thread_rng().gen_range(0., 2. * std::f64::consts::PI),
            direction: crate::space::RotationDirection::Clockwise,
        }
    }
}

fn setup(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    mut asset_handles: ResMut<crate::AssetHandles>,
    asset_server: Res<AssetServer>,
    mut nine_patches: ResMut<Assets<bevy_ninepatch::NinePatchBuilder<()>>>,
    mut buttons: ResMut<Assets<crate::ui::button::Button>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");

        let panel_handles = asset_handles.get_menu_panel_handle(&asset_server, &mut nine_patches);

        let button_handle = asset_handles.get_button_handle(
            &asset_server,
            &mut materials,
            &mut nine_patches,
            &mut buttons,
        );
        let button = buttons.get(&button_handle).unwrap();

        let game_handles =
            asset_handles.get_game_handles(&asset_server, &mut materials, &mut atlases);

        let font: Handle<Font> = asset_handles.get_font_main_handle(&asset_server);
        let menu_indicator: Handle<ColorMaterial> =
            asset_handles.get_ui_selection_handle(&asset_server, &mut materials);

        let color_none = materials.add(Color::NONE.into());

        commands
            .spawn((
                Transform {
                    translation: Vec3::new(-200., -75., crate::Z_MOON),
                    ..Default::default()
                },
                GlobalTransform::default(),
            ))
            .with_children(|moon_holder| {
                moon_holder
                    .spawn(SpriteBundle {
                        transform: Transform {
                            scale: Vec3::splat(0.25),
                            ..Default::default()
                        },
                        material: game_handles
                            .orbiters
                            .choose(&mut rand::thread_rng())
                            .unwrap()
                            .clone_weak(),
                        ..Default::default()
                    })
                    .with(RotateOnSelf::every(10.));
            })
            .with(
                crate::space::SpawnShip::every(7., crate::space::RotationDirection::Clockwise)
                    .with_scale(2.)
                    .with_headstart(),
            )
            .with(crate::game::OwnedBy::Neutral)
            .with(ScreenTag);

        commands
            .spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect::<Val> {
                        left: Val::Percent(20.),
                        right: Val::Undefined,
                        bottom: Val::Undefined,
                        top: Val::Percent(25.),
                    },
                    size: Size::<Val> {
                        height: Val::Px(95.),
                        width: Val::Auto,
                    },
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                material: materials.add(Color::NONE.into()),
                ..Default::default()
            })
            .with_children(|title_parent| {
                title_parent.spawn(TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(75.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    text: Text {
                        value: "War of the Moons".to_string(),
                        font: font.clone(),
                        style: TextStyle {
                            color: crate::ui::ColorScheme::TEXT,
                            font_size: 75.,
                            ..Default::default()
                        },
                    },
                    ..Default::default()
                });
            })
            .with(ScreenTag);

        let panel_style = Style {
            position_type: PositionType::Absolute,
            position: Rect::<Val> {
                left: Val::Percent(53.),
                right: Val::Undefined,
                bottom: Val::Percent(15.),
                top: Val::Undefined,
            },
            margin: Rect::all(Val::Px(0.)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Px(400.), Val::Px(300.)),
            align_content: AlignContent::Stretch,
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        };

        let button_shift_start = 15.;
        let button_shift = 45.;
        let buttons = MENU_BUTTONS
            .iter()
            .enumerate()
            .map(|(i, button_item)| {
                commands.spawn(NodeBundle {
                    style: Style {
                        margin: Rect {
                            left: Val::Px(button_shift_start + i as f32 * button_shift),
                            right: Val::Auto,
                            top: Val::Auto,
                            bottom: Val::Auto,
                        },
                        flex_direction: FlexDirection::RowReverse,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    draw: Draw {
                        is_transparent: true,
                        ..Default::default()
                    },
                    material: color_none.clone(),
                    ..Default::default()
                });
                let entity = commands.current_entity().unwrap();
                let button = button.add(
                    commands,
                    225.,
                    50.,
                    Rect::all(Val::Auto),
                    font.clone(),
                    *button_item,
                    25.,
                );
                commands
                    .spawn(ImageBundle {
                        style: Style {
                            size: Size {
                                height: Val::Px(17.),
                                width: Val::Px(17.),
                            },
                            margin: Rect {
                                right: Val::Px(15.),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        draw: Draw {
                            is_transparent: true,
                            is_visible: false,
                            ..Default::default()
                        },
                        material: menu_indicator.clone(),
                        ..Default::default()
                    })
                    .with(MenuItemSelector(i));
                let indicator = commands.current_entity().unwrap();
                commands.push_children(entity, &[button, indicator]);
                entity
            })
            .collect::<Vec<_>>();
        let inner_content = commands
            .spawn(NodeBundle {
                material: color_none,
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    ..Default::default()
                },
                draw: Draw {
                    is_transparent: true,
                    ..Default::default()
                },
                ..Default::default()
            })
            .current_entity()
            .unwrap();
        commands.push_children(inner_content, buttons.as_slice());

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
            .current_entity()
            .unwrap();
        if screen.first_load {
            commands.with(
                Style {
                    position: Rect::<Val> {
                        left: Val::Percent(120.),
                        right: Val::Undefined,
                        bottom: Val::Percent(15.),
                        top: Val::Undefined,
                    },
                    ..panel_style
                }
                .ease_to(
                    panel_style,
                    bevy_easings::EaseFunction::BounceOut,
                    bevy_easings::EasingType::Once {
                        duration: std::time::Duration::from_millis(800),
                    },
                ),
            );
        } else {
            commands.with(panel_style);
        }

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
    mut screen: ResMut<Screen>,
    keyboard_input: Res<Input<KeyCode>>,
    mut wnds: ResMut<Windows>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && screen.loaded {
        if keyboard_input.just_released(KeyCode::Escape) {
            #[cfg(not(target_arch = "wasm32"))]
            {
                game_screen.current_screen = crate::Screen::Exit;
            }
        } else if keyboard_input.just_released(KeyCode::F) {
            let window = wnds.get_primary_mut().unwrap();
            match window.mode() {
                bevy::window::WindowMode::Windowed => {
                    window.set_mode(bevy::window::WindowMode::BorderlessFullscreen)
                }
                _ => window.set_mode(bevy::window::WindowMode::Windowed),
            }
        } else if keyboard_input.just_released(KeyCode::Down) {
            screen.menu_selected = Some(
                screen
                    .menu_selected
                    .map(|i| i32::min(MENU_BUTTONS.len() as i32 - 1, i + 1))
                    .unwrap_or(0),
            );
        } else if keyboard_input.just_released(KeyCode::Up) {
            screen.menu_selected = Some(
                screen
                    .menu_selected
                    .map(|i| i32::max(0, i - 1))
                    .unwrap_or(0),
            );
        } else if keyboard_input.just_released(KeyCode::Space)
            || keyboard_input.just_released(KeyCode::Return)
        {
            match screen.menu_selected {
                Some(0) => game_screen.current_screen = crate::Screen::Game,
                Some(1) => game_screen.current_screen = crate::Screen::About,
                Some(2) => game_screen.current_screen = crate::Screen::Exit,
                _ => (),
            }
        }
    }
}

fn button_system(
    mut game_screen: ResMut<crate::GameScreen>,
    mut screen: ResMut<Screen>,

    mut interaction_query: Query<
        (&Interaction, &crate::ui::button::ButtonId<MenuButton>),
        (With<Button>, Mutated<Interaction>),
    >,
) {
    for (interaction, button_id) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => match button_id.0 {
                #[cfg(not(target_arch = "wasm32"))]
                MenuButton::Quit => game_screen.current_screen = crate::Screen::Exit,
                MenuButton::About => game_screen.current_screen = crate::Screen::About,
                MenuButton::NewGame => game_screen.current_screen = crate::Screen::Game,
            },
            Interaction::Hovered => match button_id.0 {
                MenuButton::NewGame => screen.menu_selected = Some(0),
                MenuButton::About => screen.menu_selected = Some(1),
                #[cfg(not(target_arch = "wasm32"))]
                MenuButton::Quit => screen.menu_selected = Some(2),
            },
            Interaction::None => screen.menu_selected = None,
        }
    }
}

struct MenuItemSelector(usize);

fn display_menu_item_selector(
    screen: Res<Screen>,
    mut query: Query<(&MenuItemSelector, &mut Draw)>,
) {
    if let Some(index_selected) = screen.menu_selected {
        for (selector, mut draw) in query.iter_mut() {
            if selector.0 == index_selected as usize {
                draw.is_visible = true;
            } else {
                draw.is_visible = false;
            }
        }
    } else {
        for (_, mut draw) in query.iter_mut() {
            draw.is_visible = false;
        }
    }
}

fn rotate_on_self(time: Res<Time>, mut query: Query<(&RotateOnSelf, &mut Transform)>) {
    for (rotate, mut transform) in query.iter_mut() {
        transform.rotation = Quat::from_axis_angle(
            Vec3::unit_z(),
            rotate.direction.to_factor()
                * ((time.seconds_since_startup % rotate.duration) / rotate.duration
                    * (2. * std::f64::consts::PI)
                    + rotate.offset) as f32,
        )
    }
}

fn menu_ship_behaviour(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    query: Query<Entity, (With<crate::space::Ship>, Without<Bye>, Without<GoAwayAfter>)>,
) {
    if game_screen.current_screen == CURRENT_SCREEN {
        for ship in query.iter() {
            commands.insert_one(ship, GoAwayAfter(Timer::from_seconds(12., false)));
        }
    }
}

struct GoAwayAfter(Timer);
struct Bye {
    timer: Timer,
    target: bevy_rapier2d::rapier::math::Vector<f32>,
}

fn go_away(commands: &mut Commands, time: Res<Time>, mut query: Query<(&mut GoAwayAfter, Entity)>) {
    for (mut gaa, entity) in query.iter_mut() {
        gaa.0.tick(time.delta_seconds);
        if gaa.0.finished && rand::thread_rng().gen_bool(0.001) {
            commands.remove_one::<crate::space::Orbiter>(entity);
            commands.remove_one::<GoAwayAfter>(entity);

            let mut x = rand::thread_rng().gen_range(-1., 1.);
            x += if x > 0. { 1. } else { -1. };
            let mut y = rand::thread_rng().gen_range(-1., 1.);
            y += if y > 0. { 1. } else { -1. };
            let target = bevy_rapier2d::rapier::math::Vector::new(x, y).normalize() * 10000.;
            commands.insert_one(
                entity,
                Bye {
                    timer: Timer::from_seconds(10., false),
                    target,
                },
            );
        }
    }
}

fn despawn_gone_ships(
    commands: &mut Commands,
    time: Res<Time>,
    mut bodies: ResMut<bevy_rapier2d::rapier::dynamics::RigidBodySet>,
    mut query: Query<(
        &bevy_rapier2d::physics::RigidBodyHandleComponent,
        &mut Bye,
        Entity,
    )>,
) {
    for (rigid_body, mut bye, entity) in query.iter_mut() {
        bye.timer.tick(time.delta_seconds);
        if bye.timer.just_finished {
            commands.despawn_recursive(entity);
        } else {
            let mut body = bodies.get_mut(rigid_body.handle()).unwrap();
            let (linvel, rot) =
                crate::space::go_from_to_rapier(body.position.translation.vector, bye.target);
            body.linvel = linvel * 100.;
            body.angvel = 0.;
            body.position.rotation =
                bevy_rapier2d::na::UnitComplex::from_angle(rot - std::f32::consts::FRAC_PI_2);
        }
    }
}
