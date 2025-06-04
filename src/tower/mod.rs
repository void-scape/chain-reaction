use std::marker::PhantomData;
use std::time::Duration;

use avian2d::math::PI;
use avian2d::prelude::*;
use bevy::color::palettes::css::{GREEN, PURPLE, RED, YELLOW};
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::SamplePlayer;
use strum_macros::EnumIter;

use crate::ball::{Ball, PaddleRestMult, TowerBall};
use crate::collectables::{MoneyEvent, PointEvent};
use crate::sampler::Sampler;
use crate::state::{GameState, StateAppExt, remove_entities};
use crate::{Avian, Layer};

use self::grid::TowerGrid;

pub mod grid;

pub const TOWER_SIZE: f32 = 36.0;
pub const TOWER_RADIUS: f32 = TOWER_SIZE / 2.;

pub struct TowerPlugin;

impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_reset((
            remove_entities::<With<Tower>>,
            remove_entities::<With<TowerGrid>>,
        ))
        .add_systems(OnEnter(GameState::StartGame), spawn_tower_zone)
        .add_systems(
            Update,
            (
                tower_cooldown::<Dispenser>,
                grid::TowerGrid::spawn_slots,
                debug_impulse,
            ),
        )
        .add_systems(Avian, despawn_empty_bonks.before(PhysicsSet::Prepare))
        .add_observer(bonks)
        .add_observer(bonk_bounce);

        //#[cfg(debug_assertions)]
        //app.add_systems(Update, spawn_tower.in_set(Playing));
    }
}

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    Collider::rectangle(crate::WIDTH / 1.5, crate::HEIGHT / 1.5),
    CollisionLayers::new(Layer::TowerZone, Layer::Ball),
    CollisionEventsEnabled,
    Sensor,
    grid::TowerGrid { spacing: Vec2::new(75.0, 75.0), rotation_rads: PI * 0.25 }
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

//fn spawn_tower(
//    mut commands: Commands,
//    digits: Res<ButtonInput<KeyCode>>,
//    input: Res<ButtonInput<MouseButton>>,
//    window: Single<&Window, With<PrimaryWindow>>,
//    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
//    slots: Query<(Entity, &GlobalTransform), (With<TowerSlot>, Without<SlotTower>)>,
//
//    mut selection: Local<Tower>,
//) {
//    if digits.just_pressed(KeyCode::Digit1) {
//        *selection = Tower::Bumper;
//    } else if digits.just_pressed(KeyCode::Digit2) {
//        *selection = Tower::Dispenser;
//    }
//
//    let (camera, gt) = camera.into_inner();
//    if !input.just_pressed(MouseButton::Left) {
//        return;
//    }
//
//    let Some(world_position) = window
//        .cursor_position()
//        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
//        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
//    else {
//        return;
//    };
//
//    // check for the nearest tower slot within some threshold
//
//    let Some((nearest_slot, transform)) = slots.iter().min_by(|a, b| {
//        let a = world_position.distance_squared(a.1.compute_transform().translation.xy());
//        let b = world_position.distance_squared(b.1.compute_transform().translation.xy());
//
//        a.total_cmp(&b)
//    }) else {
//        return;
//    };
//
//    if transform
//        .compute_transform()
//        .translation
//        .xy()
//        .distance(world_position)
//        > 50.0
//    {
//        return;
//    }
//
//    selection.spawn(
//        &mut commands,
//        (SlotTowerOf(nearest_slot), ChildOf(nearest_slot)),
//    );
//}

