use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    platform::collections::HashSet,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, ColliderMassProperties, CollisionEvent, ExternalForce, ExternalImpulse,
    MassProperties, RigidBody, Sensor,
};
use rand::Rng;

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::{enemy::Enemy, food::Food, player::Player};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CursorAssets>();
    app.load_resource::<CursorAssets>();

    app.init_resource::<CursorWorldCoords>();

    app.add_systems(
        Update,
        (
            (get_cursor_coords, punch_input_system).in_set(AppSystems::RecordInput),
            move_cursor,
            punch_hit_system,
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(PausableSystems),
    );
}

#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct Cursor(pub Timer);

#[derive(Component)]
struct PunchState {
    is_punching: bool,
    timer: Timer,
    hit_entities: HashSet<Entity>,
}

impl Default for PunchState {
    fn default() -> Self {
        Self {
            is_punching: false,
            timer: Timer::from_seconds(0.2, TimerMode::Once),
            hit_entities: HashSet::new(),
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct CursorAssets {
    #[dependency]
    cursor: Handle<Image>,
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
        }
    }
}

pub fn cursor(cursor_assets: &CursorAssets) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    debug!("Creating cursor");
    (
        Name::new("ursor"),
        Transform::from_xyz(-300.0, 0.0, 0.0),
        RigidBody::KinematicPositionBased,
        Collider::ball(20.0),
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
        PunchState::default(),
        ActiveEvents::COLLISION_EVENTS,
        Sensor,
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

const BASE_DISTANCE: f32 = 30.0;
const MAX_DISTANCE: f32 = 50.0;

fn move_cursor(
    time: Res<Time>,
    mut cursor_query: Query<(&mut Transform, &mut PunchState), (With<Cursor>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    cursor_coords: Res<CursorWorldCoords>,
) {
    let Ok((mut cursor_transform, mut punch_state)) = cursor_query.single_mut() else {
        return;
    };
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    if punch_state.is_punching {
        punch_state.timer.tick(time.delta());

        if punch_state.timer.finished() {
            punch_state.is_punching = false;
        }
    }

    let direction = (cursor_coords.0 - player_transform.translation.truncate()).normalize_or_zero();

    let mut punch_percent = 0.0;
    if punch_state.is_punching {
        let t = punch_state.timer.elapsed_secs() / punch_state.timer.duration().as_secs_f32();
        punch_percent = if t < 0.5 { t * 2.0 } else { 2.0 - t * 2.0 };
    }

    let base_distance = BASE_DISTANCE;
    let max_extra = MAX_DISTANCE;
    let distance_from_player = base_distance + punch_percent * max_extra;

    let offset = direction * distance_from_player;
    cursor_transform.translation.x = player_transform.translation.x + offset.x;
    cursor_transform.translation.y = player_transform.translation.y + offset.y;

    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
    cursor_transform.rotation = Quat::from_rotation_z(angle);
}

fn punch_input_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut PunchState, With<Cursor>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(mut state) = query.single_mut() {
            if state.is_punching {
                return;
            }
            state.timer.reset();
            state.is_punching = true;
            state.hit_entities.clear();
        }
    }
}

const PUNCH_FORCE: f32 = 30000.0;

fn punch_hit_system(
    mut events: EventReader<CollisionEvent>,
    mut glove_query: Query<(&Transform, &mut PunchState), With<Cursor>>,
    mut impulse_query: Query<(&mut ExternalImpulse, &Transform)>,
    enemy_query: Query<(), With<Enemy>>,
    food_query: Query<(), With<Food>>,
) {
    for event in events.read() {
        let CollisionEvent::Started(entity1, entity2, _) = *event else {
            continue;
        };

        let (glove_entity, target_entity) = if glove_query.get(entity1).is_ok() {
            (entity1, entity2)
        } else if glove_query.get(entity2).is_ok() {
            (entity2, entity1)
        } else {
            continue;
        };

        let Ok((glove_transform, mut punch_state)) = glove_query.get_mut(glove_entity) else {
            continue;
        };

        if !punch_state.is_punching {
            continue;
        }

        // Only during extension phase
        let t = punch_state.timer.elapsed_secs() / punch_state.timer.duration().as_secs_f32();
        if t >= 0.5 {
            continue;
        }

        if !punch_state.hit_entities.insert(target_entity) {
            continue;
        }

        let is_valid_target =
            enemy_query.get(target_entity).is_ok() || food_query.get(target_entity).is_ok();

        if !is_valid_target {
            continue;
        }

        if let Ok((mut impulse, target_transform)) = impulse_query.get_mut(target_entity) {
            let punch_direction = glove_transform.rotation * Vec3::Y;
            let offset_direction = (target_transform.translation - glove_transform.translation)
                .truncate()
                .normalize_or_zero();
            let punch_dir_2d = punch_direction.truncate().normalize_or_zero();

            // Blend the directions: mostly forward, slightly offset
            let mut direction = (punch_dir_2d * 0.8 + offset_direction * 0.2).normalize_or_zero();

            let mut rng = rand::thread_rng();
            let angle_variation = rng.gen_range(-0.2..0.2);
            direction = (Quat::from_rotation_z(angle_variation) * direction.extend(0.0))
                .truncate()
                .normalize_or_zero();

            impulse.impulse += direction * PUNCH_FORCE;
        }
    }
}
