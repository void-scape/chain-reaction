use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::events::Fired;
use bevy_seedling::{
    prelude::Volume,
    sample::{PitchRange, SamplePlayer},
};

use crate::{
    Avian,
    input::{PaddleDown, PaddleUp},
    state::{GameState, StateAppExt, remove_entities},
};

pub struct PaddlePlugin;

impl Plugin for PaddlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PaddleBonk>()
            .add_reset(remove_entities::<With<Paddle>>)
            .add_systems(OnEnter(GameState::StartGame), spawn_paddles)
            .add_systems(Avian, paddles.before(PhysicsSet::Prepare))
            .add_observer(apply_pressed)
            .add_observer(apply_released);
    }
}

#[derive(Component)]
#[require(CollisionEventsEnabled)]
pub struct Paddle;

/// Bonked a ball
#[derive(Event)]
pub struct PaddleBonk(pub Entity);

fn paddle_bonk(trigger: Trigger<OnCollisionStart>, mut writer: EventWriter<PaddleBonk>) {
    writer.write(PaddleBonk(trigger.collider));
}

const START_ROT: f32 = PI / 7. + PI / 4.;
const END_OFFSET: f32 = PI / 3.;

fn spawn_paddles(mut commands: Commands) {
    let w = 90.;
    let h = 7.5;

    let x = 60.;
    let y = 50.;

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
}

#[derive(Component)]
struct PaddleTarget(Quat);

const PADDLE_SPEED: f32 = 20.0;
const PADDLE_DOWN_SPEED: f32 = PADDLE_SPEED * 0.8;

fn apply_pressed(
    _trigger: Trigger<Fired<PaddleUp>>,
    paddles: Query<(Entity, &Transform), With<Paddle>>,
    mut commands: Commands,
    server: Res<AssetServer>,
) {
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
    mut commands: Commands,
    server: Res<AssetServer>,
) {
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
