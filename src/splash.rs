use bevy::prelude::*;
use rand::Rng;

use tracing::info;

const CURRENT_SCREEN: crate::Screen = crate::Screen::Splash;

struct ScreenTag;

struct Screen {
    loaded: bool,
    done: Option<Timer>,
}
impl Default for Screen {
    fn default() -> Self {
        Screen {
            loaded: false,
            done: None,
        }
    }
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(Screen::default())
            .add_system(setup)
            .add_system(done)
            .add_system(animate_logo)
            .add_system_to_stage(crate::custom_stage::TEAR_DOWN, tear_down);
    }
}

fn setup(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if game_screen.current_screen == CURRENT_SCREEN && !screen.loaded {
        info!("Loading screen");
        screen.loaded = true;

        // let logo = include_bytes!("../assets/logo.png");
        // let texture_handle = asset_server.load_from(Box::new(logo.as_ref())).unwrap();
        let texture_handle = asset_server.load("logo.png");

        commands
            .spawn(SpriteBundle {
                material: materials.add(texture_handle.into()),
                ..Default::default()
            })
            .with(ScreenTag)
            .with(SplashGiggle(Timer::from_seconds(0.05, true)));

        screen.done = Some(Timer::from_seconds(0.7, false));
    }
}

struct SplashGiggle(Timer);

fn tear_down(
    commands: &mut Commands,
    game_screen: Res<crate::GameScreen>,
    mut screen: ResMut<Screen>,
    query: Query<Entity, With<ScreenTag>>,
) {
    if game_screen.current_screen != CURRENT_SCREEN && screen.loaded {
        info!("tear down");

        for entity in query.iter() {
            commands.despawn_recursive(entity);
        }

        screen.loaded = false;
    }
}

fn done(time: Res<Time>, mut screen: ResMut<Screen>, mut state: ResMut<crate::GameScreen>) {
    if let Some(ref mut timer) = screen.done {
        timer.tick(time.delta_seconds());
        if timer.just_finished() {
            state.current_screen = crate::Screen::Menu;
        }
    }
}

fn animate_logo(
    time: Res<Time>,
    mut query: Query<(&mut SplashGiggle, &mut Transform), With<ScreenTag>>,
) {
    for (mut timer, mut transform) in query.iter_mut() {
        timer.0.tick(time.delta_seconds());
        if timer.0.just_finished() {
            let translation = transform.translation;
            if translation.x != 0. || translation.y != 0. {
                *transform = Transform::identity();
                continue;
            }

            let scale = transform.scale;
            // `scale.0 != 1.` for floating numbers
            if (scale.x - 1.) > 0.01 {
                *transform = Transform::identity();
                continue;
            }

            let mut rng = rand::thread_rng();
            let act = rng.gen_range(0, 100);

            if act < 20 {
                let span = 1.;
                let x: f32 = rng.gen_range(-span, span);
                let y: f32 = rng.gen_range(-span, span);
                *transform = Transform::from_translation(Vec3::new(x, y, 0.));
            }
            if act > 80 {
                let scale_diff = 0.02;
                let new_scale: f32 = rng.gen_range(1. - scale_diff, 1. + scale_diff);
                *transform = Transform::from_scale(Vec3::splat(new_scale));
            }
        }
    }
}
