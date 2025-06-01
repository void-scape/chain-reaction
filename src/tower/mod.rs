use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::color::palettes::css::{GREEN, RED};
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;
use grid::{SlotTower, SlotTowerOf, TowerSlot};
use rand::Rng;
use strum_macros::EnumIter;

#[cfg(debug_assertions)]
use bevy::window::PrimaryWindow;
#[cfg(debug_assertions)]
use bevy_optix::pixel_perfect::OuterCamera;

use crate::ball::{Ball, TowerBall};
use crate::sampler::Sampler;
use crate::{Avian, GameState, Layer};

mod grid;

pub const TOWER_SIZE: f32 = 36.0;
pub const TOWER_RADIUS: f32 = TOWER_SIZE / 2.;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_tower_zone)
            .add_systems(
                Update,
                (tower_cooldown::<Dispenser>, grid::TowerGrid::spawn_slots),
            )
            .add_systems(Avian, despawn_empty_bonks.before(PhysicsSet::Prepare))
            .add_observer(bonks)
            .add_observer(bonk_bounce);

        #[cfg(debug_assertions)]
        app.add_systems(Update, spawn_tower);
    }
}

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    Collider::rectangle(crate::WIDTH / 2., crate::HEIGHT / 3.),
    CollisionLayers::new(Layer::TowerZone, Layer::TowerBall),
    CollisionEventsEnabled,
    Sensor,
    grid::TowerGrid { spacing: Vec2::new(75.0, 75.0) }
)]
pub struct TowerZone;

fn spawn_tower_zone(mut commands: Commands) {
    commands
        .spawn((TowerZone, Transform::from_xyz(0., 50., 0.)))
        .observe(validate_balls)
        .observe(invalidate_balls);
}

/// Marks a [`TowerBall`] as a valid target for constructing a new tower.
#[derive(Component)]
pub struct ValidZone;

fn validate_balls(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    invalid_balls: Query<Entity, (With<TowerBall>, Without<ValidZone>)>,
) {
    if let Ok(entity) = invalid_balls.get(trigger.collider) {
        commands.entity(entity).insert(ValidZone);
    }
}

fn invalidate_balls(
    trigger: Trigger<OnCollisionEnd>,
    mut commands: Commands,
    valid_balls: Query<Entity, (With<TowerBall>, With<ValidZone>)>,
) {
    if let Ok(entity) = valid_balls.get(trigger.collider) {
        commands.entity(entity).remove::<ValidZone>();
    }
}

fn spawn_tower(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
    slots: Query<(Entity, &GlobalTransform), (With<TowerSlot>, Without<SlotTower>)>,
) {
    let (camera, gt) = camera.into_inner();
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate())
    else {
        return;
    };

    // check for the nearest tower slot within some threshold

    let Some((nearest_slot, transform)) = slots.iter().min_by(|a, b| {
        let a = world_position.distance_squared(a.1.compute_transform().translation.xy());
        let b = world_position.distance_squared(b.1.compute_transform().translation.xy());

        a.total_cmp(&b)
    }) else {
        return;
    };

    if transform
        .compute_transform()
        .translation
        .xy()
        .distance(world_position)
        > 50.0
    {
        return;
    }

    info!("spawning tower!");

    commands
        .spawn((
            Dispenser,
            SlotTowerOf(nearest_slot),
            ChildOf(nearest_slot),
            Transform::default(),
        ))
        .observe(dispense);

    // if let Some(world_position) = window
    //     .cursor_position()
    //     .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
    //     .map(|ray| ray.origin.truncate())
    // {
    //     commands
    //         .spawn((
    //             Dispenser,
    //             Transform::from_translation(
    //                 (world_position / crate::RESOLUTION_SCALE).extend(0.),
    //             ),
    //         ))
    //         .observe(dispense);
    // }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Component)]
#[require(Bonks(1), BonkImpulse(1.))]
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

/// The number of bonks before the tower despawns.
#[derive(Component)]
#[require(RigidBody::Kinematic, CollisionEventsEnabled)]
struct Bonks(usize);

fn bonks(trigger: Trigger<OnCollisionStart>, mut bonks: Query<&mut Bonks>) {
    if let Ok(mut bonks) = bonks.get_mut(trigger.target()) {
        bonks.0 = bonks.0.saturating_sub(1);
    }
}

fn despawn_empty_bonks(mut commands: Commands, bonks: Query<(Entity, &Bonks)>) {
    for (entity, _) in bonks.iter().filter(|(_, bonks)| bonks.0 == 0) {
        commands.entity(entity).despawn();
    }
}

/// The factor applied to the impulse generated by a bonk.
#[derive(Component)]
struct BonkImpulse(f32);

#[derive(Component)]
#[require(
    Tower::Bumper,
    BonkImpulse(2.),
    Bonks(10),
    DebugCircle::color(TOWER_RADIUS, RED),
    Collider::circle(TOWER_RADIUS)
)]
pub struct Bumper;

#[derive(Component)]
#[require(
    Tower::Dispenser,
    Bonks(10),
    DebugCircle::color(TOWER_RADIUS, GREEN),
    Collider::circle(TOWER_RADIUS)
)]
pub struct Dispenser;

fn dispense(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    filtered: Query<&TowerCooldown<Dispenser>>,
    transforms: Query<&GlobalTransform, With<Dispenser>>,
) {
    if filtered.contains(trigger.collider) {
        return;
    }

    if let Ok(transform) = transforms.get(trigger.target()).copied() {
        let mut transform = transform.compute_transform();
        transform.translation.y -= 12.;
        commands.spawn((
            Ball,
            TowerCooldown::<Dispenser>::from_seconds(0.5),
            transform,
        ));
    }
}

fn bonk_bounce(
    trigger: Trigger<OnCollisionStart>,
    towers: Query<(&Transform, &BonkImpulse), With<Tower>>,
    mut balls: Query<(&Transform, &mut ExternalImpulse), Or<(With<Ball>, With<TowerBall>)>>,
) {
    match (
        towers.get(trigger.target()),
        balls.get_mut(trigger.collider),
    ) {
        (Ok((transform, mult)), Ok((ball_transform, mut impulse))) => {
            impulse.apply_impulse(
                (ball_transform.translation.xy() - transform.translation.xy()).normalize_or_zero()
                    * 50_000.
                    * mult.0,
            );
        }
        _ => {}
    }
}
