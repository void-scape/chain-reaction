use avian2d::prelude::ColliderDisabled;
use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};
use convert_case::{Case, Casing};

use crate::feature::grid::{FeatureSlot, SlotFeature, SlotFeatureOf};
use crate::feature::{Feature, FeatureSpawner, Rarity};
use crate::sandbox;
use crate::stage::{AdvanceEvent, StageSet};
use crate::state::{GameState, Playing, StateAppExt, remove_entities};
use crate::tooltips::Tooltips;

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
    features: Query<(&Tooltips, &Rarity, &FeatureSpawner)>,

    mut rare_offset: Local<f32>,
) {
    commands.spawn((
        Selection,
        Sprite::from_color(
            Color::BLACK.with_alpha(0.95),
            Vec2::new(crate::RES_WIDTH, crate::RES_HEIGHT),
        ),
        Transform::from_xyz(0., 0., SELECTIONZ - 1.),
    ));

    commands.spawn((
        Selection,
        Text2d::new("SELECT A TOWER"),
        HIGH_RES_LAYER,
        Transform::from_xyz(0., crate::RES_HEIGHT / 3., SELECTIONZ),
    ));

    let mut rng = rand::thread_rng();

    let Some(pack) = packs.0.pop() else {
        debug_assert!(false, "`FeaturePacks` has 0 packs");
        return;
    };

    let samples = features
        .iter()
        .map(|(tips, prob, spawner)| ((tips, spawner, prob), prob.as_prob(*rare_offset)))
        .collect::<Vec<_>>();

    const RARE_INCREASE: f32 = 0.05;
    if samples
        .iter()
        .any(|((_, _, prob), _)| matches!(prob, Rarity::Rare))
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

    for ((tips, spawner, _), x) in features.into_iter().zip(positions) {
        let mut selection = commands.spawn((
            spawner.clone(),
            Selection,
            SelectionFeature,
            ColliderDisabled,
            Transform::from_xyz(x, y, SELECTIONZ),
        ));
        spawner.0(&mut selection);

        commands.spawn((
            Selection,
            Text2d::new(tips.name.to_case(Case::Title)),
            Transform::from_xyz(x, y - 40., SELECTIONZ),
            HIGH_RES_LAYER,
        ));

        commands.spawn((
            Selection,
            Text2d::new(tips.desc),
            Transform::from_xyz(x, y - 80., SELECTIONZ),
            HIGH_RES_LAYER,
        ));
    }

    commands.set_state(SelectionState::SelectAndSpawn);
}

fn select_feature(
    mut commands: Commands,
    options: Query<
        (Entity, &FeatureSpawner, &GlobalTransform),
        (With<SelectionFeature>, With<Feature>),
    >,

    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,

    selection_entities: Query<Entity, With<Selection>>,
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

    let Some((_, selected_feature, transform)) = options.iter().min_by(|a, b| {
        let a = world_position.distance_squared(a.2.translation().xy());
        let b = world_position.distance_squared(b.2.translation().xy());

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

    commands.spawn(SelectedFeature(selected_feature.clone()));
    if !sandbox::ENABLED {
        for entity in selection_entities.iter() {
            commands
                .entity(entity)
                .insert_recursive::<Children>(Disabled);
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
