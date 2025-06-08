use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};

use crate::ball::PlayerBall;
use crate::feature::{FeatureSpawner, spawn_feature_list};
use crate::selection::{SelectedFeature, SelectionFeature};
use crate::state::{GameState, Playing};

pub const ENABLED: bool = true;

const CAM_OFFSET: f32 = crate::WIDTH / 3.;

const CORNER: Vec2 = Vec2::new(-crate::WIDTH + 100., crate::HEIGHT / 2. - 300.);
const GAP: Vec2 = Vec2::splat(80.);
const X: usize = 5;
const Y: usize = 5;

pub struct SandboxPlugin;

impl Plugin for SandboxPlugin {
    fn build(&self, app: &mut App) {
        if ENABLED {
            app.add_systems(Startup, move_camera)
                .add_systems(
                    OnEnter(GameState::StartGame),
                    spawn_selection.after(spawn_feature_list),
                )
                .add_systems(Update, spawn_ball.in_set(Playing));
        }
    }
}

fn move_camera(mut camera: Single<&mut Transform, With<OuterCamera>>) {
    camera.translation.x -= CAM_OFFSET;
}

fn spawn_ball(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
    selection_entity: Option<Single<&SelectedFeature>>,
) {
    if selection_entity.is_some() {
        return;
    }

    let (camera, gt) = camera.into_inner();
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
    else {
        return;
    };

    if world_position.x > crate::cabinet::WIDTH / 2.
        || world_position.x < -crate::cabinet::WIDTH / 2.
    {
        return;
    }

    commands.spawn((
        PlayerBall,
        Transform::from_translation(world_position.extend(0.)),
    ));
}

fn spawn_selection(mut commands: Commands, features: Query<&FeatureSpawner>) {
    let mut i = 0;
    for y in 0..Y {
        for x in 0..X {
            match features.iter().nth(i) {
                Some(spawner) => {
                    let position = CORNER + Vec2::new(x as f32, -(y as f32)) * GAP;
                    let mut entity = commands.spawn((
                        HIGH_RES_LAYER,
                        SelectionFeature,
                        Transform::from_translation(position.extend(0.) * crate::RESOLUTION_SCALE),
                    ));
                    spawner.0(&mut entity);
                    i += 1;
                }
                None => return,
            }
        }
    }
}
