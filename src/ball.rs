use avian2d::prelude::*;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;

use crate::paddle::PaddleBonk;
use crate::particles::{Emitters, ParticleBundle, ParticleEmitter, transform};
use crate::state::{GameState, StateAppExt, remove_entities};
use crate::tower::ValidZone;

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_reset(remove_entities::<With<BallComponents>>)
            .add_systems(
                Update,
                (despawn_ball, tower_ball, recharge)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Default, Component)]
#[require(
    RigidBody::Dynamic,
    LinearDamping(0.5),
    AngularDamping(0.3),
    Restitution::new(0.7),
    Collider::circle(8.)
)]
struct BallComponents;

#[derive(Component)]
#[require(BallComponents, DebugCircle::new(8.))]
pub struct Ball;

#[derive(Component)]
#[require(
    BallComponents,
    DebugCircle::color(8., YELLOW),
    //CollisionLayers::new(Layer::TowerBall, [Layer::Default, Layer::TowerZone]),
    ParticleBundle = Self::particles(),
)]
pub struct TowerBall;

impl TowerBall {
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

fn tower_ball(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    tower_ball: Single<(Entity, &Transform), (With<TowerBall>, With<ValidZone>, Without<Depleted>)>,
    //mut writer: EventWriter<SpawnTower>,
) {
    if input.just_pressed(KeyCode::KeyD) {
        commands
            .entity(tower_ball.0)
            .remove::<(
                ParticleBundle,
                DebugCircle,
                Mesh2d,
                MeshMaterial2d<ColorMaterial>,
            )>()
            .despawn_related::<Emitters>()
            .insert((Ball, Depleted));
        //writer.write(SpawnTower(tower_ball.1.translation.xy()));
    }
}

fn recharge(
    mut commands: Commands,
    mut reader: EventReader<PaddleBonk>,
    depleted: Query<Entity, (With<TowerBall>, With<Depleted>)>,
) {
    for event in reader.read() {
        if let Ok(entity) = depleted.get(event.0) {
            commands
                .entity(entity)
                .remove::<(Depleted, DebugCircle, Mesh2d, MeshMaterial2d<ColorMaterial>)>()
                .insert(TowerBall);
        }
    }
}
