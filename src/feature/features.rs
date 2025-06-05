use std::f32::consts::PI;

use avian2d::prelude::*;
use bevy::color::palettes::css::{BLUE, GREEN, MAROON, PURPLE, RED, YELLOW};
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;
use bevy::reflect::Typed;
use bevy_optix::debug::DebugCircle;
use bevy_seedling::prelude::Volume;
use bevy_seedling::sample::SamplePlayer;

use crate::ball::{Ball, BallComponents, PlayerBall};
use crate::collectables::MoneyEvent;
use crate::sampler::Sampler;

use super::{BonkImpulse, Bonks, FeatureCooldown, Points};

pub const FEATURE_SIZE: f32 = 36.0;
pub const FEATURE_RADIUS: f32 = FEATURE_SIZE / 2.;

#[derive(Default, Clone, Component)]
#[require(Bonks::Unlimited, BonkImpulse(1.), Points(0))]
pub struct Feature;

#[derive(Component)]
pub struct Prob(pub f32);

#[derive(Default, Component)]
#[require(Feature, Transform, Visibility::Visible)]
pub struct Tooltips {
    pub name: &'static str,
    pub desc: &'static str,
}

impl Tooltips {
    pub fn new<T: Typed>() -> Self {
        Self {
            name: T::type_ident().unwrap(),
            desc: T::type_info()
                .docs()
                .expect("`Feature` has no documentation"),
        }
    }
}

pub fn spawn_feature_list(mut commands: Commands) {
    spawn_feature::<Bumper>(&mut commands.spawn(Prob(1.)));
    spawn_feature::<MoneyBumper>(&mut commands.spawn(Prob(1.)));
    spawn_feature::<Dispenser>(&mut commands.spawn(Prob(1.)));
    spawn_feature::<Splitter>(&mut commands.spawn(Prob(1.)));
    spawn_feature::<Lotto>(&mut commands.spawn(Prob(1.)));
    spawn_feature::<Bouncer>(&mut commands.spawn(Prob(1.)));
}

fn spawn_feature<T: Default + Component + Typed>(feature: &mut EntityCommands) {
    feature.insert((T::default(), Tooltips::new::<T>(), Disabled));
}

/// Gives balls impulses when bonked.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
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

/// Produce $1 when bonked.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
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

        commands.spawn((
            Ball,
            FeatureCooldown::<Dispenser>::from_seconds(0.2),
            feature,
            LinearVelocity(initial_velocity * 0.75),
        ));
    }
}

/// Loose $1 when bonked. Every bonk has a 1 in 5 chance to produce $7.
#[derive(Default, Component, Reflect)]
#[require(
    Feature,
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

/// Every second, gain $2 for every ball on the screen.
#[derive(Component, Reflect)]
#[require(
    Feature,
    DebugCircle::color(FEATURE_RADIUS - 2., MAROON),
    Collider::circle(FEATURE_RADIUS - 2.)
)]
pub struct Bouncer(f32);

impl Default for Bouncer {
    fn default() -> Self {
        Self(2.)
    }
}

pub fn bouncer(
    trigger: Trigger<OnCollisionStart>,
    balls: Query<&BallComponents>,
    mut bouncers: Query<&mut Bouncer>,
) {
    if let (Ok(mut bouncer), Ok(_)) = (
        bouncers.get_mut(trigger.target()),
        balls.get(trigger.collider),
    ) {
        bouncer.0 /= 2.;
    }
}

pub fn reset_bouncers(mut bouncers: Query<&mut Bouncer>) {
    for mut bouncer in bouncers.iter_mut() {
        *bouncer = Bouncer::default();
    }
}
