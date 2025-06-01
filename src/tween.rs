#![allow(unused)]

use avian2d::prelude::{LinearVelocity, Physics, PhysicsTime};
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy_tween::bevy_time_runner::TimeRunner;
use bevy_tween::prelude::*;
use bevy_tween::tween::{apply_component_tween_system, apply_resource_tween_system};

use crate::float_tween;

pub struct TweenPlugin;

impl Plugin for TweenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsTimeMult::default())
            .insert_resource(VirtualTimeMult::default())
            .insert_resource(TimeMult::default())
            .add_tween_systems((
                apply_resource_tween_system::<PhysicsTimeTween>,
                apply_resource_tween_system::<VirtualTimeTween>,
                apply_resource_tween_system::<TimeTween>,
                apply_component_tween_system::<InterpolateLinearVelocity>,
            ))
            .add_systems(
                Update,
                (update_physics_time, update_virtual_time, update_time),
            )
            .add_systems(
                PostUpdate,
                ((despawn_finished_tweens, run_tween_on_end).chain(),),
            );
    }
}

float_tween!(Resource, TimeMult, 1., time_mult, TimeTween);

fn update_time(
    mut virtual_time: ResMut<Time<Virtual>>,
    mut physics_time: ResMut<Time<Physics>>,
    mult: Res<TimeMult>,
) {
    if mult.is_changed() {
        virtual_time.set_relative_speed(mult.0);
        physics_time.set_relative_speed(mult.0);
    }
}

float_tween!(
    Resource,
    PhysicsTimeMult,
    1.,
    physics_time_mult,
    PhysicsTimeTween
);

fn update_physics_time(mut time: ResMut<Time<Physics>>, mult: Res<PhysicsTimeMult>) {
    if mult.is_changed() {
        time.set_relative_speed(mult.0);
    }
}

float_tween!(
    Resource,
    VirtualTimeMult,
    1.,
    virtual_time_mult,
    VirtualTimeTween
);

fn update_virtual_time(mut time: ResMut<Time<Virtual>>, mult: Res<VirtualTimeMult>) {
    if mult.is_changed() {
        time.set_relative_speed(mult.0);
    }
}

#[derive(Component, Default)]
pub struct DespawnTweenFinish;

fn despawn_finished_tweens(
    mut commands: Commands,
    tweens: Query<(Entity, &TimeRunner), With<DespawnTweenFinish>>,
) {
    for (entity, runner) in tweens.iter() {
        if runner.is_completed() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct OnEnd(SystemId);

impl OnEnd {
    pub fn new<Marker>(
        commands: &mut Commands,
        system: impl IntoSystem<(), (), Marker> + 'static,
    ) -> Self {
        Self(commands.register_system(system))
    }
}

fn run_tween_on_end(mut commands: Commands, tweens: Query<(Entity, &TimeRunner, &OnEnd)>) {
    for (tween, runner, on_end) in tweens.iter() {
        if runner.is_completed() {
            commands.run_system(on_end.0);
            commands.unregister_system(on_end.0);
            commands.entity(tween).despawn();
        }
    }
}

#[macro_export]
macro_rules! float_tween {
    ($kind:ident, $name:ident, $default:expr, $func:ident, $tween:ident) => {
        #[derive($kind)]
        pub struct $name(pub f32);

        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }

        pub fn $func(start: f32, end: f32) -> $tween {
            $tween::new(start, end)
        }

        #[derive(Component)]
        pub struct $tween {
            start: f32,
            end: f32,
        }

        impl $tween {
            pub fn new(start: f32, end: f32) -> Self {
                Self { start, end }
            }
        }

        impl Interpolator for $tween {
            type Item = $name;

            fn interpolate(&self, item: &mut Self::Item, value: f32) {
                item.0 = self.start.lerp(self.end, value);
            }
        }
    };
}

pub fn linear_velocity(start: Vec2, end: Vec2) -> InterpolateLinearVelocity {
    InterpolateLinearVelocity { start, end }
}

pub struct InterpolateLinearVelocity {
    start: Vec2,
    end: Vec2,
}

impl Interpolator for InterpolateLinearVelocity {
    type Item = LinearVelocity;

    fn interpolate(&self, vel: &mut Self::Item, value: f32) {
        vel.0 = self.start.lerp(self.end, value);
    }
}
