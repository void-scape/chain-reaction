use std::f32::consts::PI;
use std::sync::Arc;

use avian2d::prelude::*;
use bevy::color::palettes::css::{BLUE, GREEN, MAROON, PURPLE, RED, YELLOW};
use bevy::color::palettes::tailwind::CYAN_700;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::reflect::Typed;
use bevy_optix::debug::DebugCircle;
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::SamplePlayer;

use crate::ball::{Ball, BallComponents, PlayerBall};
use crate::collectables::{MoneyEvent, PointEvent};
use crate::paddle::PaddleBonk;
use crate::sampler::Sampler;
use crate::state::{GameState, Playing};

use super::{BonkImpulse, Bonks, FeatureCooldown, Points, feature_cooldown};

pub const MAX_BALLS: usize = 2000;
pub const FEATURE_SIZE: f32 = 36.0;
pub const FEATURE_RADIUS: f32 = FEATURE_SIZE / 2.;

pub struct FeaturesPlugin;

impl Plugin for FeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::StartGame), spawn_feature_list)
            .add_systems(
                Update,
                (
                    feature_cooldown::<Dispenser>,
                    feature_cooldown::<Splitter>,
                    clear_bing_bong,
                )
                    .in_set(Playing),
            )
            .add_observer(bumper)
            .add_observer(kaching)
            .add_observer(dispense)
            .add_observer(bing_bong)
            .add_observer(splitter)
            .add_observer(lotto)
            .add_observer(field_inverter)
            .add_observer(field_inversion);
    }
}

#[derive(Default, Clone, Component)]
#[require(Bonks::Unlimited, BonkImpulse(1.), Points(0), CollisionEventsEnabled)]
pub struct Feature;

#[derive(Component)]
pub struct Prob(pub f32);

#[derive(Component, Clone)]
pub struct FeatureSpawner(pub Arc<dyn Fn(&mut EntityCommands) + Send + Sync>);

impl FeatureSpawner {
    pub fn new<T: Component + Default>() -> Self {
        Self(Arc::new(|commands: &mut EntityCommands| {
            commands.insert(T::default());
        }))
    }
}

#[derive(Default, Component)]
#[require(Transform, Visibility::Visible)]
#[component(on_insert = Self::on_insert_hook)]
pub struct Tooltips {
    pub name: &'static str,
    pub desc: &'static str,
}

impl Tooltips {
    fn on_insert_hook(mut world: DeferredWorld, ctx: HookContext) {
        let entity = world.get::<Tooltips>(ctx.entity).unwrap();
        let name = entity.name;

        world.commands().entity(ctx.entity).insert(Name::new(name));
    }
}

impl Tooltips {
    pub fn new<T: Typed>() -> Self {
        Self::named::<T>(T::type_ident().unwrap())
    }

    pub fn named<T: Typed>(name: &'static str) -> Self {
        Self {
            name,
            desc: T::type_info()
                .docs()
                .expect("`Feature` has no documentation"),
        }
    }
}

pub fn spawn_feature_list(mut commands: Commands) {
    commands.spawn((Bumper, Prob(1.), feature_bundle()));
    commands.spawn((MoneyBumper, Prob(1.), feature_bundle()));
    commands.spawn((BingBong, Prob(1.), feature_bundle()));
    commands.spawn((Dispenser, Prob(1.), feature_bundle()));
    commands.spawn((Splitter, Prob(1.), feature_bundle()));
    commands.spawn((Lotto, Prob(1.), feature_bundle()));
    commands.spawn((FieldInverter, Prob(1.), feature_bundle()));
}

fn feature_bundle() -> impl Bundle {
    (
        ColliderDisabled,
        Visibility::Hidden,
        Transform::from_xyz(crate::WIDTH * 2., 0., 0.),
    )
}

/// Gives balls impulses when bonked.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
    FeatureSpawner::new::<Self>(),
    Tooltips::new::<Self>(),
    Points(20),
    BonkImpulse(2.),
    DebugCircle::color(FEATURE_RADIUS, RED),
    Collider::circle(FEATURE_RADIUS)
)]
pub struct Bumper;

