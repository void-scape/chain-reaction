use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_optix::debug::{DebugCircle, DebugRect};

use crate::{Avian, GameState};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                |mut commands: Commands| commands.insert_resource(Lives(2)),
                spawn_edges,
            )
                .chain(),
        )
        .add_systems(Avian, update_paddles.before(PhysicsSet::Prepare))
        .add_systems(
            Update,
            (spawn_ball, despawn_ball).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource)]
pub struct Lives(usize);

#[derive(Component)]
#[require(
    RigidBody::Dynamic,
    Restitution::new(0.7),
    DebugCircle::new(12.),
    Collider::circle(12.)
)]
pub struct Ball;

fn spawn_ball(
    mut commands: Commands,
    #[cfg(debug_assertions)] input: Res<ButtonInput<KeyCode>>,
    #[cfg(not(debug_assertions))] mut lives: ResMut<Lives>,
    #[cfg(not(debug_assertions))] alive: Query<&Ball>,
) {
    #[cfg(not(debug_assertions))]
    let cond = alive.is_empty();
    #[cfg(debug_assertions)]
    let cond = input.just_pressed(KeyCode::KeyA);

    if cond {
        let transform = Transform::from_xyz(-crate::WIDTH / 2. + 40., crate::HEIGHT / 2. - 20., 0.);

        #[cfg(not(debug_assertions))]
        if lives.0 > 0 {
            lives.0 -= 1;
            commands.spawn((Ball, transform));
        }

        #[cfg(debug_assertions)]
        commands.spawn((Ball, transform));
    }
}

fn despawn_ball(mut commands: Commands, balls: Query<(Entity, &Transform)>) {
    for (entity, transform) in balls.iter() {
        if transform.translation.y < -crate::HEIGHT / 2. - 12. {
            commands.entity(entity).despawn();
        }
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

    let x = 75. + x;
    let y = 150. + y;

    let hh = 1000.;

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(h, hh)),
        Collider::rectangle(50., hh),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(h, hh)),
        Collider::rectangle(50., hh),
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

fn update_paddles(
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
