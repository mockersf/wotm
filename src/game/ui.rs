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
