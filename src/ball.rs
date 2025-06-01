use avian2d::prelude::*;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;

use crate::paddle::PaddleBonk;
use crate::particles::{Emitters, ParticleBundle, ParticleEmitter, transform};
use crate::queue::SpawnTower;
use crate::tower::ValidZone;
use crate::{GameState, Layer};

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), |mut commands: Commands| {
            commands.insert_resource(Lives(2))
        })
        .add_systems(
            Update,
            (spawn_ball, (despawn_ball, tower_ball, recharge).chain())
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[allow(unused)]
#[derive(Resource)]
pub struct Lives(usize);

#[derive(Component)]
#[require(
    RigidBody::Dynamic,
    Restitution::new(0.7),
    DebugCircle::new(12.),
    Collider::circle(12.)
)]
pub struct Ball;

#[derive(Component)]
#[require(
    RigidBody::Dynamic,
    Restitution::new(0.7),
    DebugCircle::color(12., YELLOW),
    Collider::circle(12.),
    CollisionLayers::new(Layer::TowerBall, [Layer::Default, Layer::TowerZone]),
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

fn spawn_ball(
    mut commands: Commands,
    #[cfg(debug_assertions)] input: Res<ButtonInput<KeyCode>>,
    #[cfg(not(debug_assertions))] mut lives: ResMut<Lives>,
    alive: Query<&TowerBall>,
) {
    #[cfg(not(debug_assertions))]
    let cond = alive.is_empty();
    #[cfg(debug_assertions)]
    let cond = alive.is_empty() || input.just_pressed(KeyCode::KeyA);

    if cond {
        let transform = Transform::from_xyz(-crate::WIDTH / 2. + 80., crate::HEIGHT / 2. - 20., 0.);

        #[cfg(not(debug_assertions))]
        if lives.0 > 0 {
            lives.0 -= 1;
            commands.spawn((Ball, transform));
        }

        #[cfg(debug_assertions)]
        commands.spawn((TowerBall, transform));
    }
}

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
    mut writer: EventWriter<SpawnTower>,
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
        writer.write(SpawnTower(tower_ball.1.translation.xy()));
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
