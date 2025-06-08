use bevy::{
    ecs::observer::TriggerTargets,
    image::{ImageLoaderSettings, ImageSampler},
    math::NormedVectorSpace,
    prelude::*,
};
use bevy_rapier2d::{
    prelude::{
        ActiveEvents, AdditionalMassProperties, Collider, ColliderMassProperties, CollisionEvent,
        Damping, ExternalForce, ExternalImpulse, LockedAxes, MassProperties, RigidBody, Velocity,
    },
    rapier::prelude::ColliderMassProps,
};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::{
    explosion::{EXPLOSION_RADIUS, Explosion, ExplosionAssets, explosion},
    food::Food,
    player::Player,
    spawner::SpawnEvent,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<EnemyAssets>();
    app.load_resource::<EnemyAssets>();

    app.add_event::<StartExplodingEvent>();

    app.add_systems(
        Update,
        (
            run_to_player,
            run_to_food,
            eat,
            start_explode,
            explode,
            start_explode_near_player,
            start_exploding_event_handler,
            tick_eat_cooldown,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct EnemyAssets {
    #[dependency]
    enemy: Handle<Image>,
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
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Enemy {
    speed: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self { speed: 2.0 }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Hungry(usize, Timer);

impl Default for Hungry {
    fn default() -> Self {
        Self(0, Timer::from_seconds(0.1, TimerMode::Once))
    }
}

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
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

pub fn enemy(
    transform: Transform,
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
        Enemy::default(),
        Hungry::default(),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Collider::ball(10.0),
        Velocity::default(),
        Damping {
            linear_damping: 0.9,
            ..default()
        },
        ColliderMassProperties::MassProperties(MassProperties {
            mass: 100.0,
            ..default()
        }),
        Sprite {
            image: enemy_assets.enemy.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, //player_animation.get_atlas_index(),
            }),
            ..default()
        },
        transform,
        ExternalImpulse::default(),
        ActiveEvents::COLLISION_EVENTS,
    )
}

#[derive(Event)]
pub struct StartExplodingEvent {
    entity: Entity,
}

fn start_exploding_event_handler(
    mut start_exploding_er: EventReader<StartExplodingEvent>,
    mut enemy_query: Query<&mut Velocity, With<Enemy>>,
    mut commands: Commands,
) {
    for event in start_exploding_er.read() {
        let Ok(mut velocity) = enemy_query.get_mut(event.entity) else {
            continue;
        };
        velocity.linvel *= 0.5;
        commands
            .entity(event.entity)
            .remove::<Hunting>()
            .remove::<Hungry>()
            .insert(Exploding::default());
    }
}

pub const ENEMY_MAX_SPEED_BASE: f32 = 100.0;
pub const ENEMY_ACCELERATION: f32 = 500.0;

pub fn run_to_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut Velocity, &Enemy),
        (With<Enemy>, With<Hunting>, Without<Exploding>),
    >,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let delta = time.delta_secs();

    for (enemy_transform, mut velocity, enemy) in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();

        // Direction to the player
        let direction = (player_pos - enemy_pos).normalize_or_zero();

        // Accelerate toward the player
        let target_velocity = direction * ENEMY_MAX_SPEED_BASE * enemy.speed;
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

    if food_query.is_empty() {
        return;
    }

    for (enemy_transform, mut velocity, enemy_entity) in &mut enemy_query {
        let enemy_pos = enemy_transform.translation.truncate();

        // Find the closest food
        let mut closest_food_pos = None;
        let mut closest_distance = f32::MAX;

        for (food_transform, _) in &food_query {
            let food_pos = food_transform.translation.truncate();
            let dist = food_pos.distance(enemy_pos);

            if dist < closest_distance {
                closest_distance = dist;
                closest_food_pos = Some(food_pos);
            }
        }

        // Cant eat, go to nearest food

        if let Some(target_pos) = closest_food_pos {
            let direction = (target_pos - enemy_pos).normalize_or_zero();
            let target_velocity = direction * ENEMY_MAX_SPEED_BASE;
            let velocity_diff = target_velocity - velocity.linvel;
            let acceleration_step = velocity_diff.clamp_length_max(ENEMY_ACCELERATION * delta);
            velocity.linvel += acceleration_step;
        }
    }
}

