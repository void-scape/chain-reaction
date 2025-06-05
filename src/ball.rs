use avian2d::prelude::*;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};

use crate::Layer;
use crate::feature::ValidZone;
use crate::paddle::PaddleBonk;
use crate::particles::{Emitters, ParticleBundle, ParticleEmitter, transform};
use crate::state::{Playing, StateAppExt, remove_entities};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_reset(remove_entities::<With<BallComponents>>)
            .add_systems(
                Update,
                (despawn_ball, player_ball, recharge)
                    .chain()
                    .in_set(Playing),
            )
            .add_tween_systems(component_tween_system::<PaddleRestMultTween>());
    }
}

/// Stores the paddle rest time when this ball was hit.
#[derive(Component)]
pub struct PaddleRestMult(pub f32);
crate::float_tween_wrapper!(PaddleRestMult, paddle_mult, PaddleRestMultTween);

#[derive(Default, Component)]
#[require(
    RigidBody::Dynamic,
    LinearDamping(0.5),
    AngularDamping(0.3),
    Restitution::new(0.7),
    Collider::circle(8.),
    CollisionLayers::new(Layer::Ball, [Layer::Default, Layer::Paddle]),
)]
pub struct BallComponents;

#[derive(Component)]
#[require(BallComponents, DebugCircle::new(8.))]
pub struct Ball;

#[derive(Component)]
#[require(
    BallComponents,
    DebugCircle::color(8., YELLOW),
    ParticleBundle = Self::particles(),
)]
pub struct PlayerBall;

impl PlayerBall {
    fn particles() -> ParticleBundle {
        ParticleBundle::from_emitter(
            ParticleEmitter::from_effect("particles/tower-ball.ron")
                .with(transform(Transform::from_xyz(0., 0., -1.))),
        )
    }
}

#[derive(Default, Component)]
pub struct Depleted;

fn despawn_ball(mut commands: Commands, balls: Query<(Entity, &Transform)>) {
    for (entity, transform) in balls.iter() {
        if transform.translation.y < -crate::HEIGHT / 2. - 12. {
            commands.entity(entity).despawn();
        }
    }
}

fn player_ball(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    feature_ball: Single<
        (Entity, &Transform),
        (With<PlayerBall>, With<ValidZone>, Without<Depleted>),
    >,
) {
    if input.just_pressed(KeyCode::KeyD) {
        commands
            .entity(feature_ball.0)
            .remove::<(
                ParticleBundle,
                DebugCircle,
                Mesh2d,
                MeshMaterial2d<ColorMaterial>,
            )>()
            .despawn_related::<Emitters>()
            .insert((Ball, Depleted));
    }
}

fn recharge(
    mut commands: Commands,
    mut reader: EventReader<PaddleBonk>,
    depleted: Query<Entity, (With<PlayerBall>, With<Depleted>)>,
) {
    for event in reader.read() {
        if let Ok(entity) = depleted.get(event.0) {
            commands
                .entity(entity)
                .remove::<(Depleted, DebugCircle, Mesh2d, MeshMaterial2d<ColorMaterial>)>()
                .insert(PlayerBall);
        }
    }
}
