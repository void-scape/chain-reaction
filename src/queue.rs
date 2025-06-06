use bevy::prelude::*;

use crate::state::GameState;
use crate::feature::{TOWER_SIZE, Feature};

pub struct QueuePlugin;

impl Plugin for QueuePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFeature>()
            .add_systems(OnEnter(GameState::Playing), spawn_queue)
            .add_systems(Update, spawn_feature.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
#[relationship_target(relationship = Queue)]
struct QueuedFeatures(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = QueuedFeatures)]
struct Queue(Entity);

#[derive(Component)]
struct FeatureQueue;

fn spawn_queue(mut commands: Commands) {
    let shown = 4;

    let padding = 5.;
    let offset = TOWER_SIZE + padding;

    let id = commands.spawn(FeatureQueue).id();

    let mut rng = rand::thread_rng();
    for i in 0..shown {
        Feature::spawn_random(
            &mut commands,
            &mut rng,
            (
                Queue(id),
                Transform::from_xyz(crate::cabinet::WIDTH / 2. - 20., i as f32 * -offset, 0.),
            ),
        );
    }
}

#[derive(Event)]
pub struct SpawnFeature(pub Vec2);

fn spawn_feature(
    mut commands: Commands,
    mut reader: EventReader<SpawnFeature>,
    queue: Single<(Entity, &QueuedFeatures), With<FeatureQueue>>,
    mut features: Query<(Entity, &mut Transform), With<Queue>>,
) {
    let (queue, queued) = queue.into_inner();
    for event in reader.read() {
        let padding = 5.;
        let offset = TOWER_SIZE + padding;

        for (_, mut transform) in features.iter_mut() {
            transform.translation.y += offset;
        }

        let remove = queued
            .0
            .iter()
            .map(|entity| {
                features
                    .get(*entity)
                    .map(|(_, t)| (*entity, t.translation.y))
                    .unwrap()
            })
            .max_by(|(_, y1), (_, y2)| y1.total_cmp(y2))
            .unwrap()
            .0;

        commands
            .entity(remove)
            .remove::<Queue>()
            .insert(Transform::from_translation(event.0.extend(0.)));

        let mut rng = rand::thread_rng();
        Feature::spawn_random(
            &mut commands,
            &mut rng,
            (
                Queue(queue),
                Transform::from_xyz(
                    crate::cabinet::WIDTH / 2. - 20.,
                    (features.iter().count() as f32 - 1.) * -offset,
                    0.,
                ),
            ),
        );
    }
}
