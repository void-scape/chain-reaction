use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::{prelude::*, time::Stopwatch};
use bevy_enhanced_input::events::Fired;
use bevy_optix::debug::debug_single;
use bevy_seedling::{
    prelude::Volume,
    sample::{PitchRange, SamplePlayer},
};
use bevy_tween::{
    prelude::{AnimationBuilderExt, EaseKind},
    tween::IntoTarget,
};

use crate::{
    Avian, Layer,
    ball::{PaddleRestMult, paddle_mult},
    input::{PaddleDown, PaddleUp},
    state::{GameState, StateAppExt, remove_entities},
};

pub struct PaddlePlugin;

impl Plugin for PaddlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PaddleBonk>()
            .add_reset((
                remove_entities::<With<Paddle>>,
                remove_entities::<With<PaddleRest>>,
            ))
            .add_systems(
                Update,
                debug_single::<PaddleRest>(
                    Transform::from_xyz(
                        -crate::RES_WIDTH / 2.,
                        -crate::RES_HEIGHT / 2. + 50.,
                        500.,
                    ),
                    bevy::sprite::Anchor::BottomLeft,
                ),
            )
            .add_systems(OnEnter(GameState::StartGame), spawn_paddles)
            .add_systems(OnEnter(GameState::Playing), start_paddle_rest)
            .add_systems(OnExit(GameState::Playing), stop_paddle_rest)
            .add_systems(Update, paddle_rest)
            .add_systems(Avian, paddles.before(PhysicsSet::Prepare))
            .add_observer(apply_pressed)
            .add_observer(apply_released);
    }
}

#[derive(Component)]
#[require(
    CollisionEventsEnabled,
    CollisionLayers::new(Layer::Paddle, Layer::Ball)
)]
pub struct Paddle;

/// Duration that the paddle is inactive.
#[derive(Debug, Component)]
pub struct PaddleRest(pub Stopwatch);

fn paddle_rest(time: Res<Time>, mut paddle: Single<&mut PaddleRest>) {
    paddle.0.tick(time.delta());
}

fn stop_paddle_rest(mut paddle: Single<&mut PaddleRest>) {
    paddle.0.pause();
    paddle.0.reset();
}

fn start_paddle_rest(mut paddle: Single<&mut PaddleRest>) {
    paddle.0.unpause();
    paddle.0.reset();
}

/// Bonked a ball
#[derive(Event)]
pub struct PaddleBonk(pub Entity);

fn paddle_bonk(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    mut writer: EventWriter<PaddleBonk>,
    rest: Single<&PaddleRest>,
) {
    let animation = commands
        .animation()
        .insert_tween_here(
            Duration::from_secs_f32(rest.0.elapsed_secs()),
            EaseKind::ExponentialOut,
            trigger
                .collider
                .into_target()
                .with(paddle_mult(rest.0.elapsed_secs(), 0.)),
        )
        .id();

    writer.write(PaddleBonk(trigger.collider));
    commands
        .entity(trigger.collider)
        .insert(PaddleRestMult(rest.0.elapsed_secs()))
        .add_child(animation);
}

const START_ROT: f32 = PI / 7. + PI / 4.;
const END_OFFSET: f32 = PI / 3.;

fn spawn_paddles(mut commands: Commands) {
    let w = 90.;
    let h = 7.5;

    let x = 60.;
    let y = 20.;

    let fact = 1.4;

    // paddles
    commands
        .spawn((
            Paddle,
            RigidBody::Kinematic,
            Restitution::new(0.7),
            Transform::from_xyz(-x * fact, -crate::HEIGHT / 2. + y + 15., 0.)
                .with_rotation(Quat::from_rotation_z(START_ROT)),
            Collider::capsule(h, w),
        ))
        .observe(paddle_bonk);

    commands
        .spawn((
            Paddle,
            RigidBody::Kinematic,
            Restitution::new(0.7),
            Transform::from_xyz(x * fact, -crate::HEIGHT / 2. + y + 15., 0.)
                .with_rotation(Quat::from_rotation_z(-START_ROT)),
            Collider::capsule(h, w),
        ))
        .observe(paddle_bonk);

    let mut watch = Stopwatch::new();
    watch.pause();
    commands.spawn(PaddleRest(watch));
}

#[derive(Component)]
struct PaddleTarget(Quat);

const PADDLE_SPEED: f32 = 20.0;
const PADDLE_DOWN_SPEED: f32 = PADDLE_SPEED * 0.8;

fn apply_pressed(
    _trigger: Trigger<Fired<PaddleUp>>,
    paddles: Query<(Entity, &Transform), With<Paddle>>,
    mut rest: Single<&mut PaddleRest>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    rest.0.pause();

    commands.spawn((
        SamplePlayer::new(server.load("audio/pinball/FlipperUp.ogg"))
            .with_volume(Volume::Decibels(-12.0)),
        PitchRange(0.99..1.01),
    ));

    for (entity, position) in paddles.iter() {
        let sign = if position.translation.x > 0.0 {
            -1.
        } else {
            1.
        };

        commands.entity(entity).insert((
            AngularVelocity(sign * PADDLE_SPEED),
            PaddleTarget(Quat::from_rotation_z(sign * (START_ROT + END_OFFSET))),
        ));
    }
}

fn apply_released(
    _trigger: Trigger<Fired<PaddleDown>>,
    paddles: Query<(Entity, &Transform), With<Paddle>>,
    mut rest: Single<&mut PaddleRest>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
    rest.0.reset();
    rest.0.unpause();

    commands.spawn((
        SamplePlayer::new(server.load("audio/pinball/FlipperDown.ogg"))
            .with_volume(Volume::Decibels(-12.0)),
        PitchRange(0.99..1.01),
    ));

    for (entity, position) in paddles.iter() {
        let sign = if position.translation.x > 0.0 {
            -1.
        } else {
            1.
        };

        commands.entity(entity).insert((
            AngularVelocity(-sign * PADDLE_DOWN_SPEED),
            PaddleTarget(Quat::from_rotation_z(sign * START_ROT)),
        ));
    }
}

fn paddles(
    mut commands: Commands,
    mut paddles: Query<(Entity, &Transform, Option<&PaddleTarget>), With<Paddle>>,
) {
    for (entity, position, target) in paddles.iter_mut() {
        if let Some(target) = target {
            if (target.0.angle_between(position.rotation)).abs() < 0.15 {
                commands
                    .entity(entity)
                    .remove::<PaddleTarget>()
                    .insert(AngularVelocity::default());
            }
        }
    }
}
