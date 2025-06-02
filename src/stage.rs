use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_optix::debug::debug_single;
use bevy_seedling::prelude::*;

use crate::ball::TowerBall;
use crate::collectables::Points;
use crate::state::{GameState, StateAppExt, remove_entities};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct StageSet;

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AdvanceEvent>()
            .add_reset(remove_entities::<With<Stage>>)
            .add_systems(OnEnter(GameState::Playing), spawn_stage)
            .add_systems(
                Update,
                debug_single::<Stage>(
                    Transform::from_xyz(-crate::RES_WIDTH / 2., -crate::RES_HEIGHT / 2., 500.),
                    Anchor::BottomLeft,
                ),
            )
            .add_systems(PreUpdate, stage.in_set(StageSet))
            .configure_sets(PreUpdate, StageSet.run_if(in_state(GameState::Playing)));
    }
}

// #####################################
//
// Observe `OnAdd` for components below.
//
// #####################################

#[derive(Component)]
pub struct Win;

#[derive(Component)]
pub struct Loose;

#[derive(Component)]
pub struct Advance;

fn spawn_stage(mut commands: Commands) {
    commands
        .spawn(Stage::new())
        .observe(win)
        .observe(loose)
        .observe(advance);
}

#[derive(Debug, Clone, Component)]
pub struct Stage {
    pub points: usize,
    pub level: usize,
    pub lives: usize,
}

impl Stage {
    pub fn new() -> Self {
        Self {
            points: Self::points(0),
            lives: Self::lives(0),
            level: 0,
        }
    }

    pub fn progress(&mut self, acquired_points: usize) -> bool {
        let progress = acquired_points >= self.points;
        self.level += 1;
        self.points = Self::points(self.level);
        self.lives = Self::lives(self.level);

        progress
    }

    pub fn win(&self) -> bool {
        self.level >= 2
    }

    fn points(level: usize) -> usize {
        match level {
            0 => 20,
            1 => 1_000,
            2 => 1_000,
            3 => 1_000,
            4 => 1_000,
            5 => 1_000,
            6 => 1_000,
            _ => todo!("points for level {}", level),
        }
    }

    fn lives(_level: usize) -> usize {
        1
    }
}

fn stage(
    mut commands: Commands,
    server: Res<AssetServer>,
    points: Res<Points>,
    alive: Query<&TowerBall>,
    stage: Single<(Entity, &mut Stage)>,
) {
    if alive.is_empty() {
        let (entity, mut stage) = stage.into_inner();

        let transform = Transform::from_xyz(-crate::WIDTH / 2. + 80., crate::HEIGHT / 2. - 20., 0.);

        if stage.lives > 0 {
            stage.lives -= 1;
            commands.spawn((TowerBall, transform));

            commands.spawn(
                SamplePlayer::new(server.load("audio/pinball/1BootUp.ogg"))
                    .with_volume(Volume::Linear(0.5)),
            );
        } else {
            let mut entity = commands.entity(entity);
            if stage.progress(points.get()) {
                if stage.win() {
                    entity.insert(Win);
                } else {
                    entity.insert(Advance);
                }
            } else {
                entity.insert(Loose);
            }
        }
    }
}

fn win(_: Trigger<OnAdd, Win>, mut commands: Commands) {
    info!("you win!");
    commands.set_state(GameState::Reset);
}

fn loose(_: Trigger<OnAdd, Loose>, mut commands: Commands, server: Res<AssetServer>) {
    commands.set_state(GameState::Leaderboard);
    commands.spawn(
        SamplePlayer::new(server.load("audio/pinball/1destroyed.ogg"))
            .with_volume(Volume::Linear(0.5)),
    );
}

#[derive(Event)]
pub struct AdvanceEvent {
    pub points: usize,
    pub level: usize,
}

fn advance(
    _: Trigger<OnAdd, Advance>,
    mut commands: Commands,
    server: Res<AssetServer>,
    mut points: ResMut<Points>,
    stage: Single<&Stage>,
    mut writer: EventWriter<AdvanceEvent>,
) {
    writer.write(AdvanceEvent {
        points: points.get(),
        level: stage.level - 1,
    });
    commands.spawn(
        SamplePlayer::new(server.load("audio/pinball/1JACKPOT.ogg"))
            .with_volume(Volume::Linear(0.5)),
    );
    points.reset();
}
