use bevy::prelude::*;

macro_rules! load {
    ($assets:ident, $path:expr) => {
        $assets.load($path);
    };
}

macro_rules! colormaterial {
    ($mats:ident, $assets:ident, $path:expr) => {
        $mats.add($assets.load($path).into())
    };
    ($mats:ident, $assets:ident, $path:expr, $color:ident) => {
        $mats.add(ColorMaterial {
            texture: Some($assets.load($path)),
            color: $color,
        });
    };
}

#[derive(Default, Clone)]
pub struct AssetHandles {
    panel_handle: Option<(
        Handle<bevy_ninepatch::NinePatchBuilder<()>>,
        Handle<Texture>,
    )>,
    menu_panel_handle: Option<(
        Handle<bevy_ninepatch::NinePatchBuilder<()>>,
        Handle<Texture>,
    )>,
    button_handle: Option<Handle<crate::ui::button::Button>>,
    selection_handle: Option<Handle<ColorMaterial>>,
    font_main_handle: Option<Handle<Font>>,
    font_sub_handle: Option<Handle<Font>>,
    game: Option<GameHandles>,

    color_spawning_self: Option<Handle<ColorMaterial>>,
    color_spawning_enemy: Option<Handle<ColorMaterial>>,
    color_spawning_neutral: Option<Handle<ColorMaterial>>,
    color_selected_self: Option<Handle<ColorMaterial>>,
    color_selected_other: Option<Handle<ColorMaterial>>,
    color_highlighted_self: Option<Handle<ColorMaterial>>,
    color_highlighted_other: Option<Handle<ColorMaterial>>,
}

#[derive(Clone)]
pub struct GameHandles {
    pub planets: Vec<(Handle<ColorMaterial>, u32)>,
    pub orbiters: Vec<Handle<ColorMaterial>>,
    pub ships: Vec<Vec<Handle<ColorMaterial>>>,
    pub explosion_handle: Handle<TextureAtlas>,
}

