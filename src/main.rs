// disable console opening on windows
#![windows_subsystem = "windows"]

use bevy::{app::AppExit, prelude::*, window::WindowMode};
use serde::{Deserialize, Serialize};

mod assets;
pub mod ui;
use assets::AssetHandles;

mod about;
mod game;
mod lost;
mod menu;
mod space;
mod splash;
mod win;

pub const Z_PLANET: f32 = 0.0;
pub const Z_MOON: f32 = 1.0;
pub const Z_SHIP: f32 = 2.0;

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    width: u32,
    height: u32,
    fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            width: 1280,
            height: 720,
            fullscreen: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bigger_player_ship_rate: f64,
    pub bigger_player_ship_change: f32,
    pub fleet_timer: f32,
    pub fleet_delay: f32,
    pub fleet_chance: f32,
    pub asteroid_timer: f32,
    pub asteroid_chance: f32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bigger_player_ship_rate: 0.25,
            bigger_player_ship_change: 1.25,
            fleet_timer: 1.,
            fleet_delay: 15.,
            fleet_chance: 0.2,
            asteroid_timer: 30.,
            asteroid_chance: 0.5,
        }
    }
}

use lazy_static::lazy_static;
lazy_static! {
    static ref CONFIG: Config = config::read_from("config.conf").unwrap();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings: Settings = config::read_from("settings.conf")?;

    let mut builder = App::build();

    builder
        // resources
        .add_resource(WindowDescriptor {
            title: "wotm".to_string(),
            width: settings.width,
            height: settings.height,
            vsync: true,
            resizable: false,
            mode: if settings.fullscreen {
                WindowMode::Fullscreen { use_size: true }
            } else {
                WindowMode::Windowed
            },
            ..Default::default()
        })
        .add_resource(settings)
        .add_resource(ClearColor(Color::rgb(0., 0., 0.01)));

    #[cfg(not(target_arch = "wasm32"))]
    if cfg!(debug_assertions) {
        builder.add_resource(bevy::log::LogSettings {
            level: bevy::log::Level::INFO,
            filter:
                "bevy_log_diagnostic=debug,gfx_backend_metal=warn,wgpu_core=warn,bevy_render=warn"
                    .to_string(),
            ..Default::default()
        });
    } else {
        builder.add_resource(bevy::log::LogSettings {
            level: bevy::log::Level::WARN,
            ..Default::default()
        });
    }

    builder.add_plugins_with(DefaultPlugins, |group| {
        #[cfg(feature = "bundled")]
        return group
            .disable::<bevy::asset::AssetPlugin>()
            .add_after::<bevy::asset::AssetPlugin, _>(asset_io::InMemoryAssetPlugin);
        #[cfg(not(feature = "bundled"))]
        group
    });

    #[cfg(target_arch = "wasm32")]
    builder.add_plugin(bevy_webgl2::WebGL2Plugin::default());

    builder
        .add_plugin(::bevy_easings::EasingsPlugin)
        .add_plugin(bevy_ninepatch::NinePatchPlugin::<()>::default());

    if cfg!(debug_assertions) {
        builder
            .add_plugin(::bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugin(::bevy_diagnostic_counter::EntityCountDiagnosticsPlugin)
            .add_plugin(::bevy_diagnostic_counter::AssetCountDiagnosticsPlugin::<ColorMaterial>::default())
            .add_plugin(::bevy_diagnostic_counter::AssetCountDiagnosticsPlugin::<Texture>::default())
            .add_plugin(::bevy_log_diagnostic::LogDiagnosticsPlugin::filtered(vec![
                ::bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS,
                ::bevy_diagnostic_counter::EntityCountDiagnosticsPlugin::ENTITY_COUNT,
                ::bevy_diagnostic_counter::AssetCountDiagnosticsPlugin::<ColorMaterial>::diagnostic_id(),
                ::bevy_diagnostic_counter::AssetCountDiagnosticsPlugin::<Texture>::diagnostic_id(),
            ]));
    }

    builder
        // game management
        .add_startup_system(general_setup)
        .add_system(handle_state)
        .add_resource(GameScreen::default())
        .add_resource(Config::default())
        .add_stage_after(bevy::app::stage::UPDATE, custom_stage::TEAR_DOWN)
        // ui
        .add_plugin(crate::ui::button::Plugin)
        .add_resource(AssetHandles::default())
        // collisions
        .add_plugin(bevy_rapier2d::physics::RapierPhysicsPlugin)
        // screens
        .add_plugin(crate::splash::Plugin)
        .add_plugin(crate::menu::Plugin)
        .add_plugin(crate::about::Plugin)
        .add_plugin(crate::game::Plugin)
        .add_plugin(crate::lost::Plugin)
        .add_plugin(crate::space::Plugin)
        .add_plugin(crate::win::Plugin)
        .run();

    Ok(())
}

pub mod custom_stage {
    pub const TEAR_DOWN: &str = "kmanb:tear_down";
}

#[derive(Debug, PartialEq, Clone)]
pub enum Screen {
    Splash,
    Menu,
    About,
    Game,
    Exit,
    Lost,
    Win,
}

#[derive(Debug)]
pub struct GameScreen {
    pub current_screen: Screen,
    pub highscore: u32,
}

impl Default for GameScreen {
    fn default() -> Self {
        GameScreen {
            current_screen: Screen::Splash,
            highscore: 0,
        }
    }
}

impl GameScreen {
    pub fn is_new_highscore(&self, score: u32) -> bool {
        self.highscore != 0 && score > self.highscore
    }
}

fn general_setup(
    commands: &mut Commands,
    mut configuration: ResMut<bevy_rapier2d::physics::RapierConfiguration>,
) {
    configuration.gravity = bevy_rapier2d::rapier::math::Vector::new(0., 0.);

    commands.spawn(Camera2dBundle::default());
    commands.spawn(UiCameraBundle::default());
}

fn handle_state(game_screen: Res<crate::GameScreen>, mut app_exit_events: ResMut<Events<AppExit>>) {
    if game_screen.current_screen == Screen::Exit {
        app_exit_events.send(AppExit);
    }
}
