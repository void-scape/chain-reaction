use bevy::ecs::query::QueryFilter;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;

use crate::sandbox;
use crate::selection::{FeaturePack, SelectionEvent};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Reset), reset)
            .add_systems(OnEnter(GameState::StartGame), start)
            .state_variant::<Loading, _>(GameState::Loading)
            .state_variant::<Menu, _>(GameState::Menu)
            .state_variant::<StartGame, _>(GameState::StartGame)
            .state_variant::<Playing, _>(GameState::Playing)
            .state_variant::<Leaderboard, _>(GameState::Leaderboard)
            .state_variant::<Selection, _>(GameState::Selection)
            .state_variant::<Reset, _>(GameState::Reset);

        #[cfg(debug_assertions)]
        app.add_systems(Update, state);
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    StartGame,
    Playing,
    Leaderboard,
    Selection,
    Reset,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Loading;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Menu;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct StartGame;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Playing;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Leaderboard;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Selection;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct Reset;

/// Configure a [`SystemSet`] `V` to run if state is `S` for these schedules:
///
/// * [`PreStartup`]
/// * [`Startup`]
/// * [`PostStartup`]
///
/// * [`First`]
/// * [`PreUpdate`]
/// * [`Update`]
/// * [`PostUpdate`]
/// * [`Last`]
trait StateVariantAppExt {
    fn state_variant<V: SystemSet + Default, S: States + Clone>(&mut self, state: S) -> &mut Self;
}

impl StateVariantAppExt for App {
    fn state_variant<V: SystemSet + Default, S: States + Clone>(&mut self, state: S) -> &mut Self {
        self.configure_sets(PreStartup, V::default().run_if(in_state(state.clone())))
            .configure_sets(Startup, V::default().run_if(in_state(state.clone())))
            .configure_sets(PostStartup, V::default().run_if(in_state(state.clone())))
            .configure_sets(First, V::default().run_if(in_state(state.clone())))
            .configure_sets(PreUpdate, V::default().run_if(in_state(state.clone())))
            .configure_sets(Update, V::default().run_if(in_state(state.clone())))
            .configure_sets(PostUpdate, V::default().run_if(in_state(state.clone())))
            .configure_sets(Last, V::default().run_if(in_state(state)))
    }
}

#[cfg(debug_assertions)]
fn state(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyR) {
        commands.set_state(GameState::Reset);
    }
}

fn reset(mut commands: Commands) {
    commands.set_state(GameState::StartGame);
}

fn start(mut commands: Commands, mut writer: EventWriter<SelectionEvent>) {
    if !sandbox::ENABLED {
        writer.write(SelectionEvent {
            packs: FeaturePack::triple_starter(),
        });
    } else {
        commands.set_state(GameState::Playing);
    }
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
