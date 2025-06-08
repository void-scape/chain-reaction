use crate::RESOLUTION_SCALE;
use crate::big::BigPoints;
use crate::state::{StateAppExt, insert_resource};
use crate::text::flash_text_rotate;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use dashu::integer::IBig;
use rand::Rng;
use std::f32::consts::PI;
use std::usize;

pub const POINT_COLOR: HexColor = HexColor(0xfff540);
pub const MONEY_COLOR: HexColor = HexColor(0x00ff00);
pub const MONEY_COLOR_REMOVE: HexColor = HexColor(0xff1100);
pub const SIZE: f32 = 25.;
pub const POINT_TEXT_Z: f32 = 500.;

pub struct CollectablePlugin;

impl Plugin for CollectablePlugin {
    fn build(&self, app: &mut App) {
        app.add_reset((
            insert_resource(TotalPoints(Default::default())),
            insert_resource(Points::default()),
            insert_resource(Money(0)),
        ))
        .add_event::<PointEvent>()
        .add_event::<MoneyEvent>()
        .insert_resource(Points::default())
        .insert_resource(TotalPoints(Default::default()))
        .insert_resource(Money(0))
        .add_systems(PostUpdate, effects);

        #[cfg(debug_assertions)]
        {
            use bevy_optix::debug::debug_res;
            app.add_systems(
                Update,
                (
                    debug_res::<Money>(
                        Transform::from_xyz(
                            -crate::RES_WIDTH / 2.,
                            -crate::RES_HEIGHT / 2. + 25.,
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
pub struct TotalPoints(BigPoints);

impl TotalPoints {
    pub fn get(&self) -> &BigPoints {
        &self.0
    }
}

#[derive(Debug, Default, Clone, Resource)]
pub struct Points(BigPoints);

impl Points {
    pub fn get(&self) -> &BigPoints {
        &self.0
    }

    pub fn reset(&mut self) {
        self.0 = BigPoints(IBig::ZERO);
    }
}

#[derive(Event)]
pub struct PointEvent {
    pub points: BigPoints,
    pub position: Vec2,
}

#[derive(Debug, Clone, Resource)]
pub struct Money(i32);

impl Money {
    pub fn get(&self) -> i32 {
        self.0
    }
}

#[derive(Event)]
pub struct MoneyEvent {
    pub money: i32,
    pub position: Vec2,
}

fn effects(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut points: EventReader<PointEvent>,
    mut money: EventReader<MoneyEvent>,
    mut total_points: ResMut<Points>,
    mut total_total_points: ResMut<TotalPoints>,
    mut total_money: ResMut<Money>,
) {
    if !points.is_empty() || !money.is_empty() {
        commands.spawn(
            SamplePlayer::new(server.load("audio/score.ogg")).with_volume(Volume::Linear(0.5)),
        );
    }

    let mut rng = rand::thread_rng();
    let rot = PI / 9.;

    for event in points.read() {
        total_points.0.0 += event.points.0.clone();
        flash_text_rotate(
            &mut commands,
            &server,
            format!("+{}", event.points),
            SIZE,
            (event.position * RESOLUTION_SCALE).extend(POINT_TEXT_Z),
            rng.gen_range(-rot..rot),
            POINT_COLOR,
        );
    }

    for event in points.read() {
        total_points.0.0 += event.points.0.clone();
        total_total_points.0.0 += event.points.0.clone();
        flash_text_rotate(
            &mut commands,
            &server,
            format!("+{}", event.points),
            SIZE,
            (event.position * RESOLUTION_SCALE).extend(POINT_TEXT_Z),
            rng.gen_range(-rot..rot),
            POINT_COLOR,
        );
    }

    for event in money.read() {
        total_money.0 += event.money;

        let money_color = if event.money >= 0 {
            MONEY_COLOR
        } else {
            MONEY_COLOR_REMOVE
        };

        flash_text_rotate(
            &mut commands,
            &server,
            format!("${}", event.money),
            SIZE,
            (event.position * RESOLUTION_SCALE).extend(POINT_TEXT_Z),
            rng.gen_range(-rot..rot),
            money_color,
        );
    }
}
