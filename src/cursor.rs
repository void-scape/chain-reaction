use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cursor)
            .add_systems(Update, move_cursor);
    }
}

#[derive(Component)]
struct GameCursor;

fn setup_cursor(
    #[cfg(not(debug_assertions))] mut window: Single<&mut Window, With<PrimaryWindow>>,
    #[cfg(debug_assertions)] window: Single<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    #[cfg(not(debug_assertions))]
    {
        window.cursor_options.visible = false;
    }
    commands.spawn((
        HIGH_RES_LAYER,
        Sprite {
            image: server.load("textures/cursor.png"),
            anchor: Anchor::TopLeft,
            ..Default::default()
        },
        Transform::from_translation(window.cursor_position().unwrap_or_default().extend(990.)),
        GameCursor,
    ));
}

fn move_cursor(
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
    mut cursor: Single<&mut Transform, With<GameCursor>>,
) {
    let (camera, gt) = camera.into_inner();
    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
    else {
        return;
    };

    cursor.translation.x = world_position.x;
    cursor.translation.y = world_position.y;
}
