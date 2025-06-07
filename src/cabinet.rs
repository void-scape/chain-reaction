use avian2d::prelude::*;
use bevy::core_pipeline::bloom::Bloom;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy_light_2d::light::{AmbientLight2d, PointLight2d};
use bevy_optix::camera::MainCamera;
use bevy_optix::post_process::PostProcessCommand;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Repeat};
use bevy_tween::tween::IntoTarget;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};
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
        app.add_plugins((
            Material2dPlugin::<ScrollingTexture>::default(),
            Material2dPlugin::<Diamonds>::default(),
        ))
        .insert_resource(ClearColor(HexColor(0x2b0f54).into()))
        .add_systems(Startup, (spawn_edges, background))
        .add_systems(OnEnter(GameState::StartGame), lighting)
        .add_systems(Update, (make_colliders, update_scrolling_background))
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
struct CabinetMesh {
    scene: Handle<Gltf>,
    mesh: &'static str,
}

const CABINET_SCALE: f32 = 235.0;

fn make_colliders(
    meshes: Query<(Entity, &CabinetMesh), Without<Collider>>,
    mut commands: Commands,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_mesh_assets: Res<Assets<bevy::gltf::GltfMesh>>,
    mesh_assets: Res<Assets<Mesh>>,
) {
    for (mesh_entity, CabinetMesh { scene, mesh }) in &meshes {
        let Some(mesh_data) = gltf_assets.get(scene) else {
            continue;
        };

        let plane = &mesh_data.named_meshes[*mesh];
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
            .map(|v| {
                let mut twod = v.xz() * CABINET_SCALE;
                twod.y *= -1.0;
                twod
            })
            .collect::<Vec<_>>();
        let index_buffer = (0..vertex_buffer.len() as u32 / 3)
            .map(|i| {
                let start = i * 3;
                [start, start + 1, start + 2]
            })
            .collect::<Vec<_>>();

        let collider = Collider::trimesh(vertex_buffer, index_buffer);

        let aabb = collider.aabb(Vec2::default(), Quat::default());
        let size = aabb.size();
        info_once!("aabb size: {:#?}", size);

        commands.entity(mesh_entity).insert(collider);
    }
}

fn spawn_edges(mut commands: Commands, server: Res<AssetServer>) {
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

    let cabinet_transform = Transform::from_xyz(0., crate::HEIGHT * 0.15, CABZ);

    commands.spawn((
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "Cabinet",
        },
        cabinet_transform,
    ));

    commands.spawn((
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "LeftSling",
        },
        cabinet_transform,
    ));

    commands.spawn((
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "RightSling",
        },
        cabinet_transform,
    ));

    commands.spawn((
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "LeftChannel",
        },
        cabinet_transform,
    ));

    commands.spawn((
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "RightChannel",
        },
        cabinet_transform,
    ));
}

fn spawn_light(commands: &mut Commands, color: impl Into<Color>) {
    let entity = commands
        .spawn((
            PointLight2d {
                intensity: 2.0,
                radius: 1024.,
                cast_shadows: true,
                color: color.into(),
                falloff: 0.,
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
                point_light_intensity(2., 1.9),
            ),
        );
}

#[derive(Component)]
struct Speed(Vec2);

fn background(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ScrollingTexture>>,
    mut diamonds: ResMut<Assets<Diamonds>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1024., 1024.))),
        MeshMaterial2d(diamonds.add(Diamonds {})),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1024., 1024.))),
        Transform::from_xyz(0., 0., -900.),
        Speed(Vec2::new(0.05, 0.1) * 0.5),
        MeshMaterial2d(mats.add(ScrollingTexture {
            uv_offset: Vec2::ZERO,
            texture: server.load_with_settings("textures/checkers.png", |s: &mut _| {
                *s = ImageLoaderSettings {
                    sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::MirrorRepeat,
                        address_mode_v: ImageAddressMode::MirrorRepeat,
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        mipmap_filter: ImageFilterMode::Nearest,
                        ..default()
                    }),
                    ..default()
                }
            }),
        })),
    ));
}

fn update_scrolling_background(
    query: Query<(&MeshMaterial2d<ScrollingTexture>, &Speed)>,
    mut materials: ResMut<Assets<ScrollingTexture>>,
    time: Res<Time>,
) {
    for (handle, speed) in query.iter() {
        let material = materials.get_mut(&handle.0).unwrap();
        material.uv_offset -= speed.0 * time.delta_secs();
        if material.uv_offset.x >= 1. {
            material.uv_offset.x = 0.;
        }
        if material.uv_offset.y >= 1. {
            material.uv_offset.y = 0.;
        }
    }
}

#[derive(Clone, Asset, TypePath, AsBindGroup)]
struct ScrollingTexture {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    uv_offset: Vec2,
}

impl Material2d for ScrollingTexture {
    fn fragment_shader() -> ShaderRef {
        "shaders/scrolling_texture.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Clone, Asset, TypePath, AsBindGroup)]
struct Diamonds {}

impl Material2d for Diamonds {
    fn fragment_shader() -> ShaderRef {
        "shaders/diamonds.wgsl".into()
    }
}
