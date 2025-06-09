use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_enoki::{Particle2dEffect, ParticleEffectHandle, ParticleSpawner, prelude::OneShot};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, ExternalImpulse, Sensor};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ExplosionAssets>();
    app.load_resource::<ExplosionAssets>();

    app.add_systems(
        Update,
        (
            despawn_explosion,
            explosion_animation,
            explosion_force_system,
        )
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

pub const EXPLOSION_RADIUS: f32 = 70.0;

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Explosion(pub Timer, pub f32);

impl Default for Explosion {
    fn default() -> Self {
        Self::new(50.0)
    }
}

impl Explosion {
    pub fn new(size: f32) -> Self {
        let radius = size.max(1.0); // safety

        // Map size 50–110 to t in 0.0–1.0
        let t = ((radius - 50.0) / 60.0).clamp(0.0, 1.0);
        let duration = 0.05 + t * (0.3 - 0.1); // 0.05 → 0.4

        debug!("Creating explosion with size {size}, duration {duration}");

        let timer = Timer::from_seconds(duration, TimerMode::Once);
        Self(timer, size)
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ExplosionAssets {
    #[dependency]
    explosion: Handle<Image>,
    #[dependency]
    shader: Handle<Particle2dEffect>,
    #[dependency]
    pub sound: Vec<Handle<AudioSource>>,
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
            shader: assets.load("shaders/explosion.ron"),
            sound: vec![
                assets.load("audio/sound_effects/explosion.ogg"),
                assets.load("audio/sound_effects/explosion1.ogg"),
                assets.load("audio/sound_effects/explosion2.ogg"),
                assets.load("audio/sound_effects/explosion3.ogg"),
            ],
        }
    }
}

pub fn explosion(
    size: f32,
    transform: Transform,
    explosion_assets: &ExplosionAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    (
        Name::from("Explosion"),
        Explosion::new(size),
        Sprite {
            image: explosion_assets.explosion.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, //player_animation.get_atlas_index(),
            }),
            custom_size: Some(Vec2::splat(size * 2.0 * 0.9)),
            ..default()
        },
        Collider::ball(size),
        transform,
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        StateScoped(Screen::Gameplay),
    )
}

pub fn explosion_particles(assets: &ExplosionAssets, transform: Transform) -> impl Bundle {
    (
        Name::from("Explosion Particle Spawner"),
        ParticleSpawner::default(),
        ParticleEffectHandle(assets.shader.clone()),
        transform,
        OneShot::Despawn,
    )
}

pub fn explosion_animation(
    time: Res<Time>,
    mut query: Query<(&mut Explosion, &mut Sprite), With<Explosion>>,
) {
    for (mut explosion, mut sprite) in &mut query {
        explosion.0.tick(time.delta());

        let timer = &explosion.0;
        let progress = (timer.elapsed_secs() / timer.duration().as_secs_f32()).clamp(0.0, 1.0);

        // Assuming 5 frames in the explosion atlas
        let total_frames = 5;
        let frame_index = (progress * (total_frames as f32)) as usize;

        sprite.texture_atlas.as_mut().map(|atlas| {
            atlas.index = frame_index.min(total_frames - 1); // prevent overflow
        });
    }
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

const EXPLOSION_FORCE: f32 = 12000.0;

pub fn explosion_force_system(
    explosion_query: Query<(&Transform, &Explosion)>,
    mut affected_query: Query<(&Transform, &mut ExternalImpulse), Without<Explosion>>,
) {
    for (explosion_transform, explosion) in &explosion_query {
        let explosion_pos = explosion_transform.translation.truncate();
        let explosion_radius = explosion.1;

        for (target_transform, mut impulse) in &mut affected_query {
            let target_pos = target_transform.translation.truncate();
            let distance = explosion_pos.distance(target_pos);

            if distance <= explosion_radius {
                let direction = (target_pos - explosion_pos).normalize_or_zero();
                let strength =
                    EXPLOSION_FORCE * (1.0 - (distance / explosion_radius).clamp(0.0, 1.0));
                impulse.impulse += direction * strength;
            }
        }
    }
}
