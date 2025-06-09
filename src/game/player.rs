use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{
    Collider, ColliderMassProperties, Damping, ExternalForce, ExternalImpulse,
    KinematicCharacterController, LockedAxes, MassProperties, RigidBody, Velocity,
};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::explosion::Explosion;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    app.init_resource::<PlayerHealth>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            player_movement_system,
            trigger_game_over,
            damage_player_from_explosions,
        )
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_systems(OnEnter(Screen::Gameplay), reset_health);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    player: Handle<Image>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            player: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;
/// The player character.
pub fn player(
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_assets: &PlayerAssets,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    debug!("Creating player");
    (
        Name::new("Player"),
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Dynamic,
        Collider::ball(20.0),
        Velocity::default(),
        Sprite {
            image: player_assets.player.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0, //player_animation.get_atlas_index(),
            }),
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        ExternalImpulse::default(),
        ColliderMassProperties::MassProperties(MassProperties {
            mass: 100.0,
            ..default()
        }),
        StateScoped(Screen::Gameplay),
    )
}

#[derive(Resource)]
pub struct PlayerHealth(usize, Timer);

impl Default for PlayerHealth {
    fn default() -> Self {
        Self(5, Timer::from_seconds(1.0, TimerMode::Once))
    }
}

pub fn reset_health(mut health: ResMut<PlayerHealth>) {
    *health = PlayerHealth::default();
}

fn trigger_game_over(health: Res<PlayerHealth>, mut next_screen: ResMut<NextState<Screen>>) {
    if health.0 == 0 {
        next_screen.set(Screen::GameOver);
    }
}

pub fn damage_player_from_explosions(
    mut health: ResMut<PlayerHealth>,
    player_query: Query<&Transform, With<Player>>,
    explosion_query: Query<(&Transform, &Explosion)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let player_radius = 20.0;

    // Tick the cooldown timer
    health.1.tick(time.delta());

    for (explosion_transform, explosion) in &explosion_query {
        let explosion_pos = explosion_transform.translation.truncate();
        let explosion_radius = explosion.1;

        let distance = player_pos.distance(explosion_pos);
        if distance <= player_radius + explosion_radius && health.1.finished() && health.0 > 0 {
            health.0 -= 1;
            health.1.reset();
            info!("Player hit by explosion! Health now: {}", health.0);
            break;
        }
    }
}

pub const PLAYER_MAX_SPEED: f32 = 200.0;
pub const PLAYER_ACCELERATION: f32 = 1000.0;

fn player_movement_system(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    let direction = direction.normalize_or_zero();
    let delta = time.delta_secs();

    for mut vel in &mut query {
        // Accelerate toward desired direction
        let desired_velocity = direction * PLAYER_MAX_SPEED;

        let diff = desired_velocity - vel.linvel;
        let accel = diff.clamp_length_max(PLAYER_ACCELERATION * delta); // clamp acceleration step

        vel.linvel += accel;
    }
}
