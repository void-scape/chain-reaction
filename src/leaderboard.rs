use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_enhanced_input::events::Fired;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_persistent::prelude::*;

use crate::big::BigPoints;
use crate::cabinet::{ScrollingTexture, Speed};
use crate::collectables::TotalPoints;
use crate::input::Enter;
use crate::stage::{AdvanceEvent, StageSet};
use crate::state::{GameState, StateAppExt, remove_entities};

pub struct LeaderBoardPlugin;

impl Plugin for LeaderBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_reset(remove_entities::<With<Leaderboard>>)
            .add_systems(Startup, player_data)
            .add_systems(PreUpdate, record_points.after(StageSet))
            .add_systems(
                OnEnter(GameState::Leaderboard),
                (spawn_leaderboard, background),
            )
            .add_observer(|_: Trigger<Fired<Enter>>, mut commands: Commands| {
                commands.run_system_cached(remove_entities::<With<Leaderboard>>);
                commands.set_state(GameState::ToGame);
            });
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize, Resource)]
struct PlayerData {
    /// (level, points)
    point_record: Vec<(usize, BigPoints)>,
}

fn player_data(mut commands: Commands) {
    // TODO: Rename
    let element = "chainreaction1";
    let config_dir = dirs::config_dir()
        .map(|native_config_dir| native_config_dir.join(element))
        .unwrap_or_else(|| std::path::Path::new("local").join(element));

    commands.insert_resource(
        Persistent::<PlayerData>::builder()
            .name("player data")
            .format(StorageFormat::Bincode)
            .path(config_dir.join("data"))
            .default(Default::default())
            .revertible(true)
            .revert_to_default_on_deserialization_errors(true)
            .build()
            .unwrap(),
    )
}

fn record_points(
    mut reader: EventReader<AdvanceEvent>,
    mut data: ResMut<Persistent<PlayerData>>,
    total_points: Res<TotalPoints>,
) {
    for event in reader.read() {
        data.point_record
            .push((event.level, total_points.get().clone()));
        if let Err(e) = data.persist() {
            error!("failed to save player data: {e}");
        }
    }
}

#[derive(Component)]
struct Leaderboard;

const LEADERZ: f32 = 800.;

fn background(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ScrollingTexture>>,
) {
    commands.spawn((
        Leaderboard,
        HIGH_RES_LAYER,
        Mesh2d(meshes.add(Rectangle::new(1024., 1024.))),
        Speed(Vec2::new(0.05, 0.1) * 0.5),
        MeshMaterial2d(mats.add(ScrollingTexture {
            uv_offset: Vec2::ZERO,
            texture: server.load_with_settings("textures/checkers.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    }),
                    ..default()
                }
            }),
        })),
    ));
}

fn spawn_leaderboard(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut data: ResMut<Persistent<PlayerData>>,
) {
    commands.spawn((
        Leaderboard,
        Text2d::new("LEADERBOARDS"),
        TextFont {
            font_size: 54.,
            font: server.load("fonts/saiba.ttf"),
            ..Default::default()
        },
        HIGH_RES_LAYER,
        Transform::from_xyz(0., crate::RES_HEIGHT / 3., LEADERZ),
    ));

    data.point_record.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    let largest_text = data
        .point_record
        .first()
        .map(|(level, points)| format!("S{level}   {points}").len())
        .unwrap_or_default();

    for (i, (level, points)) in data.point_record.iter().enumerate().take(10) {
        commands.spawn((
            Leaderboard,
            Text2d::new(format!("S{}   {points}", level + 1)),
            TextFont {
                font_size: 32.,
                ..Default::default()
            },
            HIGH_RES_LAYER,
            Anchor::CenterLeft,
            Transform::from_xyz(
                largest_text as f32 * -9.,
                crate::RES_HEIGHT / 3. - 80. - (i as f32 * 50.),
                LEADERZ,
            ),
        ));
    }

    if let Err(e) = data.persist() {
        error!("failed to save player data: {e}");
    }
}
