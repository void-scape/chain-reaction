use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::color::palettes::css::GREY;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;

pub const CABZ: f32 = -100.;

pub struct CabinetPlugin;

impl Plugin for CabinetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_edges);
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
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(w, h), GREY),
        Collider::rectangle(w, h),
        Rotation::radians(-rot),
    ));

    commands.spawn((
        RigidBody::Static,
        Restitution::new(0.7),
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(w, h), GREY),
        Collider::rectangle(w, h),
        Rotation::radians(rot),
    ));

    let x = 65. + x;
    let y = 150. + y;

    let hh = 1000.;

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(h, hh), GREY),
        Collider::rectangle(h, hh),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(h, hh), GREY),
        Collider::rectangle(h, hh),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(0., crate::HEIGHT / 2., CABZ),
        DebugRect::from_size_color(Vec2::new(hh, 50.), GREY),
        Collider::rectangle(hh, 50.),
    ));
}
