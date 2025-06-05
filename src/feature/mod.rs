use std::f32::consts::PI;
use std::marker::PhantomData;
use std::time::Duration;

use crate::ball::{Ball, PaddleRestMult, PlayerBall};
use crate::collectables::PointEvent;
use crate::state::{GameState, StateAppExt, remove_entities};
use crate::{Avian, Layer};
use avian2d::prelude::PhysicsSet;
use avian2d::prelude::*;
use bevy::prelude::*;

use self::features::{Dispenser, Splitter, reset_bouncers, spawn_feature_list};
use self::grid::FeatureGrid;

mod features;
pub mod grid;

pub use features::*;

pub struct FeaturePlugin;

impl Plugin for FeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_reset((
            remove_entities::<With<Feature>>,
            remove_entities::<With<FeatureGrid>>,
        ))
        .add_event::<BonksReload>()
        .add_systems(
            OnEnter(GameState::StartGame),
            (spawn_feature_zone, spawn_feature_list),
        )
        .add_systems(
            Update,
            (
                feature_cooldown::<Dispenser>,
                feature_cooldown::<Splitter>,
                grid::FeatureGrid::spawn_slots,
                debug_impulse,
            ),
        )
        .add_systems(Avian, despawn_empty_bonks.before(PhysicsSet::Prepare))
        .add_systems(OnEnter(GameState::Selection), reset_bouncers)
        .add_observer(bonks)
        .add_observer(bonk_bounce)
        .add_observer(feature_bonk)
        .add_observer(bumper)
        .add_observer(kaching)
        .add_observer(dispense)
        .add_observer(splitter)
        .add_observer(lotto)
        .add_observer(bouncer);

        //#[cfg(debug_assertions)]
        //app.add_systems(Update, spawn_feature.in_set(Playing));
    }
}

#[derive(Component)]
#[require(
    RigidBody::Kinematic,
    Collider::rectangle(crate::WIDTH / 1.5, crate::HEIGHT / 1.5),
    CollisionLayers::new(Layer::FeatureZone, Layer::Ball),
    CollisionEventsEnabled,
    Sensor,
    grid::FeatureGrid { spacing: Vec2::new(75.0, 75.0), rotation_rads: PI * 0.25 }
)]
pub struct FeatureZone;

fn spawn_feature_zone(mut commands: Commands) {
    commands
        .spawn((FeatureZone, Transform::from_xyz(0., 50., 0.)))
        .observe(validate_balls)
        .observe(invalidate_balls);
}

/// Marks a [`PlayerBall`] as a valid target for constructing a new feature.
#[derive(Component)]
pub struct ValidZone;

fn validate_balls(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    invalid_balls: Query<Entity, (With<PlayerBall>, Without<ValidZone>)>,
) {
    if let Ok(entity) = invalid_balls.get(trigger.collider) {
        commands.entity(entity).insert(ValidZone);
    }
}

fn invalidate_balls(
    trigger: Trigger<OnCollisionEnd>,
    mut commands: Commands,
    valid_balls: Query<Entity, (With<PlayerBall>, With<ValidZone>)>,
) {
    if let Ok(entity) = valid_balls.get(trigger.collider) {
        commands.entity(entity).remove::<ValidZone>();
    }
}

//fn spawn_feature(
//    mut commands: Commands,
//    digits: Res<ButtonInput<KeyCode>>,
//    input: Res<ButtonInput<MouseButton>>,
//    window: Single<&Window, With<PrimaryWindow>>,
//    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
//    slots: Query<(Entity, &GlobalTransform), (With<FeatureSlot>, Without<SlotFeature>)>,
//
//    mut selection: Local<Feature>,
//) {
//    if digits.just_pressed(KeyCode::Digit1) {
//        *selection = Feature::Bumper;
//    } else if digits.just_pressed(KeyCode::Digit2) {
//        *selection = Feature::Dispenser;
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
//    // check for the nearest feature slot within some threshold
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
//        (SlotFeatureOf(nearest_slot), ChildOf(nearest_slot)),
//    );
//}

/// The base amount of points a feature should give.
#[derive(Clone, Component)]
pub struct Points(pub usize);

/// Temporarily disable the effect of a feature collision for a [`Ball`].
#[derive(Component)]
pub struct FeatureCooldown<T: 'static> {
    timer: Timer,
    _feature: PhantomData<fn() -> T>,
}

impl<T> FeatureCooldown<T> {
    pub fn from_seconds(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            _feature: PhantomData,
        }
    }
}

fn feature_cooldown<T>(
    mut commands: Commands,
    time: Res<Time>,
    mut cooldowns: Query<(Entity, &mut FeatureCooldown<T>)>,
) {
    for (entity, mut cooldown) in cooldowns.iter_mut() {
        cooldown.timer.tick(time.delta());
        if cooldown.timer.finished() {
            commands.entity(entity).remove::<FeatureCooldown<T>>();
        }
    }
}

/// The number of bonks before the feature despawns.
#[derive(Clone, Component)]
#[require(RigidBody::Kinematic, CollisionEventsEnabled)]
enum Bonks {
    Limited(usize),
    #[allow(unused)]
    Reloading {
        max: usize,
        current: usize,
    },
    Unlimited,
}

fn bonks(
    trigger: Trigger<OnCollisionStart>,
    mut bonks: Query<&mut Bonks>,
    mut writer: EventWriter<BonksReload>,
) {
    if let Ok(mut bonks) = bonks.get_mut(trigger.target()) {
        match bonks.as_mut() {
            Bonks::Limited(bonks) => {
                *bonks = bonks.saturating_sub(1);
            }
            Bonks::Reloading { max, current } => {
                *current = current.saturating_sub(1);
                if *current == 0 {
                    *current = *max;
                    writer.write(BonksReload(trigger.target()));
                }
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

/// [`Bonks::Reloading`] reaches 0.
#[derive(Event)]
#[allow(unused)]
struct BonksReload(Entity);

/// The factor applied to the impulse generated by a bonk.
#[derive(Clone, Component)]
struct BonkImpulse(f32);

fn feature_bonk(
    trigger: Trigger<OnCollisionStart>,
    mut writer: EventWriter<PointEvent>,
    features: Query<(&GlobalTransform, &Points), With<Feature>>,
    collider: Query<Option<&PaddleRestMult>>,
) {
    let Ok((transform, Points(points))) = features.get(trigger.target()) else {
        return;
    };

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
    features: Query<(&GlobalTransform, &BonkImpulse), With<Feature>>,
    mut balls: Query<(&GlobalTransform, &mut ExternalImpulse), Or<(With<Ball>, With<PlayerBall>)>>,
    mut commands: Commands,
) {
    match (
        features.get(trigger.target()),
        balls.get_mut(trigger.collider),
    ) {
        (Ok((transform, mult)), Ok((ball_transform, mut bonk))) => {
            let ball_trans = ball_transform.translation().xy();
            let feature_trans = transform.translation().xy();

            let impulse = (ball_trans - feature_trans).normalize_or_zero() * 38_000. * mult.0;
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
