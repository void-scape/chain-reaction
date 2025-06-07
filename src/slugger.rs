use bevy::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;

pub struct SluggerPlugin;

impl Plugin for SluggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, server: Res<AssetServer>) {
    //commands.spawn((
    //    HIGH_RES_LAYER,
    //    Sprite::from_image(server.load("textures/slugger.png")),
    //    Transform::from_scale(Vec3::splat(4. * crate::RESOLUTION_SCALE)),
    //));
}
