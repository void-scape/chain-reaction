use std::time::Duration;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_light_2d::light::PointLight2d;
use bevy_optix::pixel_perfect::OuterCamera;
use bevy_tween::combinator::{sequence, tween};
use bevy_tween::interpolate::sprite_color;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Interpolator, Repeat};
use bevy_tween::tween::IntoTarget;

use crate::animation::{AnimationAppExt, AnimationSprite};
use crate::cabinet::{point_light_color, point_light_intensity};
use crate::collectables::HexColor;
use crate::state::{self, GameState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.register_layout(
            "textures/open.png",
            TextureAtlasLayout::from_grid(UVec2::new(31, 24), 18, 4, None, None),
        )
        .register_layout(
            "textures/sign1.png",
            TextureAtlasLayout::from_grid(UVec2::new(60, 34), 17, 3, None, None),
        )
        .add_systems(OnEnter(GameState::Menu), setup_menu)
        .add_systems(Update, await_enter.in_set(state::Menu))
        .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct Menu;

fn setup_menu(mut commands: Commands, server: Res<AssetServer>) {
    let scale = Vec3::new(crate::WIDTH / 200., crate::HEIGHT / 200., 0.);

    commands.spawn((
        Menu,
        Transform::from_scale(scale).with_translation(Vec3::NEG_Z),
        Sprite::from_image(server.load("textures/menu_mask.png")),
    ));

    commands.spawn((
        Menu,
        Transform::from_scale(scale),
        Sprite::from_image(server.load("textures/menu_graffiti.png")),
    ));

    commands.spawn((
        Menu,
        Transform::from_scale(scale).with_translation(Vec3::Z),
        Sprite::from_image(server.load("textures/menu_back.png")),
    ));

    let sign = commands
        .spawn((
            Menu,
            //HIGH_RES_LAYER,
            Transform::from_scale(scale).with_translation(Vec3::new(12., -20., 1.)),
            AnimationSprite::repeating("textures/open.png", 0.08, 0..68),
            PointLight2d {
                intensity: 0.5,
                radius: 200.,
                color: HexColor(0x5ff6be).into(),
                ..Default::default()
            },
        ))
        .id();

    commands
        .animation()
        .repeat(Repeat::Infinitely)
        .insert(sequence((
            tween(
                Duration::from_secs_f32(0.1),
                EaseKind::SineInOut,
                sign.into_target().with(point_light_intensity(0.3, 0.5)),
            ),
            tween(
                Duration::from_secs_f32(0.15),
                EaseKind::SineInOut,
                sign.into_target().with(point_light_intensity(0.5, 0.4)),
            ),
            tween(
                Duration::from_secs_f32(0.08),
                EaseKind::SineInOut,
                sign.into_target().with(point_light_intensity(0.4, 0.6)),
            ),
            tween(
                Duration::from_secs_f32(0.12),
                EaseKind::SineInOut,
                sign.into_target().with(point_light_intensity(0.6, 0.3)),
            ),
        )));

    //commands.spawn((
    //    Menu,
    //    HIGH_RES_LAYER,
    //    Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE * 6.))
    //        .with_translation(Vec3::new(300., 250., 1.)),
    //    AnimationSprite::repeating("textures/sign1.png", 0.05, 0..49),
    //));

    let light = commands
        .spawn((
            Menu,
            Transform::from_xyz(0., 0., -3.),
            Sprite::from_color(Color::WHITE, Vec2::new(crate::WIDTH, crate::HEIGHT)),
        ))
        .id();
    insert_light_tweens(&mut commands, light, sprite_color);

    let point_light = commands
        .spawn((
            Menu,
            Transform::from_xyz(12., -100., 10.),
            PointLight2d {
                intensity: 1.,
                radius: 500.,
                ..Default::default()
            },
        ))
        .id();
    insert_light_tweens(&mut commands, point_light, point_light_color);
}

fn insert_light_tweens<I: Interpolator>(
    commands: &mut Commands,
    entity: Entity,
    the_tween: impl Fn(Color, Color) -> I,
) {
    let red: Color = HexColor(0xb4202a).into();
    let orange: Color = HexColor(0xfa6a0a).into();
    let purple: Color = HexColor(0xbc4a9b).into();

    let a = 0.2;
    let red_low = red.with_alpha(a);
    let orange_low = orange.with_alpha(a);
    let purple_low = purple.with_alpha(a);

    let a = 1.;
    let red = red.with_alpha(a);
    let orange = orange.with_alpha(a);
    let purple = purple.with_alpha(a);

    commands
        .entity(entity)
        .animation()
        .repeat(Repeat::Infinitely)
        .insert(sequence((
            tween(
                Duration::from_secs_f32(0.3),
                EaseKind::SineInOut,
                entity.into_target().with(the_tween(red_low, red)),
            ),
            tween(
                Duration::from_secs_f32(0.3),
                EaseKind::SineInOut,
                entity.into_target().with(the_tween(red, orange_low)),
            ),
            tween(
                Duration::from_secs_f32(0.3),
                EaseKind::SineInOut,
                entity.into_target().with(the_tween(orange_low, orange)),
            ),
            tween(
                Duration::from_secs_f32(0.3),
                EaseKind::SineInOut,
                entity.into_target().with(the_tween(orange, purple_low)),
            ),
            tween(
                Duration::from_secs_f32(0.3),
                EaseKind::SineInOut,
                entity.into_target().with(the_tween(purple_low, purple)),
            ),
            tween(
                Duration::from_secs_f32(0.3),
                EaseKind::SineInOut,
                entity.into_target().with(the_tween(purple, red_low)),
            ),
        )));
}

#[derive(Component)]
struct EnterSprite;

fn await_enter(
    mut commands: Commands,
    server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,

    enter: Option<Single<Entity, With<EnterSprite>>>,
) {
    let (camera, gt) = camera.into_inner();
    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
    else {
        return;
    };

    let target = Vec2::new(12., -100.);
    if world_position.distance_squared(target) > 8_000. {
        if let Some(entity) = enter {
            commands.entity(*entity).despawn();
        }
        return;
    }

    let scale = Vec3::new(crate::WIDTH / 200., crate::HEIGHT / 200., 0.);
    if enter.is_none() {
        commands.spawn((
            Menu,
            EnterSprite,
            Transform::from_scale(scale).with_translation(Vec3::Z * 10.),
            Sprite::from_image(server.load("textures/menu_enter.png")),
        ));
    }

    if input.just_pressed(MouseButton::Left) {
        commands.set_state(GameState::ToGame);
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}