impl AssetHandles {
    pub fn get_planet_names(&self) -> Vec<&'static str> {
        include!("../assets/star_names.in").to_vec()
    }

    pub fn get_panel_handle(
        &mut self,
        assets: &AssetServer,
        nine_patches: &mut Assets<bevy_ninepatch::NinePatchBuilder<()>>,
    ) -> (
        Handle<bevy_ninepatch::NinePatchBuilder<()>>,
        Handle<Texture>,
    ) {
        if self.panel_handle.is_none() {
            let panel_texture_handle = assets.load("ui/glassPanel_right_corners.png");
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(20, 20, 20, 10);
            self.panel_handle = Some((nine_patches.add(np), panel_texture_handle));
        };
        self.panel_handle.as_ref().unwrap().clone()
    }

    pub fn get_menu_panel_handle(
        &mut self,
        assets: &AssetServer,
        nine_patches: &mut Assets<bevy_ninepatch::NinePatchBuilder<()>>,
    ) -> (
        Handle<bevy_ninepatch::NinePatchBuilder<()>>,
        Handle<Texture>,
    ) {
        if self.menu_panel_handle.is_none() {
            let panel_texture_handle = assets.load("ui/glassPanel_corners.png");
            let np = bevy_ninepatch::NinePatchBuilder::by_margins(20, 20, 20, 20);
            self.menu_panel_handle = Some((nine_patches.add(np), panel_texture_handle));
        };
        self.menu_panel_handle.as_ref().unwrap().clone()
    }

    pub fn get_button_handle(
        &mut self,
        assets: &AssetServer,
        mut mats: &mut ResMut<Assets<ColorMaterial>>,
        mut nine_patches: &mut Assets<bevy_ninepatch::NinePatchBuilder<()>>,

        buttons: &mut Assets<crate::ui::button::Button>,
    ) -> Handle<crate::ui::button::Button> {
        if self.button_handle.is_none() {
            let button_texture_handle = assets.load("ui/glassPanel_projection.png");
            let button = crate::ui::button::Button::setup(
                &mut mats,
                &mut nine_patches,
                button_texture_handle,
            );
            self.button_handle = Some(buttons.add(button));
        };
        self.button_handle.as_ref().unwrap().clone()
    }

    pub fn get_ui_selection_handle(
        &mut self,
        assets: &AssetServer,
        mats: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.selection_handle.is_none() {
            self.selection_handle = Some(colormaterial!(mats, assets, "ui/dotBlue.png"));
        }
        self.selection_handle.as_ref().unwrap().clone()
    }

    pub fn get_font_main_handle(&mut self, assets: &AssetServer) -> Handle<Font> {
        if self.font_main_handle.is_none() {
            self.font_main_handle = Some(load!(assets, "fonts/kenvector_future.ttf"));
        }
        self.font_main_handle.as_ref().unwrap().clone()
    }

    pub fn get_font_sub_handle(&mut self, assets: &AssetServer) -> Handle<Font> {
        if self.font_sub_handle.is_none() {
            self.font_sub_handle = Some(load!(assets, "fonts/mandrill.ttf"));
        }
        self.font_sub_handle.as_ref().unwrap().clone()
    }

    pub fn get_color_spawning_self(
        &mut self,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.color_spawning_self.is_none() {
            self.color_spawning_self = Some(materials.add(Color::hex("43BA25").unwrap().into()));
        }
        self.color_spawning_self.as_ref().unwrap().clone()
    }

    pub fn get_color_spawning_neutral(
        &mut self,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.color_spawning_neutral.is_none() {
            self.color_spawning_neutral = Some(materials.add(Color::hex("222222").unwrap().into()));
        }
        self.color_spawning_neutral.as_ref().unwrap().clone()
    }

    pub fn get_color_spawning_enemy(
        &mut self,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.color_spawning_enemy.is_none() {
            self.color_spawning_enemy = Some(materials.add(Color::hex("FA314A").unwrap().into()));
        }
        self.color_spawning_enemy.as_ref().unwrap().clone()
    }

    pub fn get_color_selected_self(
        &mut self,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.color_selected_self.is_none() {
            self.color_selected_self = Some(materials.add(Color::hex("1F7AFF").unwrap().into()));
        }
        self.color_selected_self.as_ref().unwrap().clone()
    }

    pub fn get_color_selected_other(
        &mut self,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.color_selected_other.is_none() {
            self.color_selected_other = Some(materials.add(Color::hex("F3FA66").unwrap().into()));
        }
        self.color_selected_other.as_ref().unwrap().clone()
    }

    pub fn get_color_highlighted_self(
        &mut self,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.color_highlighted_self.is_none() {
            self.color_highlighted_self = Some(materials.add(Color::hex("0F3C80").unwrap().into()));
        }
        self.color_highlighted_self.as_ref().unwrap().clone()
    }

    pub fn get_color_highlighted_other(
        &mut self,
        materials: &mut Assets<ColorMaterial>,
    ) -> Handle<ColorMaterial> {
        if self.color_highlighted_other.is_none() {
            self.color_highlighted_other =
                Some(materials.add(Color::hex("7C8034").unwrap().into()));
        }
        self.color_highlighted_other.as_ref().unwrap().clone()
    }

    fn build_explosion_atlas(texture: Handle<Texture>) -> TextureAtlas {
        let mut atlas = TextureAtlas::new_empty(texture, Vec2::new(512., 512.));
        atlas.add_texture(bevy::sprite::Rect {
            min: Vec2::new(0., 0.),
            max: Vec2::new(192., 192.),
        });
        atlas.add_texture(bevy::sprite::Rect {
            min: Vec2::new(0., 192.),
            max: Vec2::new(152., 342.),
        });
        atlas.add_texture(bevy::sprite::Rect {
            min: Vec2::new(292., 360.),
            max: Vec2::new(374., 451.),
        });
        atlas.add_texture(bevy::sprite::Rect {
            min: Vec2::new(292., 258.),
            max: Vec2::new(384., 360.),
        });
        atlas.add_texture(bevy::sprite::Rect {
            min: Vec2::new(290., 134.),
            max: Vec2::new(410., 258.),
        });
        atlas.add_texture(bevy::sprite::Rect {
            min: Vec2::new(192., 0.),
            max: Vec2::new(325., 134.),
        });
        atlas.add_texture(bevy::sprite::Rect {
            min: Vec2::new(152., 192.),
            max: Vec2::new(290., 332.),
        });

        atlas
    }

    pub fn get_game_handles(
        &mut self,
        assets: &AssetServer,
        mats: &mut Assets<ColorMaterial>,
        atlases: &mut Assets<TextureAtlas>,
    ) -> GameHandles {
        if self.game.is_none() {
            self.game = Some(GameHandles {
                planets: vec![
                    (colormaterial!(mats, assets, "planets/1.png"), 365),
                    (colormaterial!(mats, assets, "planets/2.png"), 340),
                    (colormaterial!(mats, assets, "planets/3.png"), 390),
                    (colormaterial!(mats, assets, "planets/4.png"), 250),
                    (colormaterial!(mats, assets, "planets/5.png"), 395),
                    (colormaterial!(mats, assets, "planets/6.png"), 360),
                    (colormaterial!(mats, assets, "planets/7.png"), 430),
                    (colormaterial!(mats, assets, "planets/8.png"), 430),
                    (colormaterial!(mats, assets, "planets/9.png"), 395),
                    (colormaterial!(mats, assets, "planets/10.png"), 400),
                    (colormaterial!(mats, assets, "planets/11.png"), 400),
                    (colormaterial!(mats, assets, "planets/12.png"), 395),
                    (colormaterial!(mats, assets, "planets/13.png"), 375),
                    (colormaterial!(mats, assets, "planets/14.png"), 370),
                    (colormaterial!(mats, assets, "planets/15.png"), 385),
                    (colormaterial!(mats, assets, "planets/16.png"), 380),
                    (colormaterial!(mats, assets, "planets/17.png"), 385),
                    (colormaterial!(mats, assets, "planets/18.png"), 385),
                    (colormaterial!(mats, assets, "planets/19.png"), 370),
                    (colormaterial!(mats, assets, "planets/20.png"), 390),
                ],
                orbiters: vec![
                    colormaterial!(mats, assets, "Station/spaceStation_017.png"),
                    colormaterial!(mats, assets, "Station/spaceStation_018.png"),
                    colormaterial!(mats, assets, "Station/spaceStation_024.png"),
                    colormaterial!(mats, assets, "Station/spaceStation_026.png"),
                    colormaterial!(mats, assets, "Meteors/spaceMeteors_001.png"),
                    colormaterial!(mats, assets, "Meteors/spaceMeteors_002.png"),
                    colormaterial!(mats, assets, "Meteors/spaceMeteors_003.png"),
                    colormaterial!(mats, assets, "Meteors/spaceMeteors_004.png"),
                ],
                ships: vec![
                    vec![
                        colormaterial!(mats, assets, "Ships/enemyBlue1.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlue2.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlue3.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlue4.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlue5.png"),
                    ],
                    vec![
                        colormaterial!(mats, assets, "Ships/enemyGreen1.png"),
                        colormaterial!(mats, assets, "Ships/enemyGreen2.png"),
                        colormaterial!(mats, assets, "Ships/enemyGreen3.png"),
                        colormaterial!(mats, assets, "Ships/enemyGreen4.png"),
                        colormaterial!(mats, assets, "Ships/enemyGreen5.png"),
                    ],
                    vec![
                        colormaterial!(mats, assets, "Ships/enemyRed1.png"),
                        colormaterial!(mats, assets, "Ships/enemyRed2.png"),
                        colormaterial!(mats, assets, "Ships/enemyRed3.png"),
                        colormaterial!(mats, assets, "Ships/enemyRed4.png"),
                        colormaterial!(mats, assets, "Ships/enemyRed5.png"),
                    ],
                    vec![
                        colormaterial!(mats, assets, "Ships/enemyBlack1.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlack2.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlack3.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlack4.png"),
                        colormaterial!(mats, assets, "Ships/enemyBlack5.png"),
                    ],
                ],
                explosion_handle: atlases.add(Self::build_explosion_atlas(
                    assets.load("spritesheet_regularExplosion.png"),
                )),
            });
        }
        self.game.as_ref().unwrap().clone()
    }

    pub fn get_game_handles_unsafe(&self) -> GameHandles {
        self.game.as_ref().unwrap().clone()
    }
}
