use std::marker::PhantomData;

use avian2d::prelude::*;
use bevy::color::palettes::css::{GREEN, RED};
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::SamplePlayer;
use grid::{SlotTower, SlotTowerOf, TowerSlot};
use strum_macros::EnumIter;

#[cfg(debug_assertions)]
use bevy::window::PrimaryWindow;
#[cfg(debug_assertions)]
use bevy_optix::pixel_perfect::OuterCamera;

use crate::ball::{Ball, TowerBall};
use crate::collectables::PointEvent;
use crate::state::{GameState, StateAppExt, remove_entities};
use crate::{Avian, Layer};

use self::grid::TowerGrid;

mod grid;

pub const TOWER_SIZE: f32 = 36.0;
pub const TOWER_RADIUS: f32 = TOWER_SIZE / 2.;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_reset((
            remove_entities::<With<Tower>>,
            remove_entities::<With<TowerGrid>>,
        ))
        .add_systems(OnEnter(GameState::Playing), spawn_tower_zone)
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
    Collider::rectangle(crate::WIDTH / 1.5, crate::HEIGHT / 1.5),
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
    digits: Res<ButtonInput<KeyCode>>,
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
    slots: Query<(Entity, &GlobalTransform), (With<TowerSlot>, Without<SlotTower>)>,

    mut selection: Local<Tower>,
) {
    if digits.just_pressed(KeyCode::Digit1) {
        *selection = Tower::Bumper;
    } else if digits.just_pressed(KeyCode::Digit2) {
        *selection = Tower::Dispenser;
    }

    let (camera, gt) = camera.into_inner();
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
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

    selection.spawn(
        &mut commands,
        (SlotTowerOf(nearest_slot), ChildOf(nearest_slot)),
    );
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, EnumIter, Component)]
#[require(Bonks::Unlimited, BonkImpulse(1.))]
pub enum Tower {
    #[default]
    Bumper,
    Dispenser,
}

impl Tower {
    //pub fn spawn_random(commands: &mut Commands, rng: &mut impl Rng, bundle: impl Bundle) {
    //    Sampler::new(&[(Self::Bumper, 1.), (Self::Dispenser, 0.5)])
    //        .sample(rng)
    //        .spawn(commands, bundle);
    //}

    pub fn spawn(&self, commands: &mut Commands, bundle: impl Bundle) {
        match self {
            Tower::Bumper => {
                commands.spawn((Bumper, bundle)).observe(tower_bonk);
            }
            Tower::Dispenser => {
                commands
                    .spawn((Dispenser, bundle))
                    .observe(dispense)
                    .observe(tower_bonk);
            }
        }
    }
}

/// The number of bonks before the tower despawns.
#[derive(Component)]
#[require(RigidBody::Kinematic, CollisionEventsEnabled)]
enum Bonks {
    Limited(usize),
    Unlimited,
}

fn bonks(trigger: Trigger<OnCollisionStart>, mut bonks: Query<&mut Bonks>) {
    if let Ok(mut bonks) = bonks.get_mut(trigger.target()) {
        match bonks.as_mut() {
            Bonks::Limited(bonks) => {
                *bonks = bonks.saturating_sub(1);
            }
            _ => {}
        }
    }
}

fn despawn_empty_bonks(mut commands: Commands, bonks: Query<(Entity, &Bonks)>) {
    for (entity, _) in bonks
        .iter()
        .filter(|(_, bonks)| matches!(bonks, Bonks::Limited(bonks) if *bonks == 0))
    {
        commands.entity(entity).despawn();
    }
}

/// The factor applied to the impulse generated by a bonk.
#[derive(Component)]
struct BonkImpulse(f32);

fn tower_bonk(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    mut writer: EventWriter<PointEvent>,
    server: Res<AssetServer>,
    towers: Query<(&GlobalTransform, &Tower)>,
) {
    let Ok((transform, tower)) = towers.get(trigger.target()) else {
        return;
    };

    match tower {
        Tower::Bumper => {
            commands.spawn(
                SamplePlayer::new(server.load("audio/pinball/1MetalKLANK.ogg"))
                    .with_volume(Volume::Linear(0.4)),
            );
        }
        _ => {}
    }

    writer.write(PointEvent {
        points: 20,
        position: transform.translation().xy(),
    });
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

#[derive(Component)]
#[require(
    Tower::Bumper,
    BonkImpulse(2.),
    DebugCircle::color(TOWER_RADIUS, RED),
    Collider::circle(TOWER_RADIUS)
)]
pub struct Bumper;

#[derive(Component)]
#[require(
    Tower::Dispenser,
    Bonks::Limited(10),
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
