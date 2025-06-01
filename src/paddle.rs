use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;

use crate::{Avian, GameState};

pub struct PaddlePlugin;

impl Plugin for PaddlePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PaddleBonk>()
            .add_systems(OnEnter(GameState::Playing), spawn_edges)
            .add_systems(Avian, paddles.before(PhysicsSet::Prepare));
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

const START_ROT: f32 = PI / 4.;
const END_OFFSET: f32 = PI / 3.;

fn spawn_edges(mut commands: Commands) {
    let w = 80.;
    let h = 10.;

    let x = 50.;
    let y = 25.;

    let fact = 1.4;

    // paddles
    commands
        .spawn((
            Paddle,
            RigidBody::Kinematic,
            Transform::from_xyz(-x * fact, -crate::HEIGHT / 2. + y + 15., 0.)
                .with_rotation(Quat::from_rotation_z(START_ROT)),
            Collider::capsule(h, w),
        ))
        .observe(paddle_bonk);

    commands
        .spawn((
            Paddle,
            RigidBody::Kinematic,
            Transform::from_xyz(x * fact, -crate::HEIGHT / 2. + y + 15., 0.)
                .with_rotation(Quat::from_rotation_z(-START_ROT)),
            Collider::capsule(h, w),
        ))
        .observe(paddle_bonk);

    let x = 90. + x;
    let y = 105. + y;

    let w = 200.;
    let h = 15.;

    let rot = PI / 3.5;

    // walls
    commands.spawn((
        RigidBody::Static,
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombine::Min,
        },
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(w, h)),
        Collider::rectangle(w, h),
        Rotation::radians(-rot),
    ));

    commands.spawn((
        RigidBody::Static,
        Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombine::Min,
        },
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

fn paddles(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut paddles: Query<(Entity, &Transform, Option<&PaddleTarget>), With<Paddle>>,
) {
    let speed = 20.;

    for (entity, position, target) in paddles.iter_mut() {
        let sign = if position.translation.x > 0.0 {
            -1.
        } else {
            1.
        };

        if let Some(target) = target {
            if (target.0.angle_between(position.rotation)).abs() < 0.15 {
                commands
                    .entity(entity)
                    .remove::<PaddleTarget>()
                    .insert(AngularVelocity::default());
            }
        }

        if input.just_pressed(KeyCode::Space) {
            commands.entity(entity).insert((
                AngularVelocity(sign * speed),
                PaddleTarget(Quat::from_rotation_z(sign * (START_ROT + END_OFFSET))),
            ));
        } else if input.just_released(KeyCode::Space) {
            commands.entity(entity).insert((
                AngularVelocity(-1. * sign * speed),
                PaddleTarget(Quat::from_rotation_z(sign * START_ROT)),
            ));
        }
    }
}
