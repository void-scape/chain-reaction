use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::state::GameState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<ActivePlay>()
            .add_input_context::<Menu>()
            .add_systems(Update, action_ctx)
            .add_observer(bind_active)
            .add_observer(bind_menu);
    }
}

#[derive(InputContext)]
pub struct ActivePlay;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct PaddleUp;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct PaddleDown;

fn bind_active(
    trigger: Trigger<Binding<ActivePlay>>,
    mut actions: Query<&mut Actions<ActivePlay>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<PaddleUp>()
        .to((KeyCode::Space, GamepadButton::South))
        .with_conditions(JustPress::new(1.0));

    actions
        .bind::<PaddleDown>()
        .to((KeyCode::Space, GamepadButton::South))
        .with_conditions(Release::new(1.0));
}

#[derive(InputContext)]
pub struct Menu;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Enter;

fn bind_menu(trigger: Trigger<Binding<Menu>>, mut actions: Query<&mut Actions<Menu>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Enter>()
        .to((KeyCode::Space, KeyCode::Enter, GamepadButton::South));
}

fn action_ctx(
    mut commands: Commands,
    state: Res<State<GameState>>,
    active: Option<Single<Entity, With<Actions<ActivePlay>>>>,
    menu: Option<Single<Entity, With<Actions<Menu>>>>,
) {
    if state.is_changed() || state.is_added() {
        match state.get() {
            GameState::Menu | GameState::Leaderboard => {
                if let Some(entity) = active {
                    commands.entity(*entity).despawn();
                }

                if menu.is_none() {
                    commands.spawn(Actions::<Menu>::default());
                }
            }
            GameState::Selection => {
                if let Some(entity) = active {
                    commands.entity(*entity).despawn();
                }

                if let Some(entity) = menu {
                    commands.entity(*entity).despawn();
                }
            }
            _ => {
                if let Some(entity) = menu {
                    commands.entity(*entity).despawn();
                }

                if active.is_none() {
                    commands.spawn(Actions::<ActivePlay>::default());
                }
            }
        }
    }
}
