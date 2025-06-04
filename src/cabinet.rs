use avian2d::prelude::*;
use bevy::color::palettes::css::GREY;
use bevy::prelude::*;
use bevy_optix::debug::DebugRect;
use std::f32::consts::PI;

pub const CABZ: f32 = -100.;

pub struct CabinetPlugin;

impl Plugin for CabinetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_edges)
            .add_systems(Update, make_colliders);
    }
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
    ));

    commands.spawn((
        RigidBody::Static,
        Restitution::new(0.7),
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(w, h), GREY),
        Collider::rectangle(w, h),
        Rotation::radians(rot),
    ));

    let x = 65. + x;
    let y = 150. + y;

    let hh = 1000.;

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(-x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(h, hh), GREY),
        Collider::rectangle(h, hh),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(x, -crate::HEIGHT / 2. + y, CABZ),
        DebugRect::from_size_color(Vec2::new(h, hh), GREY),
        Collider::rectangle(h, hh),
    ));

    commands.spawn((
        RigidBody::Static,
        Transform::from_xyz(0., crate::HEIGHT / 2., CABZ),
        DebugRect::from_size_color(Vec2::new(hh, 50.), GREY),
        Collider::rectangle(hh, 50.),
    ));

    // commands.spawn((
    //     CabinetMesh(server.load("meshes/square.gltf")),
    //     Transform::from_xyz(0., crate::HEIGHT * 0.33, CABZ),
    // ));
}