const STOMACH_CAP: usize = 5;
const ENEMY_SPEED_DELTA: f32 = 5.0;
const BOUNCE_FORCE: f32 = 30000.0;

pub fn eat(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut food_query: Query<(&Transform, &mut Food)>,
    mut enemy_query: Query<
        (
            Entity,
            &Transform,
            &mut Hungry,
            &mut Enemy,
            &mut ExternalImpulse,
        ),
        With<Enemy>,
    >,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = *event else {
            continue;
        };

        // Determine which entity is food and which is enemy
        let (food_entity, enemy_entity) =
            if food_query.get(e1).is_ok() && enemy_query.get(e2).is_ok() {
                (e1, e2)
            } else if food_query.get(e2).is_ok() && enemy_query.get(e1).is_ok() {
                (e2, e1)
            } else {
                continue;
            };

        let Ok((food_transform, mut food)) = food_query.get_mut(food_entity) else {
            continue;
        };

        let Ok((enemy_ent, enemy_transform, mut hungry, mut enemy, mut impulse)) =
            enemy_query.get_mut(enemy_entity)
        else {
            continue;
        };

        // Only eat if there's food left
        if food.0 == 0 {
            continue;
        }

        if !hungry.1.finished() {
            continue;
        }

        // Eat one unit of food
        food.0 -= 1;
        hungry.0 += 1;
        enemy.speed += ENEMY_SPEED_DELTA;

        hungry.1.reset();

        // Bounce away from the food
        let direction = (enemy_transform.translation - food_transform.translation)
            .truncate()
            .normalize_or_zero();
        impulse.impulse += direction * BOUNCE_FORCE;

        // Check if full
        if hungry.0 >= STOMACH_CAP {
            debug!("HUNTING");
            commands
                .entity(enemy_ent)
                .remove::<Eating>()
                .remove::<Hungry>()
                .insert(Hunting);
        }
    }
}

fn tick_eat_cooldown(time: Res<Time>, mut enemy_query: Query<&mut Hungry>) {
    for mut hungry in enemy_query {
        hungry.1.tick(time.delta());
    }
}

pub const START_EXPLODING_DISTANCE: f32 = 80.0;

pub fn start_explode(
    enemy_query: Query<(&Transform, Entity), (With<Enemy>, Without<Exploding>)>,
    explosion_query: Query<&Transform, With<Explosion>>,
    mut start_exploding_ew: EventWriter<StartExplodingEvent>,
) {
    for (enemy_transform, enemy_entity) in enemy_query {
        // Check if near explosion
        for explosion_transform in explosion_query {
            if explosion_transform
                .translation
                .distance(enemy_transform.translation)
                < EXPLOSION_RADIUS
            {
                start_exploding_ew.write(StartExplodingEvent {
                    entity: enemy_entity,
                });
            }
        }
    }
}

pub fn start_explode_near_player(
    enemy_query: Query<(&Transform, Entity), (With<Enemy>, With<Hunting>, Without<Exploding>)>,
    player_query: Query<&Transform, With<Player>>,
    mut start_exploding_ew: EventWriter<StartExplodingEvent>,
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
            start_exploding_ew.write(StartExplodingEvent {
                entity: enemy_entity,
            });
            continue;
        }
    }
}

pub fn explode(
    enemy_query: Query<(&Transform, Entity, &mut Exploding), With<Enemy>>,
    mut commands: Commands,
    mut spawn_ew: EventWriter<SpawnEvent>,
    time: Res<Time>,
) {
    for (enemy_transform, enemy_entity, mut exploding) in enemy_query {
        exploding.0.tick(time.delta());

        if exploding.0.finished() {
            commands.entity(enemy_entity).despawn();
            spawn_ew.write(SpawnEvent::Explosion {
                position: enemy_transform.clone(),
            });
        }
    }
}
