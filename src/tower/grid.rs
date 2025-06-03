use avian2d::prelude::{Collider, SimpleCollider};
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct TowerGrid {
    pub spacing: Vec2,
}

impl TowerGrid {
    pub fn spawn_slots(
        grids: Query<(Entity, &Self, &Collider), Changed<Self>>,
        mut commands: Commands,
    ) {
        for (grid_entity, grid, collider) in grids {
            let bounds = collider.aabb(Vec2::default(), Quat::default());
            let size = bounds.size();

            if grid.spacing.x <= 0.0 || grid.spacing.y <= 0.0 {
                continue;
            }

            let cols = (size.x / grid.spacing.x) as usize;
            let rows = (size.y / grid.spacing.y) as usize;

            let offset = Vec2::new(
                (cols.saturating_sub(1) as f32 * grid.spacing.x) * -0.5,
                (rows.saturating_sub(1) as f32 * grid.spacing.y) * 0.5,
            );

            for x in 0..cols {
                for y in 0..rows {
                    let position =
                        offset + Vec2::new(x as f32 * grid.spacing.x, y as f32 * -grid.spacing.y);

                    commands.spawn((
                        ChildOf(grid_entity),
                        TowerSlot,
                        DebugCircle::new(4.0),
                        Transform::from_translation(position.extend(0.0)),
                    ));
                }
            }
        }
    }
}

#[derive(Component)]
pub struct TowerSlot;

#[derive(Component)]
#[relationship(relationship_target = SlotTower)]
pub struct SlotTowerOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = SlotTowerOf, linked_spawn)]
pub struct SlotTower(Entity);

impl SlotTower {
    #[expect(dead_code)]
    pub fn tower(&self) -> Entity {
        self.0
    }
}
