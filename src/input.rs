use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(bind_actions)
            .add_input_context::<ActivePlay>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(Actions::<ActivePlay>::default());
            });
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

fn bind_actions(
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
