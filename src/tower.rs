use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_optix::debug::DebugCircle;
use bevy_optix::pixel_perfect::OuterCamera;

use crate::ball::Ball;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_tower, tower_cooldown::<Dispenser>));
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

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    Restitution::new(0.7),
    DebugCircle::new(18.),
    Collider::circle(18.),
    CollisionEventsEnabled
)]
struct Dispenser;

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
