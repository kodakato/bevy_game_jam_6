use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{Collider, RigidBody};
use rand::Rng;

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::{
    enemy::{EnemyAssets, enemy},
    explosion::{ExplosionAssets, explosion},
    food::{FoodAssets, food},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpawnerAssets>();
    app.load_resource::<SpawnerAssets>();

    app.add_event::<SpawnEvent>();

    app.add_systems(
        Update,
        (spawn_event_handler, spawn_enemy)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_systems(OnEnter(Screen::Gameplay), spawn_spawners);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SpawnerAssets {
    #[dependency]
    spawner: Handle<Image>,
}

impl FromWorld for SpawnerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            spawner: assets.load_with_settings(
                "images/cave.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Spawner(pub Timer);

impl Default for Spawner {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

const SPAWNER_SIZE: f32 = 50.0;
pub fn spawner(
    transform: Transform,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    spawner_assets: &SpawnerAssets,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    debug!("Creating food");
    (
        Name::new("Spawner"),
        Spawner::default(),
        transform,
        RigidBody::Fixed,
        Collider::capsule_x(SPAWNER_SIZE / 3.2, SPAWNER_SIZE / 1.3),
        Sprite {
            image: spawner_assets.spawner.clone(),
            custom_size: Some(Vec2::new(SPAWNER_SIZE * 2.0, SPAWNER_SIZE * 1.8)),
            ..default()
        },
    )
}

pub const SPAWNER_AMOUNT: usize = 3;

pub fn spawn_spawners(mut spawn_ew: EventWriter<SpawnEvent>) {
    for _ in 0..SPAWNER_AMOUNT {
        let x = rand::thread_rng().gen_range(-1000.0..1000.0);
        let y = rand::thread_rng().gen_range(-1000.0..1000.0);

        let transform = Transform::from_xyz(x, y, 0.0);

        spawn_ew.write(SpawnEvent::Pipe {
            position: transform,
        });
    }
}

#[derive(Event)]
pub enum SpawnEvent {
    Enemy { position: Transform },
    Food { position: Transform },
    Explosion { position: Transform, size: f32 },
    Pipe { position: Transform },
}

pub fn spawn_event_handler(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnEvent>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    enemy_assets: Res<EnemyAssets>,
    food_assets: Res<FoodAssets>,
    explosion_assets: Res<ExplosionAssets>,
    spawner_assets: Res<SpawnerAssets>,
) {
    for event in event_reader.read() {
        match *event {
            SpawnEvent::Enemy { position } => {
                commands.spawn(enemy(position, &mut texture_atlas_layouts, &enemy_assets));
            }
            SpawnEvent::Food { position } => {
                commands.spawn(food(position, &food_assets));
            }
            SpawnEvent::Explosion { position, size } => {
                commands.spawn(explosion(
                    size,
                    position,
                    &explosion_assets,
                    &mut texture_atlas_layouts,
                ));
            }
            SpawnEvent::Pipe { position } => {
                commands.spawn(spawner(
                    position,
                    &mut texture_atlas_layouts,
                    &spawner_assets,
                ));
            }
        }
    }
}

fn spawn_enemy(
    mut spawn_ew: EventWriter<SpawnEvent>,
    spawner_query: Query<(&Transform, &mut Spawner)>,
    time: Res<Time>,
) {
    for (spawner_transform, mut spawner) in spawner_query {
        spawner.0.tick(time.delta());
        if spawner.0.finished() {
            let mut position = spawner_transform.clone();
            position.translation.x -= SPAWNER_SIZE;
            spawn_ew.write(SpawnEvent::Enemy { position });
        }
    }
}
