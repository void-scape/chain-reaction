use std::time::Duration;

use crate::collectables::HexColor;
use crate::loading::TextureAssets;
use crate::state::{self, GameState};
use bevy::prelude::*;
use bevy_optix::pixel_perfect::HIGH_RES_LAYER;
use bevy_tween::combinator::{sequence, tween};
use bevy_tween::interpolate::sprite_color;
use bevy_tween::prelude::{AnimationBuilderExt, EaseKind, Repeat};
use bevy_tween::tween::IntoTarget;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(Update, click_play_button.in_set(state::Menu))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Component)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::linear_rgb(0.15, 0.15, 0.15),
            hovered: Color::linear_rgb(0.25, 0.25, 0.25),
        }
    }
}

#[derive(Component)]
struct Menu;

fn setup_menu(mut commands: Commands, server: Res<AssetServer>) {
    commands.spawn((Camera2d, Msaa::Off));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            let button_colors = ButtonColors::default();
            children
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(140.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BackgroundColor(button_colors.normal),
                    button_colors,
                    ChangeState(GameState::StartGame),
                ))
                .with_child((
                    Text::new("Enter?"),
                    TextFont {
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::linear_rgb(0.9, 0.9, 0.9)),
                ));
        });

    commands.spawn((
        Menu,
        HIGH_RES_LAYER,
        Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE)),
        Sprite::from_image(server.load("textures/menu_back.png")),
    ));

    let light = commands
        .spawn((
            Menu,
            HIGH_RES_LAYER,
            Transform::from_scale(Vec3::splat(crate::RESOLUTION_SCALE)).with_translation(Vec3::Z),
            Sprite::from_image(server.load("textures/menu_lights.png")),
        ))
        .id();

    let red: Color = HexColor(0xb4202a).into();
    let orange: Color = HexColor(0xfa6a0a).into();
    let purple: Color = HexColor(0xbc4a9b).into();

    let a = 0.5;
    let red = red.with_alpha(a);
    let orange = orange.with_alpha(a);
    let purple = purple.with_alpha(a);

    commands
        .entity(light)
        .animation()
        .repeat(Repeat::Infinitely)
        .insert(sequence((
            tween(
                Duration::from_secs_f32(1.),
                EaseKind::SineInOut,
                light.into_target().with(sprite_color(red, orange)),
            ),
            tween(
                Duration::from_secs_f32(1.),
                EaseKind::SineInOut,
                light.into_target().with(sprite_color(orange, purple)),
            ),
            tween(
                Duration::from_secs_f32(1.),
                EaseKind::SineInOut,
                light.into_target().with(sprite_color(purple, red)),
            ),
        )));
}

#[derive(Component)]
struct ChangeState(GameState);

#[derive(Component)]
struct OpenLink(&'static str);

fn click_play_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ButtonColors,
            Option<&ChangeState>,
            Option<&OpenLink>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, button_colors, change_state, open_link) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if let Some(state) = change_state {
                    next_state.set(state.0.clone());
                } else if let Some(link) = open_link {
                    if let Err(error) = webbrowser::open(link.0) {
                        warn!("Failed to open link {error:?}");
                    }
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn();
    }
}
