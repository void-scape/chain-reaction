use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_seedling::prelude::*;
use dashu::ibig;

use crate::ball::{BallComponents, PlayerBall};
use crate::big::BigPoints;
use crate::collectables::Points;
use crate::sandbox;
use crate::state::{GameState, Playing, StateAppExt, remove_entities};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct StageSet;

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AdvanceEvent>()
            .add_reset(remove_entities::<With<Stage>>)
            .add_systems(OnEnter(GameState::StartGame), spawn_stage)
            .configure_sets(PreUpdate, StageSet.in_set(Playing));

        if !sandbox::ENABLED {
            app.add_systems(PreUpdate, stage.in_set(StageSet));
        }

        #[cfg(debug_assertions)]
        {
            use bevy_optix::debug::debug_single;
            app.add_systems(
                Update,
                debug_single::<Stage>(
                    Transform::from_xyz(-crate::RES_WIDTH / 2., -crate::RES_HEIGHT / 2., 500.),
                    Anchor::BottomLeft,
                ),
            );
        }
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
    pub points: BigPoints,
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

    pub fn progress(&mut self, acquired_points: BigPoints) -> bool {
        let progress = acquired_points.0 >= self.points.0;
        self.level += 1;
        self.points = Self::points(self.level);
        self.lives = Self::lives(self.level);

        progress
    }

    pub fn win(&self) -> bool {
        false
        //self.level >= 2
    }

    fn points(level: usize) -> BigPoints {
        let base = ibig!(2);
        let required = ibig!(10);
        let total = required * base.pow(level);
        BigPoints(total)

        // match level {
        //     0 => 0,
        //     _ => 20,
        //     //1 => 0,
        //     //2 => 0,
        //     //3 => 0,
        //     //4 => 0,
        //     //5 => 0,
        //     //6 => 0,
        //     //_ => todo!("points for level {}", level),
        // }
    }

    fn lives(_level: usize) -> usize {
        1
    }
}

fn stage(
    mut commands: Commands,
    server: Res<AssetServer>,
    points: Res<Points>,
    alive: Query<&BallComponents>,
    stage: Single<(Entity, &mut Stage)>,
) {
    if alive.is_empty() {
        let (entity, mut stage) = stage.into_inner();

        if stage.lives > 0 {
            stage.lives -= 1;
            commands.spawn((
                PlayerBall,
                Transform::from_xyz(
                    -crate::cabinet::WIDTH / 2. + 80.,
                    crate::cabinet::HEIGHT / 2. - 60.,
                    0.,
                ),
            ));

            commands.spawn(
                SamplePlayer::new(server.load("audio/pinball/1BootUp.ogg"))
                    .with_volume(Volume::Linear(0.5)),
            );
        } else {
            let mut entity = commands.entity(entity);
            if stage.progress(points.get().clone()) {
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

fn win(trigger: Trigger<OnAdd, Win>, mut commands: Commands, server: Res<AssetServer>) {
    commands.entity(trigger.target()).remove::<Win>();

    commands.set_state(GameState::Reset);
    commands.spawn(
        SamplePlayer::new(server.load("audio/pinball/1JACKPOT.ogg"))
            .with_volume(Volume::Linear(0.5)),
    );
}

fn loose(trigger: Trigger<OnAdd, Loose>, mut commands: Commands, server: Res<AssetServer>) {
    commands.entity(trigger.target()).remove::<Loose>();

    commands.set_state(GameState::Reset);
    commands.spawn(
        SamplePlayer::new(server.load("audio/pinball/1destroyed.ogg"))
            .with_volume(Volume::Linear(0.5)),
    );
}

#[derive(Event)]
pub struct AdvanceEvent {
    pub points: BigPoints,
    pub level: usize,
}

fn advance(
    trigger: Trigger<OnAdd, Advance>,
    mut commands: Commands,
    server: Res<AssetServer>,
    mut points: ResMut<Points>,
    stage: Single<&Stage>,
    mut writer: EventWriter<AdvanceEvent>,
) {
    commands.entity(trigger.target()).remove::<Advance>();

    writer.write(AdvanceEvent {
        points: points.get().clone(),
        level: stage.level - 1,
    });
    commands.spawn(
        SamplePlayer::new(server.load("audio/pinball/1JACKPOT.ogg"))
            .with_volume(Volume::Linear(0.5)),
    );
    points.reset();
}
