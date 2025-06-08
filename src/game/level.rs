use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use rand::{Rng, thread_rng};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::{cursor::cursor, spawner::spawner},
    screens::Screen,
};

use super::{
    cursor::CursorAssets,
    enemy::{EnemyAssets, enemy},
    food::{FoodAssets, food},
    player::{PlayerAssets, player},
    spawner::SpawnerAssets,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    rock: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            rock: assets.load_with_settings(
                "images/level/rock.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Level;

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,

    cursor_assets: Res<CursorAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    info!("Spawning Level");
    let level_entity = commands.spawn((
        Name::new("Level"),
        Level,
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            player(&mut texture_atlas_layouts, &player_assets),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
            cursor(&cursor_assets),
        ],
    ));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Structure;
/// Creates a bundle of objects to spawn in the level
pub fn structures(map_centre: Transform, level_assets: &LevelAssets) -> impl Bundle {
    let rock = (
        Name::new("Rock"),
        Structure,
        Sprite {
            image: level_assets.rock.clone(),
            color: Color::linear_rgb(1.0, 1.0, 1.0),
            ..default()
        },
        Transform::from_xyz(rand::thread_rng().gen_range(-10.0..10.0), 0.0, 0.0),
    );
    rock
}
