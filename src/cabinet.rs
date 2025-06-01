use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;

use crate::GameState;

pub struct CabinetPlugin;

impl Plugin for CabinetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_edges);
    }
}

fn spawn_edges(mut commands: Commands) {
    let x = 160.;
    let y = 130.;

    let w = 180.;
    let h = 15.;

    let rot = PI / 4.5;

    // walls
    commands.spawn((
        RigidBody::Static,
        Restitution::new(0.7),
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, 0.),
        DebugRect::from_size(Vec2::new(w, h)),
        Collider::rectangle(w, h),
        Rotation::radians(-rot),
    ));

    commands.spawn((
        RigidBody::Static,
        Restitution::new(0.7),
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
