use avian2d::prelude::ColliderDisabled;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_optix::pixel_perfect::{HIGH_RES_LAYER, OuterCamera};

use crate::stage::{AdvanceEvent, StageSet};
use crate::state::{GameState, StateAppExt, remove_entities};
use crate::tower::Tower;
use crate::tower::grid::{SlotTower, SlotTowerOf, TowerSlot};

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
            .add_systems(OnEnter(SelectionState::SpawnSelection), spawn_selection)
            .add_systems(
                Update,
                (select_tower, spawn_tower).run_if(in_state(SelectionState::SelectAndSpawn)),
            );
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

const SELECTIONZ: f32 = 800.;

fn spawn_selection(
    mut commands: Commands,
    _server: Res<AssetServer>,
    mut packs: Single<&mut FeaturePacks>,
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

    let sampler = crate::sampler::Sampler::new(&[
        (Tower::Bumper, 1.),
        (Tower::Dispenser, 0.5),
        (Tower::MoneyBumper, 0.1),
        (Tower::Lotto, 0.3),
    ]);

    let towers = match pack {
        FeaturePack::Starter => (0..3).map(|_| sampler.sample(&mut rng)).collect::<Vec<_>>(),
    };

    let positions = [-300., 0., 300.];
    let y = crate::RES_HEIGHT / 3. - 50.;

    for (tower, x) in towers.into_iter().zip(positions) {
        tower.spawn(
            &mut commands,
            (
                Selection,
                ColliderDisabled,
                Transform::from_xyz(x, y, SELECTIONZ),
                HIGH_RES_LAYER,
            ),
        );

        commands.spawn((
            Selection,
            Text2d::new(format!("{tower:?}")),
            Transform::from_xyz(x, y - 40., SELECTIONZ),
            HIGH_RES_LAYER,
        ));

        let desc = match tower {
            Tower::Bumper => "Bumps balls",
            Tower::Dispenser => "Dispenses balls",
            Tower::MoneyBumper => "Earn money",
            Tower::Lotto => "Play the lottery",
        };

        commands.spawn((
            Selection,
            Text2d::new(desc),
            Transform::from_xyz(x, y - 80., SELECTIONZ),
            HIGH_RES_LAYER,
        ));
    }

    commands.set_state(SelectionState::SelectAndSpawn);
}

fn select_tower(
    mut commands: Commands,
    options: Query<(&Tower, &GlobalTransform), With<Selection>>,

    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,
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

    // check for the nearest tower slot within some threshold

    let Some((selected_tower, transform)) = options.iter().min_by(|a, b| {
        let a = world_position.distance_squared(a.1.translation().xy());
        let b = world_position.distance_squared(b.1.translation().xy());

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

    commands.spawn(SelectedTower(*selected_tower));
    commands.run_system_cached(remove_entities::<With<Selection>>);
}

#[derive(Component)]
struct SelectedTower(Tower);

fn spawn_tower(
    mut commands: Commands,
    slots: Query<(Entity, &GlobalTransform), (With<TowerSlot>, Without<SlotTower>)>,
    selected_tower: Single<(Entity, &SelectedTower)>,

    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<OuterCamera>>,

    packs: Single<(Entity, &FeaturePacks)>,
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

    // check for the nearest tower slot within some threshold

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

    let (entity, selected_tower) = selected_tower.into_inner();
    selected_tower.0.spawn(
        &mut commands,
        (SlotTowerOf(nearest_slot), ChildOf(nearest_slot)),
    );
    commands.entity(entity).despawn();

    let (entity, packs) = packs.into_inner();
    if packs.0.is_empty() {
        commands.set_state(GameState::Playing);
        commands.entity(entity).despawn();
    } else {
        commands.set_state(SelectionState::SpawnSelection);
    }
}
