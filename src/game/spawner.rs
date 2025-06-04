use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::enemy::{EnemyAssets, enemy};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpawnerAssets>();
    app.load_resource::<SpawnerAssets>();

    app.add_systems(
        Update,
        spawn_enemies
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct SpawnerAssets {
    #[dependency]
    spawner: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for SpawnerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            spawner: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Spawner(pub Timer);

impl Default for Spawner {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

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
        RigidBody::KinematicVelocityBased,
        Collider::ball(1.0),
        Sprite {
            image: spawner_assets.spawner.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, //player_animation.get_atlas_index(),
            }),
            ..default()
        },
    )
}

pub fn spawn_enemies(
    spawner_query: Query<&mut Spawner>,
    time: Res<Time>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    enemy_assets: Res<EnemyAssets>,
) {
    for (mut spawner) in spawner_query {
        spawner.0.tick(time.delta());
        if !spawner.0.finished() {
            return;
        }
        // Spawn
        commands.spawn(enemy(10.0, &mut texture_atlas_layouts, &enemy_assets));
    }
}
