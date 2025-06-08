use std::ops::Deref;
use std::time::Duration;

use avian2d::prelude::ColliderDisabled;
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};
use bevy_seedling::sample::SamplePlayer;

use crate::collectables::{Money, MoneyEvent};
use crate::feature::grid::{FeatureSlot, SlotFeature, SlotFeatureOf};
use crate::feature::{FeatureSpawner, Price, Rarity};
use crate::sandbox;
use crate::stage::{AdvanceEvent, StageSet};
use crate::state::{GameState, Playing, StateAppExt, remove_entities};
use crate::tooltips::{Hover, ShowTooltips};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct SelectionSet;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SelectionState>()
            .add_event::<SelectionEvent>()
            .add_reset(remove_entities::<With<Selection>>)
            .add_systems(
                PreUpdate,
                (receive_advance, enter)
                    .chain()
                    .after(StageSet)
                    .in_set(SelectionSet),
            )
            .add_systems(Update, (handle_delayed, button_system))
            .add_systems(OnEnter(SelectionState::SpawnSelection), spawn_selection);
        //.add_systems(Update, report_entities);

        if sandbox::ENABLED {
            app.add_systems(
                Update,
                (spawn_feature, select_feature).chain().in_set(Playing),
            );
        } else {
            app.add_systems(
                Update,
                (spawn_feature, select_feature)
                    .chain()
                    .run_if(in_state(SelectionState::SelectAndSpawn)),
            );
        }
    }
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum SelectionState {
    SpawnSelection,
    #[default]
    SelectAndSpawn,
}

#[derive(Event)]
pub struct SelectionEvent {
    pub packs: Vec<FeaturePack>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Component)]
pub enum FeaturePack {
    Starter,
}

impl FeaturePack {
    pub fn triple_starter() -> Vec<Self> {
        vec![Self::Starter; 3]
    }
}

#[derive(Component)]
struct FeaturePacks(Vec<FeaturePack>);

fn receive_advance(mut reader: EventReader<AdvanceEvent>, mut writer: EventWriter<SelectionEvent>) {
    for event in reader.read() {
        match event.level {
            _ => {
                writer.write(SelectionEvent {
                    packs: FeaturePack::triple_starter(),
                });
            }
        }
    }
}

fn enter(mut commands: Commands, mut reader: EventReader<SelectionEvent>) {
    for event in reader.read() {
        debug_assert!(!event.packs.is_empty(), "selection needs atleast 1 pack");
        commands.set_state(GameState::Selection);
        commands.set_state(SelectionState::SpawnSelection);
        commands.spawn(FeaturePacks(event.packs.clone()));
    }
}

#[derive(Component)]
struct Selection;

#[derive(Component)]
pub struct SelectionFeature;

const SELECTIONZ: f32 = 800.;

//#[derive(Component, Clone)]
//struct EntityReporter(String);
//
//fn report_entities(q: Query<(Entity, &EntityReporter)>, mut commands: Commands) {
//    for (entity, reporter) in q.iter() {
//        commands
//            .entity(entity)
//            .log_components()
//            .remove::<EntityReporter>();
//    }
//}

fn spawn_selection(
    mut commands: Commands,
    mut packs: Single<&mut FeaturePacks>,
    features: Query<(&Rarity, &FeatureSpawner)>,

    mut rare_offset: Local<f32>,
) {
    let mut rng = rand::thread_rng();

    let Some(pack) = packs.0.pop() else {
        debug_assert!(false, "`FeaturePacks` has 0 packs");
        return;
    };

    let samples = features
        .iter()
        .map(|(prob, spawner)| ((spawner, prob), prob.as_prob(*rare_offset)))
        .collect::<Vec<_>>();

    const RARE_INCREASE: f32 = 0.05;
    if samples
        .iter()
        .any(|((_, prob), _)| matches!(prob, Rarity::Rare))
    {
        *rare_offset = 0.;
    } else {
        *rare_offset += RARE_INCREASE / 3.;
    }
    let mut sampler = crate::sampler::Sampler::new(&samples);

    let features = match pack {
        FeaturePack::Starter => sampler.sample_unique(&mut rng, 3),
    };

    let positions = [-300., 0., 300.];
    let y = crate::RES_HEIGHT / 3. - 50.;

    let mut delay = 0.0;
    for ((spawner, _), x) in features.into_iter().zip(positions) {
        let mut selection = commands.spawn((
            spawner.clone(),
            Selection,
            SelectionFeature,
            ColliderDisabled,
            Transform::from_xyz(x, y, SELECTIONZ),
        ));
        spawner.0(&mut selection);
        selection.insert(ShowTooltips { delay });
        delay += 0.1;
    }

    commands.spawn(DelayedSpawn::new(Duration::from_millis(500), button()));

    commands.set_state(SelectionState::SelectAndSpawn);
}

#[derive(Component)]
struct DelayedSpawn {
    data: Option<Box<dyn FnOnce(&mut Commands) + Send + Sync>>,
    timer: Timer,
}

impl DelayedSpawn {
    pub fn new<B: Bundle>(delay: Duration, bundle: B) -> Self {
        Self {
            data: Some(Box::new(move |commands| {
                commands.spawn(bundle);
            })),
            timer: Timer::new(delay, TimerMode::Once),
        }
    }
}

