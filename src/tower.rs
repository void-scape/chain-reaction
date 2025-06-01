use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::color::palettes::css::{GREEN, RED};
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;
use rand::Rng;
use strum_macros::EnumIter;

#[cfg(debug_assertions)]
use bevy::window::PrimaryWindow;
#[cfg(debug_assertions)]
use bevy_optix::pixel_perfect::OuterCamera;

use crate::ball::Ball;
use crate::sampler::Sampler;

pub const TOWER_SIZE: f32 = 36.0;
pub const TOWER_RADIUS: f32 = TOWER_SIZE / 2.;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, tower_cooldown::<Dispenser>);

        #[cfg(debug_assertions)]
        app.add_systems(Update, spawn_tower);
    }
}

fn spawn_tower(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
) {
    let (camera, gt) = camera.into_inner();

    if input.just_pressed(MouseButton::Left) {
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
            .map(|ray| ray.origin.truncate())
        {
            commands
                .spawn((
                    Dispenser,
                    Transform::from_translation(
                        (world_position / crate::RESOLUTION_SCALE).extend(0.),
                    ),
                ))
                .observe(dispense);
        }
    }
}

/// Temporarily disable the effect of a tower collision for a [`Ball`].
#[derive(Component)]
pub struct TowerCooldown<T: 'static> {
    timer: Timer,
    _tower: PhantomData<fn() -> T>,
}

impl<T> TowerCooldown<T> {
    pub fn from_seconds(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            _tower: PhantomData,
        }
    }
}

fn tower_cooldown<T>(
    mut commands: Commands,
    time: Res<Time>,
    mut cooldowns: Query<(Entity, &mut TowerCooldown<T>)>,
) {
    for (entity, mut cooldown) in cooldowns.iter_mut() {
        cooldown.timer.tick(time.delta());
        if cooldown.timer.finished() {
            commands.entity(entity).remove::<TowerCooldown<T>>();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum Tower {
    Bumper,
    Dispenser,
}

impl Tower {
    pub fn spawn_random(commands: &mut Commands, rng: &mut impl Rng, bundle: impl Bundle) {
        match Sampler::new(&[(Self::Bumper, 1.), (Self::Dispenser, 0.5)]).sample(rng) {
            Tower::Bumper => {
                commands.spawn((Bumper, bundle));
            }
            Tower::Dispenser => {
                commands.spawn((Dispenser, bundle)).observe(dispense);
            }
        }
    }
}

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    DebugCircle::color(TOWER_RADIUS, RED),
    Collider::circle(TOWER_RADIUS)
)]
pub struct Bumper;

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    DebugCircle::color(TOWER_RADIUS, GREEN),
    Collider::circle(TOWER_RADIUS),
    CollisionEventsEnabled
)]
pub struct Dispenser;

fn dispense(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    filtered: Query<&TowerCooldown<Dispenser>>,
    transforms: Query<&Transform, With<Dispenser>>,
) {
    if filtered.contains(trigger.collider) {
        return;
    }

    if let Ok(mut transform) = transforms.get(trigger.target()).copied() {
        transform.translation.y -= 12.;
        commands.spawn((
            Ball,
            TowerCooldown::<Dispenser>::from_seconds(0.5),
            transform,
        ));
    }
}
