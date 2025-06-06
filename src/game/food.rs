use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{
    Collider, ColliderMassProperties, Damping, ExternalForce, ExternalImpulse, LockedAxes,
    MassProperties, RigidBody, Velocity,
};
use rand::{Rng, thread_rng};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::{enemy::eat, level::Level, spawner::SpawnEvent};

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
}

impl FromWorld for FoodAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            food: assets.load_with_settings(
                "images/cupcake.png",
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
pub struct Food(pub isize);

impl Default for Food {
    fn default() -> Self {
        Self(1)
    }
}

pub fn food(transform: Transform, food_assets: &FoodAssets) -> impl Bundle {
    debug!("Creating food");
    (
        Name::new("Food"),
        Food::default(),
        transform,
        RigidBody::Dynamic,
        Damping {
            linear_damping: 1.0,
            ..default()
        },
        ColliderMassProperties::MassProperties(MassProperties {
            mass: 200.0,
            ..default()
        }),
        LockedAxes::ROTATION_LOCKED,
        Collider::ball(15.0),
        Velocity::default(),
        ExternalImpulse::default(),
        Sprite {
            image: food_assets.food.clone(),
            custom_size: Some(Vec2::new(30.0, 30.0)),
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

pub fn spawn_food(food_query: Query<&Food>, mut spawn_ew: EventWriter<SpawnEvent>) {
    let amount = food_query.iter().count();
    if amount >= MAX_FOOD {
        return;
    }

    let mut rng = thread_rng();
    let x = rng.gen_range(-500.0..500.0);
    let y = rng.gen_range(-500.0..500.0);
    let transform = Transform::from_xyz(x, y, 0.0);

    spawn_ew.write(SpawnEvent::Food {
        position: transform,
    });
}
