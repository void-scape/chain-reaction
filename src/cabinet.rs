use avian2d::prelude::*;

use bevy::core_pipeline::bloom::Bloom;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{AlphaMode2d, Anchor, Material2d, Material2dPlugin};
use bevy_light_2d::light::{AmbientLight2d, PointLight2d};
use bevy_optix::camera::MainCamera;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};
use bevy_optix::post_process::PostProcessCommand;
use bevy_tween::combinator::{sequence, tween};
use bevy_tween::interpolate::translation;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Repeat};
use bevy_tween::tween::IntoTarget;
use bevy_tween::{BevyTweenRegisterSystems, component_tween_system};
use std::time::Duration;

use crate::collectables::{HexColor, Money, Points};
use crate::state::{GameState, StateAppExt, remove_entities};
use crate::{float_tween_wrapper, sandbox};

pub const WIDTH: f32 = 550.;
pub const HEIGHT: f32 = 750.;

pub const CABZ: f32 = -100.;

pub struct CabinetPlugin;

impl Plugin for CabinetPlugin {
    fn build(&self, app: &mut App) {
        app.add_reset(remove_entities::<With<Cabinet>>)
            .add_plugins((
                Material2dPlugin::<ScrollingTexture>::default(),
                Material2dPlugin::<Diamonds>::default(),
            ))
            .insert_resource(ClearColor(HexColor(0x090808).into()))
            .add_systems(OnEnter(GameState::StartGame), (spawn, background))
            .add_systems(Startup, lighting)
            .add_systems(
                Update,
                (
                    make_colliders,
                    update_scrolling_background,
                    points_ui,
                    money_ui,
                ),
            )
            .add_tween_systems(component_tween_system::<PointLightIntensity>());

        if !sandbox::ENABLED {
            app.add_systems(Startup, move_camera);
        }
    }
}

const CAM_OFFSET: f32 = crate::WIDTH / 2. - 150.;

fn move_camera(mut camera: Single<&mut Transform, With<OuterCamera>>) {
    camera.translation.x += CAM_OFFSET;
}

float_tween_wrapper!(
    PointLight2d,
    point_light_intensity,
    PointLightIntensity,
    intensity
);

fn lighting(mut commands: Commands) {
    commands.post_process::<OuterCamera>(Bloom::NATURAL);
    commands.post_process::<MainCamera>(AmbientLight2d {
        brightness: 0.8,
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

#[derive(Default, Component)]
struct Cabinet;

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

pub const UIZ: f32 = 100.;

#[derive(Component)]
struct PointsUI;

fn points_ui(mut text: Single<&mut Text2d, With<PointsUI>>, points: Res<Points>) {
    if points.is_changed() {
        text.0 = format!("{}", points.get());
    }
}

#[derive(Component)]
struct MoneyUI;

fn money_ui(
    mut commands: Commands,
    text: Single<(Entity, &mut Text2d), With<MoneyUI>>,
    money: Res<Money>,
) {
    let (entity, mut text) = text.into_inner();
    if money.is_changed() {
        text.0 = format!("${}", money.get());
        if money.get().is_negative() {
            commands
                .entity(entity)
                .insert(TextColor(HexColor(0xb4202a).into()));
        } else {
            commands
                .entity(entity)
                .insert(TextColor(HexColor(0x59c135).into()));
        }
    }
}

fn spawn(mut commands: Commands, server: Res<AssetServer>) {
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

    let t = Transform::from_xyz(-349. / 2. + 25., 115. / 2. - 25., 1.);
    commands.spawn((
        Cabinet,
        HIGH_RES_LAYER,
        Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE)).with_translation(Vec3::new(
            WIDTH / 1.1,
            HEIGHT / 2.5,
            UIZ,
        )),
        Sprite::from_image(server.load("textures/panel.png")),
        children![(
            PointsUI,
            Text2d::default(),
            TextFont {
                font: server.load("fonts/cube.ttf"),
                font_size: 25.,
                ..Default::default()
            },
            Anchor::TopLeft,
            t,
        )],
    ));
    commands.spawn((
        Cabinet,
        HIGH_RES_LAYER,
        Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE)).with_translation(Vec3::new(
            WIDTH / 1.1,
            HEIGHT / 2.5 - 125.,
            UIZ,
        )),
        Sprite::from_image(server.load("textures/panel.png")),
        children![(
            MoneyUI,
            Text2d::default(),
            TextFont {
                font: server.load("fonts/cube.ttf"),
                font_size: 25.,
                ..Default::default()
            },
            Anchor::TopLeft,
            t,
        )],
    ));

    //commands.spawn((
    //    HIGH_RES_LAYER,
    //    Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
    //    Sprite {
    //        image: server.load("textures/overlay5.png"),
    //        color: Color::WHITE.with_alpha(0.05),
    //        ..Default::default()
    //    },
    //));
    //commands.spawn((
    //    HIGH_RES_LAYER,
    //    Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
    //    Sprite {
    //        image: server.load("textures/overlay30.png"),
    //        color: Color::WHITE.with_alpha(0.05),
    //        ..Default::default()
    //    },
    //));

    let cabinet_transform = Transform::from_xyz(0., crate::HEIGHT * 0.15, CABZ);

    commands.spawn((
        Cabinet,
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "Cabinet",
        },
        cabinet_transform,
    ));

    commands.spawn((
        Cabinet,
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "LeftSling",
        },
        cabinet_transform,
    ));

    commands.spawn((
        Cabinet,
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "RightSling",
        },
        cabinet_transform,
    ));

    commands.spawn((
        Cabinet,
        CollisionEventsEnabled,
        RigidBody::Static,
        CabinetMesh {
            scene: server.load("meshes/cabinet.gltf"),
            mesh: "LeftChannel",
        },
        cabinet_transform,
    ));

    commands.spawn((
        Cabinet,
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
            Cabinet,
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
        Cabinet,
        Mesh2d(meshes.add(Rectangle::new(1024., 1024.))),
        MeshMaterial2d(diamonds.add(Diamonds {})),
    ));

    commands.spawn((
        Cabinet,
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

pub fn transition(mut commands: Commands, server: Res<AssetServer>) {
    let mut stagger = false;
    for y in 0..1024 / 62 {
        let right = Vec3::new(
            crate::WIDTH / 2. + 1024. + CAM_OFFSET,
            y as f32 * 62. - crate::HEIGHT / 2.,
            980.,
        );
        let left = Vec3::new(
            -crate::WIDTH / 2. - 1024. + CAM_OFFSET,
            y as f32 * 62. - crate::HEIGHT / 2.,
            980.,
        );
        let middle = Vec3::new(CAM_OFFSET, y as f32 * 62. - crate::HEIGHT / 2., 980.);

        let (start, end) = if stagger {
            (right, left)
        } else {
            (left, right)
        };

        let slider = commands
            .spawn((
                Cabinet,
                HIGH_RES_LAYER,
                Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
                Sprite::from_image(server.load("textures/slider.png")),
            ))
            .id();

        let dur = 1.5;
        let animation = commands
            .animation()
            .insert(sequence((
                tween(
                    Duration::from_secs_f32(dur / 2.),
                    EaseKind::ExponentialOut,
                    slider.into_target().with(translation(start, middle)),
                ),
                tween(
                    Duration::from_secs_f32(dur / 2.),
                    EaseKind::ExponentialIn,
                    slider.into_target().with(translation(middle, end)),
                ),
            )))
            .id();
        commands.entity(slider).add_child(animation);

        stagger = !stagger;
    }
}
