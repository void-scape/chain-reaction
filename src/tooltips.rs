use std::time::Duration;

use crate::feature::Price;
use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::reflect::Typed;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use bevy::window::PrimaryWindow;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};
use bevy_tween::interpolate::translation;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind};
use bevy_tween::tween::IntoTarget;
use convert_case::{Case, Casing};

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_systems(Update, (hover, show_tooltips_after));
    }
}

#[derive(Default, Clone, Copy, Component)]
#[require(Transform, Visibility::Visible)]
#[component(on_insert = Self::on_insert_hook)]
pub struct Tooltips {
    pub name: &'static str,
    pub desc: &'static str,
}

impl Tooltips {
    fn on_insert_hook(mut world: DeferredWorld, ctx: HookContext) {
        let entity = world.get::<Tooltips>(ctx.entity).unwrap();
        let name = entity.name;

        world.commands().entity(ctx.entity).insert(Name::new(name));
    }
}

impl Tooltips {
    pub fn new<T: Typed>() -> Self {
        Self::named::<T>(T::type_ident().unwrap())
    }

    pub fn named<T: Typed>(name: &'static str) -> Self {
        Self {
            name,
            desc: T::type_info()
                .docs()
                .expect("`Feature` has no documentation"),
        }
    }
}

#[derive(Component)]
#[component(on_add = Self::insert_tooltips)]
pub struct ShowTooltips {
    pub delay: f32,
}

impl ShowTooltips {
    fn insert_tooltips(mut world: DeferredWorld, ctx: HookContext) {
        let delay = world.get::<ShowTooltips>(ctx.entity).unwrap().delay;
        world
            .commands()
            .entity(ctx.entity)
            .insert(ShowTooltipsAfter(Timer::from_seconds(
                delay,
                TimerMode::Once,
            )));
    }
}

#[derive(Component)]
struct ShowTooltipsAfter(Timer);

fn show_tooltips_after(
    mut commands: Commands,
    server: Res<AssetServer>,
    time: Res<Time>,
    mut show: Query<(Entity, &mut ShowTooltipsAfter, &Tooltips, Option<&Price>)>,
) {
    for (entity, mut show, tips, price) in show.iter_mut() {
        show.0.tick(time.delta());
        if show.0.finished() {
            let child =
                spawn_tooltips(&mut commands, &server, tips, price.map(|p| p.0), Vec2::ZERO);
            commands
                .entity(entity)
                .remove::<ShowTooltipsAfter>()
                .add_child(child);

            commands.animation().insert_tween_here(
                Duration::from_secs_f32(0.2),
                EaseKind::ExponentialOut,
                child.into_target().with(translation(
                    Vec3::new(0., -crate::HEIGHT, 0.),
                    Vec2::new(0., -15.).extend(0.),
                )),
            );
        }
    }
}

#[derive(Component)]
#[component(on_remove = Self::despawn)]
struct Hovered(Entity);

impl Hovered {
    fn despawn(mut world: DeferredWorld, ctx: HookContext) {
        let entity = world.get::<Hovered>(ctx.entity).unwrap().0;
        if let Ok(mut entity) = world.commands().get_entity(entity) {
            entity.try_despawn();
        }
    }
}

fn hover(
    mut commands: Commands,
    server: Res<AssetServer>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
    targets: Query<
        (
            Entity,
            &Tooltips,
            &GlobalTransform,
            &Collider,
            Option<&Price>,
        ),
        (Without<Hovered>, Without<ShowTooltips>),
    >,
    hovered: Query<(Entity, &GlobalTransform, &Collider), (With<Hovered>, Without<ShowTooltips>)>,
) {
    let (camera, gt) = camera.into_inner();
    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
    else {
        return;
    };

    for (entity, tips, gt, collider, price) in targets.iter() {
        let position = gt.translation().xy();
        if collider.contains_point(position, gt.rotation(), world_position) {
            let hover = spawn_tooltips(&mut commands, &server, tips, price.map(|p| p.0), position);
            commands.entity(entity).insert(Hovered(hover));
        }
    }

    for (entity, gt, collider) in hovered.iter() {
        let position = gt.translation().xy();
        if !collider.contains_point(position, gt.rotation(), world_position) {
            commands.entity(entity).remove::<Hovered>();
        }
    }
}

fn spawn_tooltips(
    commands: &mut Commands,
    server: &AssetServer,
    tips: &Tooltips,
    price: Option<i32>,
    position: Vec2,
) -> Entity {
    let sprite = commands
        .spawn((
            HIGH_RES_LAYER,
            Sprite {
                image: server.load("textures/feature_card.png"),
                anchor: Anchor::TopCenter,
                ..Default::default()
            },
            Transform::from_scale(Vec3::splat(2.)),
            Pickable::default(),
        ))
        .observe(hover_sprite)
        .observe(remove_hover_sprite)
        .id();

    let price = price.map(|p| format!("${p}")).unwrap_or_default();

    commands
        .spawn((
            Transform::from_translation((position - Vec2::new(0., 15.)).extend(0.)),
            Visibility::Visible,
            HIGH_RES_LAYER,
            children![
                (
                    Text2d::new(tips.name.to_case(Case::Title)),
                    TextFont {
                        font_size: 25.,
                        font: server.load("fonts/saiba.ttf"),
                        ..Default::default()
                    },
                    TextBounds::new_horizontal(220.),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform::from_xyz(0., -100., 0.),
                ),
                (
                    Text2d::new(price),
                    TextBounds::new_horizontal(220.),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform::from_xyz(0., -150., 0.),
                ),
                (
                    Text2d::new(tips.desc),
                    TextBounds::new_horizontal(220.),
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform::from_xyz(0., -250., 0.),
                )
            ],
        ))
        .add_child(sprite)
        .id()
}

#[derive(Component)]
pub struct Hover;

#[derive(Component)]
struct HoverChild;

fn hover_sprite(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    server: Res<AssetServer>,
    cards: Query<&Sprite, Without<Hover>>,
) {
    if !cards.get(trigger.target()).is_ok() {
        return;
    }

    commands
        .entity(trigger.event().target)
        .insert(Hover)
        .with_child((
            HoverChild,
            HIGH_RES_LAYER,
            Sprite {
                image: server.load("textures/feature_card_hover.png"),
                anchor: Anchor::TopCenter,
                ..Default::default()
            },
            Transform::from_translation(Vec3::Z),
        ));
}

fn remove_hover_sprite(
    trigger: Trigger<Pointer<Out>>,
    mut commands: Commands,
    cards: Query<&Children, With<Hover>>,
    hovers: Query<Entity, With<HoverChild>>,
) {
    let Ok(children) = cards.get(trigger.target()) else {
        return;
    };

    commands.entity(trigger.target()).remove::<Hover>();
    for entity in hovers.iter_many(children.iter()) {
        commands.entity(entity).despawn();
    }
}
