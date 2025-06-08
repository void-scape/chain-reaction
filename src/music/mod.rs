use std::time::Duration;

use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_tween::{
    BevyTweenRegisterSystems,
    combinator::tween,
    component_dyn_tween_system, component_tween_system,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};

mod beats;
mod interpolators;

pub use beats::MusicBeats;
use interpolators::{InterpolateLowPass, InterpolateSampleSpeed, InterpolateVolume};

use crate::state::GameState;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<InterpolateLowPass>()
            .register_type::<InterpolateSampleSpeed>()
            .register_type::<InterpolateVolume>()
            .add_systems(Startup, spawn_music)
            .add_systems(PreUpdate, beats::update_beats)
            .add_tween_systems((
                component_tween_system::<InterpolateSampleSpeed>(),
                component_dyn_tween_system::<PlaybackSettings>(),
                component_tween_system::<InterpolateLowPass>(),
                component_dyn_tween_system::<LowPassNode>(),
                component_tween_system::<InterpolateVolume>(),
                component_dyn_tween_system::<VolumeNode>(),
            ))
            .add_systems(OnEnter(GameState::StartGame), tween_music_start)
            .add_systems(OnEnter(GameState::Leaderboard), tween_music_lose);
    }
}

#[derive(PoolLabel, PartialEq, Eq, Hash, Clone, Debug)]
struct MusicPool;

#[derive(Component)]
struct MusicLowPass;

const OUTSIDE_FREQ: f32 = 100.0;

fn spawn_music(mut commands: Commands, server: Res<AssetServer>) {
    let music_target = commands
        .spawn((
            SamplerPool(MusicPool),
            VolumeNode {
                volume: Volume::SILENT,
            },
        ))
        .chain_node((
            LowPassNode {
                frequency: OUTSIDE_FREQ,
            },
            MusicLowPass,
        ))
        .head();

    // fade in music
    let mut target = music_target.into_target().state(-96.0);
    commands.animation().insert(tween(
        Duration::from_secs(1),
        EaseKind::QuadraticOut,
        target.with(interpolators::volume_to(0.0)),
    ));

    commands.spawn((
        MusicPool,
        MusicBeats::new(100.0),
        SamplePlayer {
            sample: server.load("audio/music/pinball-nightclub.ogg"),
            repeat_mode: RepeatMode::RepeatEndlessly,
            volume: Volume::Decibels(-6.0),
        },
        PlaybackSettings::default(),
    ));
}

fn tween_music_start(
    music: Single<(Entity, &PlaybackSettings), (With<MusicBeats>, With<MusicPool>)>,
    low_pass: Single<(Entity, &LowPassNode), With<MusicLowPass>>,
    mut commands: Commands,
) {
    let (target, settings) = music.into_inner();
    let mut target = target.into_target().state(settings.speed as f32);

    commands.animation().insert(tween(
        Duration::from_secs_f32(2.5),
        EaseKind::QuadraticInOut,
        target.with(interpolators::sample_speed_to(1.0)),
    ));

    let (target, low_pass) = low_pass.into_inner();
    let mut target = target.into_target().state(low_pass.frequency);

    commands.animation().insert(tween(
        Duration::from_secs(1),
        EaseKind::QuadraticInOut,
        target.with(interpolators::low_pass_to(20_000.0)),
    ));
}

fn tween_music_lose(
    music: Single<(Entity, &PlaybackSettings), (With<MusicBeats>, With<MusicPool>)>,
    mut commands: Commands,
) {
    let (target, settings) = music.into_inner();
    let mut target = target.into_target().state(settings.speed as f32);

    commands.animation().insert(tween(
        Duration::from_secs(2),
        EaseKind::QuadraticOut,
        target.with(interpolators::sample_speed_to(0.4)),
    ));
}
