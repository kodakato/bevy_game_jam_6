use bevy::{
    ecs::observer::TriggerTargets,
    image::{ImageLoaderSettings, ImageSampler},
    math::NormedVectorSpace,
    prelude::*,
};
use bevy_rapier2d::prelude::{Collider, LockedAxes, RigidBody, Velocity};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};

use super::{
    explosion::{EXPLOSION_RADIUS, Explosion, ExplosionAssets, explosion},
    food::Food,
    player::Player,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<EnemyAssets>();
    app.load_resource::<EnemyAssets>();

    app.add_systems(
        Update,
        (run_to_player, run_to_food, eat, start_explode, explode)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct EnemyAssets {
    #[dependency]
    enemy: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            enemy: assets.load_with_settings(
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Enemy;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Hungry(usize);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Eating;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Hunting;

#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Exploding(pub Timer);

impl Default for Exploding {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Once))
    }
}

pub fn enemy(
    max_speed: f32,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    enemy_assets: &EnemyAssets,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    debug!("Creating enemy");
    (
        Name::new("Enemy"),
        Enemy,
        Hungry(0),
        Transform::from_xyz(-50.0, 0.0, 0.0),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Collider::ball(10.0),
        Velocity::default(),
        Sprite {
            image: enemy_assets.enemy.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, //player_animation.get_atlas_index(),
            }),
            ..default()
        },
    )
}

pub const ENEMY_MAX_SPEED: f32 = 100.0;
pub const ENEMY_ACCELERATION: f32 = 500.0;

pub fn run_to_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<(&Transform, &mut Velocity), (With<Enemy>, With<Hunting>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let delta = time.delta_secs();

    for (enemy_transform, mut velocity) in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();

        // Direction to the player
        let direction = (player_pos - enemy_pos).normalize_or_zero();

        // Accelerate toward the player
        let target_velocity = direction * ENEMY_MAX_SPEED;
        let velocity_diff = target_velocity - velocity.linvel;

        let acceleration_step = velocity_diff.clamp_length_max(ENEMY_ACCELERATION * delta);
        velocity.linvel += acceleration_step;
    }
}

pub fn run_to_food(
    mut commands: Commands,
    time: Res<Time>,
    food_query: Query<(&Transform, &Food)>,
    mut enemy_query: Query<(&Transform, &mut Velocity, Entity), (With<Enemy>, With<Hungry>)>,
) {
    let delta = time.delta_secs();

    // Early exit if there's no food at all
    if food_query.is_empty() {
        return;
    }

    for (enemy_transform, mut velocity, enemy_entity) in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();

        // Find the closest food
        let mut closest_food_pos = None;
        let mut closest_distance = f32::MAX;

        for (food_transform, food) in &food_query {
            let food_pos = food_transform.translation.truncate();
            let dist = food_pos.distance(enemy_pos);

            if dist < closest_distance {
                closest_distance = dist;
                closest_food_pos = Some(food_pos);
            }
        }

        // Check if can eat
        if closest_distance < EAT_DISTANCE {
            commands.entity(enemy_entity).insert(Eating);
            continue;
        }

        // Cant eat, go to nearest food

        if let Some(target_pos) = closest_food_pos {
            let direction = (target_pos - enemy_pos).normalize_or_zero();
            let target_velocity = direction * ENEMY_MAX_SPEED;
            let velocity_diff = target_velocity - velocity.linvel;
            let acceleration_step = velocity_diff.clamp_length_max(ENEMY_ACCELERATION * delta);
            velocity.linvel += acceleration_step;
        }
    }
}

pub const EAT_DISTANCE: f32 = 20.0;
pub const STOMACH_CAP: usize = 10;

pub fn eat(
    mut commands: Commands,
    enemy_query: Query<(&Transform, &mut Hungry, Entity), With<Enemy>>,
    mut food_query: Query<(&Transform, &mut Food)>,
) {
    for (enemy_transform, mut enemy_hungry, entity) in enemy_query {
        for (food_transform, mut food) in &mut food_query {
            let distance = enemy_transform
                .translation
                .distance(food_transform.translation);
            if distance < EAT_DISTANCE {
                food.0 -= 1;
                enemy_hungry.0 += 1;
            }
        }

        if enemy_hungry.0 >= STOMACH_CAP {
            // Convert to hunting
            commands
                .entity(entity)
                .remove::<Eating>()
                .remove::<Hungry>()
                .insert(Hunting);
        }
    }
}

pub const START_EXPLODING_DISTANCE: f32 = 20.0;

pub fn start_explode(
    mut enemy_query: Query<(&Transform, Entity), (With<Enemy>, Without<Exploding>)>,
    explosion_query: Query<&Transform, With<Explosion>>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (enemy_transform, enemy_entity) in enemy_query {
        // Check if near player
        if enemy_transform
            .translation
            .distance(player_transform.translation)
            < START_EXPLODING_DISTANCE
        {
            commands.entity(enemy_entity).insert(Exploding::default());
            continue;
        }

        // Check if near explosion
        for explosion_transform in explosion_query {
            if explosion_transform
                .translation
                .distance(enemy_transform.translation)
                < EXPLOSION_RADIUS
            {
                commands.entity(enemy_entity).insert(Exploding::default());
            }
        }
    }
}

pub fn explode(
    mut enemy_query: Query<(&Transform, Entity, &mut Exploding), With<Enemy>>,
    mut commands: Commands,
    explosion_assets: Res<ExplosionAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    time: Res<Time>,
) {
    for (enemy_transform, enemy_entity, mut exploding) in enemy_query {
        exploding.0.tick(time.delta());

        if exploding.0.finished() {
            commands.entity(enemy_entity).despawn();
            commands.spawn(explosion(
                enemy_transform,
                &explosion_assets,
                &mut texture_atlas_layouts,
            ));
        }
    }
}
