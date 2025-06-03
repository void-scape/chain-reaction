use avian2d::prelude::{Collider, SimpleCollider};
use bevy::prelude::*;
use bevy_optix::debug::DebugCircle;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct TowerGrid {
    pub spacing: Vec2,
    pub rotation_rads: f32,
}

impl TowerGrid {
    pub fn spawn_slots(
        grids: Query<(Entity, &TowerGrid, &Collider), Changed<TowerGrid>>,
        mut commands: Commands,
    ) {
        for (grid_entity, grid, collider) in grids.iter() {
            let theta = grid.rotation_rads;
            let cos_t = theta.cos();
            let sin_t = theta.sin();
            let sqrt2 = std::f32::consts::SQRT_2;

            let bounds = collider.aabb(Vec2::default(), Quat::default());

            let size = bounds.size();
            let container_center = bounds.center();

            if grid.spacing.x <= 0.0 || grid.spacing.y <= 0.0 {
                continue;
            }

            let span_gx = (size.x.abs() + size.y.abs()) / sqrt2;
            let span_gy = span_gx;

            let cols_potential = (span_gx / grid.spacing.x).floor() as usize;
            let rows_potential = (span_gy / grid.spacing.y).floor() as usize;

            if cols_potential == 0 || rows_potential == 0 {
                // Not enough space within the projected span for even a single line of points.
                continue;
            }

            let offset_local_x = (cols_potential.saturating_sub(1) as f32 * grid.spacing.x) * -0.5;
            let offset_local_y = (rows_potential.saturating_sub(1) as f32 * grid.spacing.y) * 0.5;

            for ix in 0..cols_potential {
                for iy in 0..rows_potential {
                    let p_local_x = ix as f32 * grid.spacing.x + offset_local_x;
                    let p_local_y = iy as f32 * -grid.spacing.y + offset_local_y;

                    let p_rotated_x = p_local_x * cos_t - p_local_y * sin_t;
                    let p_rotated_y = p_local_x * sin_t + p_local_y * cos_t;

                    let pos_relative_to_container_origin = Vec2::new(p_rotated_x, p_rotated_y);

                    let final_pos = container_center + pos_relative_to_container_origin;

                    if final_pos.x >= bounds.min.x
                        && final_pos.x <= bounds.max.x
                        && final_pos.y >= bounds.min.y
                        && final_pos.y <= bounds.max.y
                    {
                        commands.spawn((
                            ChildOf(grid_entity),
                            TowerSlot,
                            DebugCircle::new(4.0),
                            Transform::from_translation(final_pos.extend(0.0)),
                        ));
                    }
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
