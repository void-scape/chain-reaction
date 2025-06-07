use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::reflect::Typed;
use bevy::window::PrimaryWindow;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};
use convert_case::{Case, Casing};

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_systems(Update, hover);
    }
}

#[derive(Default, Component)]
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
struct Hovered(Entity);

const HOVERZ: f32 = 800.;

fn hover(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,

    targets: Query<(Entity, &Tooltips, &GlobalTransform, &Collider), Without<Hovered>>,
    hovered: Query<(Entity, &Hovered, &GlobalTransform, &Collider)>,
) {
    let (camera, gt) = camera.into_inner();
    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
    else {
        return;
    };

    for (entity, tips, gt, collider) in targets.iter() {
        let position = gt.translation().xy();
        if collider.contains_point(position, gt.rotation(), world_position) {
            let hover = commands
                .spawn((
                    Transform::from_translation(position.extend(HOVERZ)),
                    Visibility::Visible,
                    HIGH_RES_LAYER,
                    children![
                        (
                            Text2d::new(tips.name.to_case(Case::Title)),
                            Transform::from_xyz(0., -40., 0.),
                        ),
                        (Text2d::new(tips.desc), Transform::from_xyz(0., -80., 0.),),
                    ],
                ))
                .id();
            commands.entity(entity).insert(Hovered(hover));
        }
    }

    for (entity, hovered, gt, collider) in hovered.iter() {
        let position = gt.translation().xy();
        if !collider.contains_point(position, gt.rotation(), world_position) {
            commands.entity(entity).remove::<Hovered>();
            commands.entity(hovered.0).despawn();
        }
    }
}
