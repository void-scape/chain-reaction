use avian2d::prelude::*;
use bevy::color::palettes::css::GREY;
use bevy::core_pipeline::bloom::Bloom;
use bevy::prelude::*;
use bevy_light_2d::light::{AmbientLight2d, PointLight2d};
use bevy_optix::camera::MainCamera;
use bevy_optix::debug::DebugRect;
use bevy_optix::post_process::PostProcessCommand;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Repeat};
use bevy_tween::tween::IntoTarget;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};
use std::f32::consts::PI;
use std::time::Duration;

use crate::collectables::HexColor;
use crate::float_tween_wrapper;
use crate::state::GameState;

pub const WIDTH: f32 = 550.;
pub const HEIGHT: f32 = 750.;

pub const CABZ: f32 = -100.;

pub struct CabinetPlugin;

impl Plugin for CabinetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(HexColor(0x2b0f54).into()))
            .add_systems(Startup, spawn_edges)
            .add_systems(OnEnter(GameState::StartGame), lighting)
            .add_systems(Update, make_colliders)
            .add_tween_systems(component_tween_system::<PointLightIntensity>());
    }
}

float_tween_wrapper!(
    PointLight2d,
    point_light_intensity,
    PointLightIntensity,
    intensity
);

fn lighting(mut commands: Commands) {
    commands.post_process::<MainCamera>(Bloom::NATURAL);
    commands.post_process::<MainCamera>(AmbientLight2d {
        brightness: 0.1,
        ..Default::default()
    });
}

#[derive(Component)]
#[require(RigidBody::Static)]
struct CabinetMesh(Handle<Gltf>);

fn make_colliders(
    meshes: Query<(Entity, &CabinetMesh), Without<Collider>>,
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<bevy::gltf::GltfMesh>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    for (mesh_entity, mesh) in &meshes {
        let Some(mesh_data) = gltf_assets.get(&mesh.0) else {
            continue;
        };

        let plane = &mesh_data.named_meshes["Plane"];
        let Some(mesh) = gltf_mesh_assets.get(plane) else {
            continue;
        };
        let Some(mesh) = mesh_assets.get(&mesh.primitives[0].mesh) else {
            return;
        };

        let vertex_buffer = mesh
            .triangles()
            .unwrap()
            .flat_map(|t| t.vertices)
            .map(|v| v.xz() * 250.0)
            .collect::<Vec<_>>();
        let index_buffer = (0..vertex_buffer.len() as u32 / 3)
            .map(|i| {
                let start = i * 3;
                [start, start + 1, start + 2]
            })
            .collect::<Vec<_>>();

        info_once!("mesh: {:#?}", vertex_buffer);
        let collider = Collider::trimesh(vertex_buffer, index_buffer);

        let aabb = collider.aabb(Vec2::default(), Quat::default());
        let size = aabb.size();
        info_once!("aabb size: {:#?}", size);

        commands.entity(mesh_entity).insert(collider);
    }
}

fn spawn_edges(mut commands: Commands, _server: Res<AssetServer>) {
    let x = 160.;
    let y = 130.;

    let w = 180.;
    let h = 15.;

    let rot = PI / 4.5;

    // walls
    commands.spawn((
        RigidBody::Static,
        Restitution::new(0.7),
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(w, h), GREY),
        Collider::rectangle(w, h),
        Rotation::radians(-rot),
        CollisionEventsEnabled,
    ));

    commands.spawn((
        RigidBody::Static,
        Restitution::new(0.7),
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(w, h), GREY),
        Collider::rectangle(w, h),
        Rotation::radians(rot),
        CollisionEventsEnabled,
    ));

    let x = 65. + x;
    let y = 150. + y;

    let hh = 1000.;

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(h, hh), GREY),
        Collider::rectangle(h, hh),
        CollisionEventsEnabled,
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(h, hh), GREY),
        Collider::rectangle(h, hh),
        CollisionEventsEnabled,
    ));

    spawn_light(
        &mut commands,
        //Vec2::new(-x, 0.),
        //PI / 4.,
        HexColor(0xc53a9d),
    );
    spawn_light(
        &mut commands,
        //Vec2::new(x, 0.),
        //-PI / 4. + PI,
        HexColor(0x4a2480),
    );

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(0., crate::HEIGHT / 2., CABZ),
        DebugRect::from_size_color(Vec2::new(hh, 50.), GREY),
        Collider::rectangle(hh, 50.),
        CollisionEventsEnabled,
    ));

    // commands.spawn((
    //     CabinetMesh(server.load("meshes/square.gltf")),
    //     Transform::from_xyz(0., crate::HEIGHT * 0.33, CABZ),
    // ));
}

fn spawn_light(commands: &mut Commands, color: impl Into<Color>) {
    let entity = commands
        .spawn((
            PointLight2d {
                intensity: 10.0,
                radius: 1000.,
                cast_shadows: true,
                color: color.into(),
                ..Default::default()
            },
            //Transform::from_xyz(position.x, position.y, 0.)
            //    .with_rotation(Quat::from_rotation_z(rotation)),
            //children![
            //    (
            //        LightOccluder2d {
            //            shape: LightOccluder2dShape::Rectangle {
            //                half_size: Vec2::splat(10.),
            //            },
            //        },
            //        Transform::from_xyz(-4., -12., 0.),
            //    ),
            //    (
            //        LightOccluder2d {
            //            shape: LightOccluder2dShape::Rectangle {
            //                half_size: Vec2::splat(10.),
            //            },
            //        },
            //        Transform::from_xyz(-4., 12., 0.),
            //    )
            //],
        ))
        .id();

    commands
        .entity(entity)
        .animation()
        .repeat(Repeat::Infinitely)
        .repeat_style(bevy_tween::prelude::RepeatStyle::PingPong)
        .insert_tween_here(
            Duration::from_secs_f32(1.),
            EaseKind::SineInOut,
            entity.into_target().with(
                //    rotation(
                //    Quat::from_rotation_z(-PI / 8. + PI),
                //    Quat::from_rotation_z(PI / 8. + PI),
                //)
                point_light_intensity(10., 8.),
            ),
        );
}
