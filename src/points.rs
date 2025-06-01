use crate::RESOLUTION_SCALE;
use crate::text::flash_text;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use std::usize;

pub const COLOR: HexColor = HexColor(0xfff540);
pub const POINT_TEXT_Z: f32 = 500.;

pub struct PointPlugin;

impl Plugin for PointPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PointEvent>()
            .insert_resource(Points(0))
            .add_systems(PostUpdate, point_effects);
    }
}

pub struct HexColor(pub u32);

impl Into<Color> for HexColor {
    fn into(self) -> Color {
        Color::srgb_u8(
            (self.0 >> 16) as u8 & 0xFF,
            (self.0 >> 8) as u8 & 0xFF,
            self.0 as u8,
        )
    }
}

#[derive(Resource)]
pub struct Points(usize);

#[derive(Event)]
pub struct PointEvent {
    pub points: usize,
    pub position: Vec2,
}

fn point_effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut reader: EventReader<PointEvent>,
    mut points: ResMut<Points>,
) {
    if !reader.is_empty() {
        commands.spawn(
            SamplePlayer::new(server.load("audio/score.ogg")).with_volume(Volume::Linear(0.5)),
        );
    }

    for event in reader.read() {
        points.0 += event.points;
        flash_text(
            &mut commands,
            &server,
            format!("+{}", event.points),
            20.,
            (event.position * RESOLUTION_SCALE).extend(POINT_TEXT_Z),
            COLOR,
        );
    }
}