pub fn bumper(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    server: Res<AssetServer>,
    bumpers: Query<&Bumper>,
) {
    if bumpers.get(trigger.target()).is_err() {
        return;
    }

    commands.spawn(
        SamplePlayer::new(server.load("audio/pinball/1MetalKLANK.ogg"))
            .with_volume(Volume::Linear(0.4)),
    );
}

/// Indicates how many times a ball has hit a BingBong.
#[derive(Default, Clone, Component, Reflect)]
pub struct BingBongLevel(u32);

fn clear_bing_bong(mut commands: Commands, mut paddle_hit: EventReader<PaddleBonk>) {
    for bonk in paddle_hit.read() {
        commands.entity(bonk.0).remove::<BingBongLevel>();
    }
}

/// Double the points received until hit with the paddle.
#[derive(Default, Clone, Component, Reflect)]
#[require(
    Feature,
    FeatureSpawner::new::<Self>(),
    Tooltips::new::<Self>(),
    Points(0),
    BonkImpulse(2.),
    DebugCircle::color(FEATURE_RADIUS, CYAN_700),
    Collider::circle(FEATURE_RADIUS)
)]
pub struct BingBong;

pub fn bing_bong(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    // server: Res<AssetServer>,
    bing_bongs: Query<&GlobalTransform, With<BingBong>>,
    mut balls: Query<&mut BingBongLevel>,
    mut event_writer: EventWriter<PointEvent>,
) {
    let Ok(transform) = bing_bongs.get(trigger.target()) else {
        return;
    };

    match balls.get_mut(trigger.collider) {
        Ok(mut level) => {
            event_writer.write(PointEvent {
                points: 2usize.pow(level.0) * 10,
                position: transform.translation().xy(),
            });
            level.0 += 1;
        }
        Err(_) => {
            event_writer.write(PointEvent {
                points: 10,
                position: transform.translation().xy(),
            });
            commands.entity(trigger.collider).insert(BingBongLevel(1));
        }
    }
}

/// Produce $1 when bonked.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
    FeatureSpawner::new::<Self>(),
    Tooltips::new::<Self>(),
    Points(0),
    Bonks::Limited(3),
    BonkImpulse(1.25),
    DebugCircle::color(FEATURE_RADIUS * 0.666, YELLOW),
    Collider::circle(FEATURE_RADIUS * 0.666)
)]
pub struct MoneyBumper;

pub fn kaching(
    trigger: Trigger<OnCollisionStart>,
    transforms: Query<&GlobalTransform, With<MoneyBumper>>,
    mut event_writer: EventWriter<MoneyEvent>,
) {
    let Ok(transform) = transforms.get(trigger.target()) else {
        return;
    };

    event_writer.write(MoneyEvent {
        money: 1,
        position: transform.translation().xy(),
    });
}

/// Produce 1 new ball.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
    FeatureSpawner::new::<Self>(),
    Tooltips::new::<Self>(),
    Points(10),
    Bonks::Limited(10),
    DebugCircle::color(FEATURE_RADIUS, GREEN),
    Collider::circle(FEATURE_RADIUS)
)]
pub struct Dispenser;

pub fn dispense(
    trigger: Trigger<OnCollisionStart>,
    balls: Query<(&GlobalTransform, &LinearVelocity), Or<(With<Ball>, With<PlayerBall>)>>,
    filtered: Query<&FeatureCooldown<Dispenser>>,
    transforms: Query<&GlobalTransform, With<Dispenser>>,
    mut commands: Commands,
) {
    if filtered.contains(trigger.collider) {
        return;
    }

    if let (Ok(feature), Ok((ball, velocity))) = (
        transforms.get(trigger.target()),
        balls.get(trigger.collider),
    ) {
        let feature = feature.compute_transform();
        let ball = ball.translation().xy();

        let initial_velocity =
            (feature.translation.xy() - ball).normalize_or_zero() * velocity.0.length();

        if balls.iter().len() < MAX_BALLS {
            commands.spawn((
                Ball,
                FeatureCooldown::<Dispenser>::from_seconds(0.2),
                feature,
                LinearVelocity(initial_velocity * 0.75),
            ));
        }
    }
}

