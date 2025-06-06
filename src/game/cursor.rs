use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, MassProperties, RigidBody};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CursorAssets>();
    app.load_resource::<CursorAssets>();

    app.init_resource::<CursorWorldCoords>();

    app.add_systems(
        Update,
        (get_cursor_coords, move_cursor)
            .chain()
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Reflect)]
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
        Cursor::default(),
    )
}
#[derive(Resource, Default)]
struct CursorWorldCoords(Vec2);

fn get_cursor_coords(
    mut mycoords: ResMut<CursorWorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let Ok(window) = q_window.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = q_camera.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
            let world_position = ray.origin.truncate();
            mycoords.0 = world_position;
        }
    }
}

fn move_cursor(
    mut cursor_query: Query<&mut Transform, (With<Cursor>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Cursor>)>,
    cursor_coords: Res<CursorWorldCoords>,
) {
    let Ok(mut cursor_transform) = cursor_query.single_mut() else {
        return;
    };

    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let direction = (cursor_coords.0 - player_transform.translation.truncate()).normalize_or_zero();

    let distance_from_player = 30.0;

    let offset = direction * distance_from_player;
    cursor_transform.translation.x = player_transform.translation.x + offset.x;
    cursor_transform.translation.y = player_transform.translation.y + offset.y;

    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
    cursor_transform.rotation = Quat::from_rotation_z(angle);
}
