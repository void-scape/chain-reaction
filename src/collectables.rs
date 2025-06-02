use crate::RESOLUTION_SCALE;
use crate::state::{StateAppExt, insert_resource};
use crate::text::flash_text;
use bevy::prelude::*;
use bevy_optix::debug::debug_res;
use bevy_seedling::prelude::*;
use std::usize;

pub const POINT_COLOR: HexColor = HexColor(0xfff540);
pub const MONEY_COLOR: HexColor = HexColor(0x00ff00);
pub const SIZE: f32 = 40.;
pub const POINT_TEXT_Z: f32 = 500.;

pub struct CollectablePlugin;

impl Plugin for CollectablePlugin {
    fn build(&self, app: &mut App) {
        app.add_reset((insert_resource(Points(0)), insert_resource(Money(0))))
            .add_event::<PointEvent>()
            .add_event::<MoneyEvent>()
            .insert_resource(Points(0))
            .insert_resource(Money(0))
            .add_systems(PostUpdate, effects)
            .add_systems(
                Update,
                (
                    debug_res::<Money>(
                        Transform::from_xyz(
                            -crate::RES_WIDTH / 2.,
                            -crate::RES_HEIGHT / 2. + 40.,
                            500.,
                        ),
                        bevy::sprite::Anchor::BottomLeft,
                    ),
                    debug_res::<Points>(
                        Transform::from_xyz(
                            -crate::RES_WIDTH / 2.,
                            -crate::RES_HEIGHT / 2. + 80.,
                            500.,
                        ),
                        bevy::sprite::Anchor::BottomLeft,
                    ),
                ),
            );
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

#[derive(Debug, Clone, Resource)]
pub struct Points(usize);

impl Points {
    pub fn get(&self) -> usize {
        self.0
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

#[derive(Event)]
pub struct PointEvent {
    pub points: usize,
    pub position: Vec2,
}

#[derive(Debug, Clone, Resource)]
pub struct Money(usize);

#[derive(Event)]
pub struct MoneyEvent {
    pub money: usize,
    pub position: Vec2,
}

fn effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut points: EventReader<PointEvent>,
    mut money: EventReader<MoneyEvent>,
    mut total_points: ResMut<Points>,
    mut total_money: ResMut<Money>,
) {
    if !points.is_empty() || !money.is_empty() {
        commands.spawn(
            SamplePlayer::new(server.load("audio/score.ogg")).with_volume(Volume::Linear(0.5)),
        );
    }

    for event in points.read() {
        total_points.0 += event.points;
        flash_text(
            &mut commands,
            &server,
            format!("+{}", event.points),
            SIZE,
            (event.position * RESOLUTION_SCALE).extend(POINT_TEXT_Z),
            POINT_COLOR,
        );
    }

    for event in money.read() {
        total_money.0 += event.money;
        flash_text(
            &mut commands,
            &server,
            format!("${}", event.money),
            SIZE,
            (event.position * RESOLUTION_SCALE).extend(POINT_TEXT_Z),
            MONEY_COLOR,
        );
    }
}
