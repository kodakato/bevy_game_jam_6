use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_rapier2d::prelude::{
    Collider, Damping, ExternalForce, ExternalImpulse, KinematicCharacterController, RigidBody,
    Velocity,
};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        player_movement_system
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    player: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
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
pub struct Player;
/// The player character.
pub fn player(
    max_speed: f32,
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
        Transform::from_xyz(-50.0, 0.0, 0.0),
        RigidBody::KinematicVelocityBased,
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
    )
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
