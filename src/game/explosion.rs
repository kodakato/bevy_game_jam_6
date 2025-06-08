use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::Collider;

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ExplosionAssets>();
    app.load_resource::<ExplosionAssets>();

    app.add_systems(
        Update,
        despawn_explosion
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

pub const EXPLOSION_RADIUS: f32 = 70.0;

#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Explosion(pub Timer);

impl Default for Explosion {
    fn default() -> Self {
        Self(Timer::from_seconds(0.2, TimerMode::Once))
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ExplosionAssets {
    #[dependency]
    explosion: Handle<Image>,
}

impl FromWorld for ExplosionAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            explosion: assets.load_with_settings(
                "images/explosion.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

pub fn explosion(
    transform: Transform,
    explosion_assets: &ExplosionAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    (
        Name::from("Explosion"),
        Explosion::default(),
        Sprite {
            image: explosion_assets.explosion.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, //player_animation.get_atlas_index(),
            }),
            ..default()
        },
        Collider::ball(EXPLOSION_RADIUS),
        transform,
    )
}

pub fn despawn_explosion(
    explosion_query: Query<(&mut Explosion, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut explosion, entity) in explosion_query {
        explosion.0.tick(time.delta());
        if !explosion.0.finished() {
            return;
        }
        commands.entity(entity).despawn();
    }
}
