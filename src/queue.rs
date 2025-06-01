use bevy::prelude::*;

use crate::GameState;
use crate::tower::{TOWER_SIZE, Tower};

pub struct QueuePlugin;

impl Plugin for QueuePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTower>()
            .add_systems(OnEnter(GameState::Playing), spawn_queue)
            .add_systems(Update, spawn_tower.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
#[relationship_target(relationship = Queue)]
struct QueuedTowers(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = QueuedTowers)]
struct Queue(Entity);

#[derive(Component)]
struct TowerQueue;

fn spawn_queue(mut commands: Commands) {
    let shown = 4;

    let padding = 5.;
    let offset = TOWER_SIZE + padding;

    let id = commands.spawn(TowerQueue).id();

    let mut rng = rand::thread_rng();
    for i in 0..shown {
        Tower::spawn_random(
            &mut commands,
            &mut rng,
            (
                Queue(id),
                Transform::from_xyz(crate::WIDTH / 2. - 20., i as f32 * -offset, 0.),
            ),
        );
    }
}

#[derive(Event)]
pub struct SpawnTower(pub Vec2);

fn spawn_tower(
    mut commands: Commands,
    mut reader: EventReader<SpawnTower>,
    queue: Single<(Entity, &QueuedTowers), With<TowerQueue>>,
    mut towers: Query<(Entity, &mut Transform), With<Queue>>,
) {
    let (queue, queued) = queue.into_inner();
    for event in reader.read() {
        let padding = 5.;
        let offset = TOWER_SIZE + padding;

        for (_, mut transform) in towers.iter_mut() {
            transform.translation.y += offset;
        }

        let remove = queued
            .0
            .iter()
            .map(|entity| {
                towers
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
        Tower::spawn_random(
            &mut commands,
            &mut rng,
            (
                Queue(queue),
                Transform::from_xyz(
                    crate::WIDTH / 2. - 20.,
                    (towers.iter().count() as f32 - 1.) * -offset,
                    0.,
                ),
            ),
        );
    }
}
