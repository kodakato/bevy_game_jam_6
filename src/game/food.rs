use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{Collider, RigidBody, Velocity};
use rand::{Rng, thread_rng};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::{enemy::eat, level::Level};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<FoodAssets>();
    app.load_resource::<FoodAssets>();

    app.add_systems(
        Update,
        (spawn_food, despawn_eaten_food)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct FoodAssets {
    #[dependency]
    food: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for FoodAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            food: assets.load_with_settings(
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Food(pub isize);

impl Default for Food {
    fn default() -> Self {
        Self(1)
    }
}

pub fn food(
    transform: Transform,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    food_assets: &FoodAssets,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    debug!("Creating food");
    (
        Name::new("Food"),
        Food::default(),
        transform,
        RigidBody::KinematicVelocityBased,
        Collider::ball(1.0),
        Velocity::default(),
        Sprite {
            image: food_assets.food.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, //player_animation.get_atlas_index(),
            }),
            ..default()
        },
    )
}

pub fn despawn_eaten_food(mut commands: Commands, food_query: Query<(Entity, &Food)>) {
    for (entity, food) in food_query {
        if food.0 < 1 {
            commands.entity(entity).despawn();
        }
    }
}

pub const MAX_FOOD: usize = 10;

pub fn spawn_food(
    mut commands: Commands,
    level_query: Query<Entity, With<Level>>,
    food_query: Query<&Food>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    food_assets: Res<FoodAssets>,
) {
    let amount = food_query.iter().count();
    if amount >= MAX_FOOD {
        return;
    }

    let mut rng = thread_rng();
    let x = rng.gen_range(-500.0..500.0);
    let y = rng.gen_range(-500.0..500.0);
    let transform = Transform::from_xyz(x, y, 0.0);

    if let Ok(level) = level_query.single() {
        commands.spawn(food(transform, &mut texture_atlas_layouts, &food_assets));
    };
}
