use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionEvent, RigidBody};
use rand::{Rng, seq::SliceRandom};

use crate::{
    AppSystems, PausableSystems, asset_tracking::LoadResource, audio::sound_effect, screens::Screen,
};

use super::{
    cursor::{CursorAssets, punch_sound, punch_swish_sound},
    enemy::{EnemyAssets, enemy},
    explosion::{Explosion, ExplosionAssets, explosion, explosion_particles},
    food::{FoodAssets, food},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<SpawnerAssets>();
    app.load_resource::<SpawnerAssets>();

    app.add_event::<SpawnEvent>();

    app.add_systems(
        Update,
        (
            spawn_event_handler,
            spawn_enemy,
            damage_spawners_from_explosions,
            tick_cooldown_timers,
        )
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
    #[dependency]
    hit_sound: Handle<AudioSource>,
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
            hit_sound: assets.load("audio/sound_effects/boulder.ogg"),
        }
    }
}
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Spawner(pub Timer, bool);

impl Default for Spawner {
    fn default() -> Self {
        Self(Timer::from_seconds(10.0, TimerMode::Repeating), false)
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SpawnerHealth {
    health: usize,
    cooldown: Timer,
}

const MAX_SPAWNER_HEALTH: usize = 8;

impl Default for SpawnerHealth {
    fn default() -> Self {
        Self {
            health: MAX_SPAWNER_HEALTH,
            cooldown: Timer::from_seconds(2.0, TimerMode::Once),
        }
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
        SpawnerHealth::default(),
        ActiveEvents::COLLISION_EVENTS,
        StateScoped(Screen::Gameplay),
    )
}

pub const SPAWNER_AMOUNT: usize = 5;

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
    PunchSound,
    PunchSwish,
    BoulderSound,
}

pub fn spawn_event_handler(
    mut commands: Commands,
    mut event_reader: EventReader<SpawnEvent>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    enemy_assets: Res<EnemyAssets>,
    food_assets: Res<FoodAssets>,
    explosion_assets: Res<ExplosionAssets>,
    spawner_assets: Res<SpawnerAssets>,
    cursor_assets: Res<CursorAssets>,
    asset_server: Res<AssetServer>,
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
                    position.clone(),
                    &explosion_assets,
                    &mut texture_atlas_layouts,
                ));
                commands.spawn(explosion_particles(&explosion_assets, position.clone()));

                let rng = &mut rand::thread_rng();
                let random_explosion = explosion_assets.sound.choose(rng).unwrap().clone();
                commands.spawn(sound_effect(random_explosion));
            }
            SpawnEvent::Pipe { position } => {
                commands.spawn(spawner(
                    position,
                    &mut texture_atlas_layouts,
                    &spawner_assets,
                ));
            }
            SpawnEvent::PunchSound => {
                commands.spawn(punch_sound(&cursor_assets));
            }
            SpawnEvent::PunchSwish => {
                commands.spawn(punch_swish_sound(&cursor_assets));
            }
            SpawnEvent::BoulderSound => {
                commands.spawn(sound_effect(spawner_assets.hit_sound.clone()));
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
        if spawner.0.finished() && !spawner.1 {
            let mut position = spawner_transform.clone();
            position.translation.x -= SPAWNER_SIZE;
            spawn_ew.write(SpawnEvent::Enemy { position });
        }
    }
}

pub fn damage_spawners_from_explosions(
    mut spawner_query: Query<(&Transform, &mut SpawnerHealth, &mut Sprite, &mut Spawner)>,
    explosion_query: Query<(&Transform, &Explosion)>,
    time: Res<Time>,
    mut spawn_ew: EventWriter<SpawnEvent>,
) {
    for (spawner_transform, mut health, mut sprite, mut spawner) in &mut spawner_query {
        health.cooldown.tick(time.delta());

        let spawner_pos = spawner_transform.translation.truncate();
        let spawner_radius = SPAWNER_SIZE / 2.0;

        for (explosion_transform, explosion) in &explosion_query {
            let explosion_pos = explosion_transform.translation.truncate();
            let explosion_radius = explosion.1;

            let distance = spawner_pos.distance(explosion_pos);
            if distance <= spawner_radius + explosion_radius {
                if health.cooldown.finished() && health.health > 0 {
                    health.health -= 1;
                    health.cooldown.reset();

                    if health.health == 0 {
                        spawner.1 = true;
                        sprite.color = Color::BLACK;
                    } else {
                        let ratio = health.health as f32 / MAX_SPAWNER_HEALTH as f32;
                        // Fade from bright red to black
                        let red = 0.3 + 0.7 * ratio;
                        let green = 0.1 * ratio;
                        let blue = 0.1 * ratio;
                        sprite.color = Color::srgb(red, green, blue);
                    }

                    spawn_ew.write(SpawnEvent::BoulderSound);
                    info!("Spawner damaged by explosion! Health: {}", health.health);
                }
            }
        }
    }
}
fn tick_cooldown_timers(time: Res<Time>, query: Query<&mut SpawnerHealth>) {
    for mut health in query {
        health.cooldown.tick(time.delta());
    }
}