fn handle_delayed(
    mut q: Query<(Entity, &mut DelayedSpawn)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta();
    for (entity, mut spawn) in q.iter_mut() {
        if spawn.timer.tick(delta).just_finished() {
            if let Some(data) = spawn.data.take() {
                data(&mut commands);
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Debug, Component)]
struct SkipButton;

fn button() -> impl Bundle {
    (
        HIGH_RES_LAYER,
        SkipButton,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            top: Val::Px(270.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            HIGH_RES_LAYER,
            SkipButton,
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::new(
                Val::Percent(25.0),
                Val::Percent(25.0),
                Val::Percent(25.0),
                Val::Percent(25.0)
            ),
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            children![(
                Text::new("Skip"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            )]
        )],
    )
}

fn select_feature(
    mut commands: Commands,
    options: Query<
        (Entity, &FeatureSpawner, &GlobalTransform, &Price),
        //(With<SelectionFeature>, With<Feature>),
    >,
    child_ofs: Query<&ChildOf>,
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
    selection_entities: Query<Entity, With<Selection>>,
    hovered: Single<&ChildOf, With<Hover>>,
    money: Res<Money>,
    mut money_event: EventWriter<MoneyEvent>,
    server: Res<AssetServer>,
    skip: Query<Entity, With<SkipButton>>,
) {
    let (camera, gt) = camera.into_inner();
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let Some((_, selected_feature, transform, price)) = options.iter().find(|(entity, ..)| {
        child_ofs
            .get(hovered.parent())
            .is_ok_and(|child_of| child_of.parent() == *entity)
    }) else {
        return;
    };

    if price.0 > money.get() {
        commands.spawn(
            SamplePlayer::new(server.load("audio/pinball/1drop.ogg"))
                .with_volume(bevy_seedling::prelude::Volume::Decibels(-12.0)),
        );
        return;
    }

    money_event.write(MoneyEvent {
        money: -price.0,
        position: transform.translation().xy(),
    });

    //let Some(world_position) = window
    //    .cursor_position()
    //    .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
    //    .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
    //else {
    //    return;
    //};
    //
    //let Some((_, selected_feature, transform)) = options.iter().min_by(|a, b| {
    //    let a = world_position.distance_squared(a.2.translation().xy());
    //    let b = world_position.distance_squared(b.2.translation().xy());
    //
    //    a.total_cmp(&b)
    //}) else {
    //    return;
    //};
    //
    //if transform
    //    .compute_transform()
    //    .translation
    //    .xy()
    //    .distance(world_position)
    //    > 50.0
    //{
    //    return;
    //}

    commands.spawn(SelectedFeature(selected_feature.clone()));
    if !sandbox::ENABLED {
        for entity in selection_entities.iter() {
            commands.entity(entity).despawn();
            //.insert_recursive::<Children>(Disabled);
        }

        for entity in skip.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn button_system(
    mut interaction_query: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SkipButton>),
    >,
    selection_entities: Query<Entity, With<Selection>>,
    mut commands: Commands,
    packs: Option<Single<(Entity, &FeaturePacks)>>,
) {
    for (entity, interaction, mut color) in &mut interaction_query {
        info!("here");
        match *interaction {
            Interaction::Pressed => {
                if !sandbox::ENABLED {
                    for entity in selection_entities.iter() {
                        commands.entity(entity).despawn();
                        //.insert_recursive::<Children>(Disabled);
                    }

                    commands.entity(entity).despawn();

                    if let Some(packs) = &packs {
                        let (entity, packs) = packs.deref();
                        if packs.0.is_empty() {
                            commands.set_state(GameState::Playing);
                            commands.entity(*entity).despawn();
                        } else {
                            commands.set_state(SelectionState::SpawnSelection);
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.4, 0.4, 0.4).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}

#[derive(Component)]
pub struct SelectedFeature(FeatureSpawner);

fn spawn_feature(
    mut commands: Commands,
    slots: Query<(Entity, &GlobalTransform), (With<FeatureSlot>, Without<SlotFeature>)>,
    selected_feature: Single<(Entity, &SelectedFeature)>,

    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,

    packs: Option<Single<(Entity, &FeaturePacks)>>,
) {
    let (camera, gt) = camera.into_inner();
    if !input.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gt, cursor).ok())
        .map(|ray| ray.origin.truncate() / crate::RESOLUTION_SCALE)
    else {
        return;
    };

    let Some((nearest_slot, transform)) = slots.iter().min_by(|a, b| {
        let a = world_position.distance_squared(a.1.compute_transform().translation.xy());
        let b = world_position.distance_squared(b.1.compute_transform().translation.xy());

        a.total_cmp(&b)
    }) else {
        return;
    };

    if transform
        .compute_transform()
        .translation
        .xy()
        .distance(world_position)
        > 50.0
    {
        return;
    }

    let (entity, selected_feature) = selected_feature.into_inner();
    let mut entity_commands = commands.spawn((
        SlotFeatureOf(nearest_slot),
        ChildOf(nearest_slot),
        Transform::default(),
    ));
    selected_feature.0.0(&mut entity_commands);
    commands.entity(entity).despawn();
    commands.run_system_cached(
        remove_entities::<(With<Selection>, Or<(With<Disabled>, Without<Disabled>)>)>,
    );

    if sandbox::ENABLED {
        return;
    }

    if let Some(packs) = packs {
        let (entity, packs) = packs.into_inner();
        if packs.0.is_empty() {
            commands.set_state(GameState::Playing);
            commands.entity(entity).despawn();
        } else {
            commands.set_state(SelectionState::SpawnSelection);
        }
    }
}
