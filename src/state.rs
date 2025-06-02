use bevy::ecs::query::QueryFilter;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Reset), reset);

        #[cfg(debug_assertions)]
        app.add_systems(Update, state);
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
    Reset,
}

fn state(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyR) {
        commands.set_state(GameState::Reset);
    }
}

fn reset(mut commands: Commands) {
    commands.set_state(GameState::Playing);
}

pub fn remove_entities<T: QueryFilter>(mut commands: Commands, entities: Query<Entity, T>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn insert_resource<R: Resource + Clone>(res: R) -> impl Fn(Commands) {
    move |mut commands| {
        commands.insert_resource(res.clone());
    }
}

pub trait StateAppExt {
    fn add_reset<M>(&mut self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> &mut Self;
}

impl StateAppExt for App {
    fn add_reset<M>(&mut self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> &mut Self {
        self.add_systems(OnEnter(GameState::Reset), systems)
    }
}
