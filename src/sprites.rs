#![allow(unused)]

use bevy::prelude::*;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_cell_sprites);
    }
}

#[derive(Clone, Copy, Component)]
#[require(Visibility)]
pub struct CellSprite {
    pub path: &'static str,
    pub size: CellSize,
    pub cell: UVec2,
    pub z: f32,
}

impl CellSprite {
    pub fn new8(path: &'static str, cell: UVec2) -> Self {
        Self {
            path,
            size: CellSize::Eight,
            cell,
            z: 0.,
        }
    }

    pub fn new16(path: &'static str, cell: UVec2) -> Self {
        Self {
            path,
            size: CellSize::Sixteen,
            cell,
            z: 0.,
        }
    }

    pub fn new24(path: &'static str, cell: UVec2) -> Self {
        Self {
            path,
            size: CellSize::TwentyFour,
            cell,
            z: 0.,
        }
    }
}

#[derive(Clone, Copy)]
pub enum CellSize {
    Eight,
    Sixteen,
    TwentyFour,
}

fn spawn_cell_sprites(
    mut commands: Commands,
    server: Res<AssetServer>,
    sprites: Query<(Entity, &CellSprite, Option<&Transform>)>,
) {
    for (entity, sprite, transform) in sprites.iter() {
        commands
            .entity(entity)
            .insert(sprite_rect(&server, sprite.path, sprite.size, sprite.cell))
            .remove::<CellSprite>();
        if transform.is_none() {
            commands
                .entity(entity)
                .insert(Transform::from_xyz(0., 0., sprite.z));
        }
    }
}

pub fn sprite_rect(
    server: &AssetServer,
    path: &'static str,
    size: CellSize,
    cell: UVec2,
) -> Sprite {
    Sprite {
        image: server.load(path),
        rect: Some(rect(size, cell)),
        ..Default::default()
    }
}

fn rect(size: CellSize, cell: UVec2) -> Rect {
    let size = match size {
        CellSize::Eight => 8.,
        CellSize::Sixteen => 16.,
        CellSize::TwentyFour => 24.,
    };
    Rect::from_corners(cell.as_vec2() * size, (cell.as_vec2() + 1.) * size)
}
