#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]
#![allow(clippy::single_match)]

use std::io::Cursor;

use avian2d::prelude::{Gravity, PhysicsLayer};
use bevy::DefaultPlugins;
use bevy::app::{App, FixedMainScheduleOrder};
use bevy::asset::AssetMetaCheck;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy::winit::WinitWindows;
use bevy_optix::pixel_perfect::CanvasDimensions;
use winit::window::Icon;

mod animation;
mod ball;
mod big;
mod cabinet;
mod collectables;
mod cursor;
mod feature;
mod input;
mod leaderboard;
mod loading;
mod menu;
mod music;
mod paddle;
mod particles;
mod sampler;
mod sandbox;
mod selection;
mod slugger;
mod sprites;
mod stage;
mod state;
mod text;
mod tooltips;
mod tween;

pub const WIDTH: f32 = 750.;
pub const HEIGHT: f32 = 750.;
pub const RESOLUTION_SCALE: f32 = 1.;

pub const RES_WIDTH: f32 = WIDTH * RESOLUTION_SCALE;
pub const RES_HEIGHT: f32 = HEIGHT * RESOLUTION_SCALE;

pub const GRAVITY: f32 = 400.;

fn main() {
    let mut app = App::new();

    #[cfg(debug_assertions)]
    app.add_systems(Update, close_on_escape);

    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // TODO: Rename
                    title: "Bevy game".to_string(),
                    fit_canvas_to_parent: true,
                    resolution: WindowResolution::new(
                        WIDTH * RESOLUTION_SCALE * 1.5,
                        HEIGHT * RESOLUTION_SCALE,
                    ),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
        bevy_egui::EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        // bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        bevy_tween::DefaultTweenPlugins,
        bevy_enhanced_input::EnhancedInputPlugin,
        avian2d::PhysicsPlugins::new(Avian).with_length_unit(10.),
        bevy_optix::pixel_perfect::PixelPerfectPlugin(CanvasDimensions {
            width: (WIDTH * 1.5) as u32,
            height: HEIGHT as u32,
            pixel_scale: RESOLUTION_SCALE,
        }),
        bevy_optix::debug::DebugPlugin,
        bevy_enoki::EnokiPlugin,
        bevy_light_2d::prelude::Light2dPlugin,
    ))
    .add_plugins((
        loading::LoadingPlugin,
        menu::MenuPlugin,
        paddle::PaddlePlugin,
        ball::BallPlugin,
        feature::FeaturePlugin,
        particles::ParticlePlugin,
        input::InputPlugin,
        cabinet::CabinetPlugin,
        text::TextPlugin,
        collectables::CollectablePlugin,
        tween::TweenPlugin,
        state::StatePlugin,
        stage::StagePlugin,
        leaderboard::LeaderBoardPlugin,
        selection::SelectionPlugin,
    ))
    .add_plugins((
        sandbox::SandboxPlugin,
        tooltips::TooltipPlugin,
        slugger::SluggerPlugin,
        cursor::CursorPlugin,
        music::MusicPlugin,
        animation::AnimationPlugin,
        sprites::SpritePlugin,
    ))
    .add_plugins((avian2d::debug_render::PhysicsDebugPlugin::new(Avian),))
    .init_schedule(Avian)
    .insert_resource(Gravity(Vec2::NEG_Y * GRAVITY))
    .add_systems(Startup, set_window_icon);

    #[cfg(target_arch = "wasm32")]
    use bevy_seedling::prelude::*;
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        bevy_seedling::SeedlingPlugin::<firewheel_web_audio::WebAudioBackend> {
            config: Default::default(),
            stream_config: Default::default(),
            spawn_default_pool: true,
            pool_size: 4..=32,
        },
    );

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(bevy_seedling::SeedlingPlugin {
        ..Default::default()
    });

    app.world_mut()
        .resource_mut::<FixedMainScheduleOrder>()
        .insert_after(FixedPostUpdate, Avian);

    app.run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Avian;

#[derive(Default, Clone, Copy, PartialEq, Eq, PhysicsLayer)]
pub enum Layer {
    #[default]
    Default,
    Ball,
    Paddle,
    FeatureZone,
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) -> Result {
    let primary_entity = primary_window.single()?;
    let Some(primary) = windows.get_window(primary_entity) else {
        return Err(BevyError::from("No primary window!"));
    };
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };

    Ok(())
}

#[cfg(debug_assertions)]
fn close_on_escape(input: Res<ButtonInput<KeyCode>>, mut writer: EventWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        writer.write(AppExit::Success);
    }
}