/// The base amount of points a tower should give.
#[derive(Component)]
pub struct Points(pub usize);

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
    MoneyBumper,
    Dispenser,
    Lotto,
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
            Tower::MoneyBumper => {
                commands
                    .spawn((MoneyBumper, bundle))
                    .observe(kaching)
                    .observe(tower_bonk);
            }
            Tower::Dispenser => {
                commands
                    .spawn((Dispenser, bundle))
                    .observe(dispense)
                    .observe(tower_bonk);
            }
            Tower::Lotto => {
                commands
                    .spawn((Lotto, bundle))
                    .observe(lotto)
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
    towers: Query<(&GlobalTransform, &Points, &Tower)>,
    collider: Query<Option<&PaddleRestMult>>,
) {
    let Ok((transform, Points(points), tower)) = towers.get(trigger.target()) else {
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

    if *points == 0 {
        return;
    }

    let mut points = *points as f32;
    if let Ok(Some(paddle_mult)) = collider.get(trigger.collider) {
        points *= 1. + paddle_mult.0;
    }

    writer.write(PointEvent {
        position: transform.translation().xy(),
        points: (points as usize).max(1),
    });
}

#[derive(Component)]
struct ImpulseGizmo {
    impulse: Vec2,
    timer: Timer,
}

fn debug_impulse(
    mut impulses: Query<(Entity, &GlobalTransform, &mut ImpulseGizmo)>,
    mut gizmos: Gizmos,
    mut commands: Commands,
    time: Res<Time>,
) {
    let delta = time.delta();
    for (entity, position, mut impulse) in impulses.iter_mut() {
        let translation = position.translation().xy();
        let scale_factor = 5e-4;

        gizmos.arrow_2d(
            translation,
            translation + impulse.impulse * scale_factor,
            Color::WHITE,
        );

        if impulse.timer.tick(delta).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn bonk_bounce(
    trigger: Trigger<OnCollisionStart>,
    towers: Query<(&GlobalTransform, &BonkImpulse), With<Tower>>,
    mut balls: Query<(&GlobalTransform, &mut ExternalImpulse), Or<(With<Ball>, With<TowerBall>)>>,
    mut commands: Commands,
) {
    match (
        towers.get(trigger.target()),
        balls.get_mut(trigger.collider),
    ) {
        (Ok((transform, mult)), Ok((ball_transform, mut bonk))) => {
            let ball_trans = ball_transform.translation().xy();
            let tower_trans = transform.translation().xy();

            let impulse = (ball_trans - tower_trans).normalize_or_zero() * 38_000. * mult.0;
            bonk.apply_impulse(impulse);

            commands.spawn((
                ImpulseGizmo {
                    impulse,
                    timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                },
                ball_transform.compute_transform(),
            ));
        }
        _ => {}
    }
}

#[derive(Component)]
#[require(
    Tower::Bumper,
    Points(20),
    BonkImpulse(2.),
    DebugCircle::color(TOWER_RADIUS, RED),
    Collider::circle(TOWER_RADIUS)
)]
pub struct Bumper;

#[derive(Component)]
#[require(
    Tower::MoneyBumper,
    Points(0),
    Bonks::Limited(3),
    BonkImpulse(1.25),
    DebugCircle::color(TOWER_RADIUS * 0.666, YELLOW),
    Collider::circle(TOWER_RADIUS * 0.666)
)]
pub struct MoneyBumper;

fn kaching(
    trigger: Trigger<OnCollisionStart>,
    transforms: Query<&GlobalTransform, With<MoneyBumper>>,
    mut event_writer: EventWriter<MoneyEvent>,
) -> Result {
    let transform = transforms.get(trigger.target())?;

    event_writer.write(MoneyEvent {
        money: 1,
        position: transform.translation().xy(),
    });

    Ok(())
}

#[derive(Component)]
#[require(
    Tower::Dispenser,
    Points(10),
    Bonks::Limited(10),
    DebugCircle::color(TOWER_RADIUS, GREEN),
    Collider::circle(TOWER_RADIUS)
)]
pub struct Dispenser;

fn dispense(
    trigger: Trigger<OnCollisionStart>,
    balls: Query<(&GlobalTransform, &LinearVelocity), Or<(With<Ball>, With<TowerBall>)>>,
    filtered: Query<&TowerCooldown<Dispenser>>,
    transforms: Query<&GlobalTransform, With<Dispenser>>,
    mut commands: Commands,
) {
    if filtered.contains(trigger.collider) {
        return;
    }

    if let (Ok(tower), Ok((ball, velocity))) = (
        transforms.get(trigger.target()),
        balls.get(trigger.collider),
    ) {
        let tower = tower.compute_transform();
        let ball = ball.translation().xy();

        let initial_velocity =
            (tower.translation.xy() - ball).normalize_or_zero() * velocity.0.length();

        commands.spawn((
            Ball,
            TowerCooldown::<Dispenser>::from_seconds(0.5),
            tower,
            LinearVelocity(initial_velocity * 0.75),
        ));
    }
}

#[derive(Component)]
#[require(
    Tower::Lotto,
    Points(0),
    Bonks::Unlimited,
    BonkImpulse(1.25),
    DebugCircle::color(TOWER_RADIUS, PURPLE),
    Collider::circle(TOWER_RADIUS)
)]
pub struct Lotto;

fn lotto(
    trigger: Trigger<OnCollisionStart>,
    transforms: Query<&GlobalTransform, With<Lotto>>,
    mut event_writer: EventWriter<MoneyEvent>,
) -> Result {
    let transform = transforms.get(trigger.target())?;

    let mut rng = rand::thread_rng();
    let probability = Sampler::new(&[(-1, 4.0), (7, 1.0)]);

    event_writer.write(MoneyEvent {
        money: probability.sample(&mut rng),
        position: transform.translation().xy(),
    });

    Ok(())
}
