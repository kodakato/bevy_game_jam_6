use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, MassProperties, RigidBody};

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CursorAssets>();
    app.load_resource::<CursorAssets>();
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Cursor(pub Timer);

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct CursorAssets {
    #[dependency]
    cursor: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for CursorAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            cursor: assets.load_with_settings(
                "images/glove.png",
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

pub fn cursor(cursor_assets: &CursorAssets) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    debug!("Creating cursor");
    (
        Name::new("Cursor"),
        Transform::from_xyz(-300.0, 0.0, 0.0),
        RigidBody::KinematicPositionBased,
        Collider::ball(10.0),
        ColliderMassProperties::MassProperties(MassProperties {
            mass: 10.0,
            ..default()
        }),
        Sprite {
            image: cursor_assets.cursor.clone(),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
    )
}
