use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::events::{Completed, Fired};
use bevy_optix::debug::DebugRect;
use bevy_seedling::{
    prelude::Volume,
    sample::{PitchRange, SamplePlayer},
};

use crate::{
    Avian, GameState,
    input::{PaddleDown, PaddleUp},
};

pub struct PaddlePlugin;

impl Plugin for PaddlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_edges)
            .add_systems(Avian, update_paddles.before(PhysicsSet::Prepare))
            .add_observer(apply_pressed)
            .add_observer(apply_released);
    }
}

#[derive(Component)]
pub struct Paddle;

const START_ROT: f32 = PI / 4.;
const END_OFFSET: f32 = PI / 3.;

fn spawn_edges(mut commands: Commands) {
    let w = 80.;
    let h = 10.;

    let x = 50.;
    let y = 25.;

    let fact = 1.4;

    // paddles
    commands.spawn((
        Paddle,
        RigidBody::Kinematic,
        Transform::from_xyz(-x * fact, -crate::HEIGHT / 2. + y + 15., 0.)
            .with_rotation(Quat::from_rotation_z(START_ROT)),
        //DebugRect::from_size(Vec2::new(w * 2., h)),
        Collider::capsule(h, w),
    ));

    commands.spawn((
        Paddle,
        RigidBody::Kinematic,
        Transform::from_xyz(x * fact, -crate::HEIGHT / 2. + y + 15., 0.)
            .with_rotation(Quat::from_rotation_z(-START_ROT)),
        //DebugRect::from_size(Vec2::new(w * 2., h)),
        Collider::capsule(h, w),
    ));

    let x = 90. + x;
    let y = 105. + y;

    let w = 200.;
    let h = 15.;

    let rot = PI / 3.5;

    // walls
    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(w, h)),
        Collider::rectangle(w, h),
        Rotation::radians(-rot),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(w, h)),
        Collider::rectangle(w, h),
        Rotation::radians(rot),
    ));

    let x = 65. + x;
    let y = 150. + y;

    let hh = 1000.;

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(h, hh)),
        Collider::rectangle(h, hh),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(h, hh)),
        Collider::rectangle(h, hh),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(0., crate::HEIGHT / 2., 0.),
        DebugRect::from_size(Vec2::new(hh, 50.)),
        Collider::rectangle(hh, 50.),
    ));
}

#[derive(Component)]
struct PaddleTarget(Quat);

const PADDLE_SPEED: f32 = 20.0;

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
            AngularVelocity(-sign * PADDLE_SPEED),
            PaddleTarget(Quat::from_rotation_z(sign * START_ROT)),
        ));
    }
}

fn update_paddles(
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
