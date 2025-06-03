use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_enhanced_input::events::Fired;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_persistent::prelude::*;

use crate::input::Enter;
use crate::stage::{AdvanceEvent, StageSet};
use crate::state::{GameState, StateAppExt, remove_entities};

pub struct LeaderBoardPlugin;

impl Plugin for LeaderBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_reset(remove_entities::<With<Leaderboard>>)
            .add_systems(Startup, player_data)
            .add_systems(PreUpdate, record_points.after(StageSet))
            .add_systems(OnEnter(GameState::Leaderboard), spawn_leaderboard)
            .add_observer(|_: Trigger<Fired<Enter>>, mut commands: Commands| {
                commands.set_state(GameState::Reset);
            });
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize, Resource)]
struct PlayerData {
    /// (level, points)
    point_record: Vec<(usize, usize)>,
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
            .build()
            .unwrap(),
    )
}

fn record_points(mut reader: EventReader<AdvanceEvent>, mut data: ResMut<Persistent<PlayerData>>) {
    for event in reader.read() {
        data.point_record.push((event.level, event.points));
        if let Err(e) = data.persist() {
            error!("failed to save player data: {e}");
        }
    }
}

#[derive(Component)]
struct Leaderboard;

const LEADERZ: f32 = 800.;

fn spawn_leaderboard(
    mut commands: Commands,
    _server: Res<AssetServer>,
    mut data: ResMut<Persistent<PlayerData>>,
) {
    commands.spawn((
        Leaderboard,
        Sprite::from_color(
            Color::BLACK.with_alpha(0.95),
            Vec2::new(crate::RES_WIDTH, crate::RES_HEIGHT),
        ),
        Transform::from_xyz(0., 0., LEADERZ - 1.),
    ));

    commands.spawn((
        Leaderboard,
        Text2d::new("LEADERBOARDS"),
        HIGH_RES_LAYER,
        Transform::from_xyz(0., crate::RES_HEIGHT / 3., LEADERZ),
    ));

    data.point_record
        .sort_by_key(|(_, score)| std::cmp::Reverse(*score));
    let largest_text = data
        .point_record
        .first()
        .map(|(level, points)| format!("Stage: {level}    Points: {points}").len())
        .unwrap_or_default();

    for (i, (level, points)) in data.point_record.iter().enumerate().take(15) {
        commands.spawn((
            Leaderboard,
            Text2d::new(format!("Stage: {}    Points: {points}", level + 1)),
            HIGH_RES_LAYER,
            Anchor::CenterLeft,
            Transform::from_xyz(
                largest_text as f32 * -5.,
                crate::RES_HEIGHT / 3. - 40. - (i as f32 * 20.),
                LEADERZ,
            ),
        ));
    }

    if let Err(e) = data.persist() {
        error!("failed to save player data: {e}");
    }
}