/// Loose $1 when bonked. Every bonk has a 1 in 5 chance to produce $7.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
    FeatureSpawner::new::<Self>(),
    Tooltips::new::<Self>(),
    BonkImpulse(1.25),
    DebugCircle::color(FEATURE_RADIUS, PURPLE),
    Collider::circle(FEATURE_RADIUS)
)]
pub struct Lotto;

pub fn lotto(
    trigger: Trigger<OnCollisionStart>,
    transforms: Query<&GlobalTransform, With<Lotto>>,
    mut event_writer: EventWriter<MoneyEvent>,
) {
    let Ok(transform) = transforms.get(trigger.target()) else {
        return;
    };

    let mut rng = rand::thread_rng();
    let probability = Sampler::new(&[(-1, 4.0), (7, 1.0)]);

    event_writer.write(MoneyEvent {
        money: probability.sample(&mut rng),
        position: transform.translation().xy(),
    });
}

/// Consumes ball, produces two new balls.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
    FeatureSpawner::new::<Self>(),
    Tooltips::new::<Self>(),
    Points(10),
    DebugCircle::color(FEATURE_RADIUS - 2., BLUE),
    Collider::circle(FEATURE_RADIUS - 2.)
)]
pub struct Splitter;

pub fn splitter(
    trigger: Trigger<OnCollisionStart>,
    balls: Query<(Entity, &GlobalTransform, &LinearVelocity), With<BallComponents>>,
    filtered: Query<&FeatureCooldown<Splitter>>,
    transforms: Query<&GlobalTransform, With<Splitter>>,
    mut commands: Commands,
) {
    if filtered.contains(trigger.collider) {
        return;
    }

    if let (Ok(feature), Ok((entity, ball, velocity))) = (
        transforms.get(trigger.target()),
        balls.get(trigger.collider),
    ) {
        let feature = feature.compute_transform();
        let ball = ball.translation().xy();

        commands.entity(entity).despawn();

        let initial_velocity =
            (feature.translation.xy() - ball).normalize_or_zero() * velocity.0.length();

        let rot = PI / 8.;

        if balls.iter().len() < MAX_BALLS {
            commands.spawn((
                Ball,
                FeatureCooldown::<Splitter>::from_seconds(0.2),
                feature,
                LinearVelocity(
                    Vec2::from_angle(rot)
                        .normalize()
                        .rotate(initial_velocity * 0.75),
                ),
            ));

            commands.spawn((
                Ball,
                FeatureCooldown::<Splitter>::from_seconds(0.2),
                feature,
                LinearVelocity(
                    Vec2::from_angle(-rot)
                        .normalize()
                        .rotate(initial_velocity * 0.75),
                ),
            ));
        }
    }
}

/// When bonked, reverse the gravity of the ball for 1 bonk.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
    FeatureSpawner::new::<Self>(),
    Tooltips::new::<Self>(),
    DebugCircle::color(FEATURE_RADIUS - 2., MAROON),
    Collider::circle(FEATURE_RADIUS - 2.)
)]
pub struct FieldInverter;

pub fn field_inverter(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    balls: Query<Entity, With<BallComponents>>,
    inverters: Query<&FieldInverter>,
) {
    if inverters.get(trigger.target()).is_err() {
        return;
    };

    if let Ok(entity) = balls.get(trigger.collider) {
        commands.entity(entity).insert(FieldInversion(1));
    }
}

#[derive(Component)]
#[require(GravityScale(-1.))]
struct FieldInversion(usize);

fn field_inversion(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    mut balls: Query<(Entity, &mut FieldInversion)>,
) {
    if let Ok((entity, mut bonks)) = balls.get_mut(trigger.collider) {
        bonks.0 = bonks.0.saturating_sub(1);
        if bonks.0 == 0 {
            commands
                .entity(entity)
                .remove::<(FieldInversion, GravityScale)>();
        }
    }
}
