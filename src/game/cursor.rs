use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    platform::collections::HashSet,
    prelude::*,
    window::PrimaryWindow,
};
use bevy_rapier2d::{
    plugin::RapierContext,
    prelude::{
        ActiveEvents, Collider, ColliderMassProperties, CollisionEvent, ExternalForce,
        ExternalImpulse, MassProperties, RigidBody, Sensor,
    },
};
use rand::{Rng, seq::SliceRandom};

use crate::{
    AppSystems, PausableSystems, asset_tracking::LoadResource, audio::sound_effect, screens::Screen,
};

use super::{
    enemy::Enemy, explosion::ExplosionAssets, food::Food, player::Player, spawner::SpawnEvent,
};

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
            manual_punch_check_system,
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
    #[dependency]
    sounds: Vec<Handle<AudioSource>>,
    #[dependency]
    swish: Vec<Handle<AudioSource>>,
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
            sounds: vec![
                assets.load("audio/sound_effects/hit.ogg"),
                assets.load("audio/sound_effects/hit1.ogg"),
                assets.load("audio/sound_effects/hit2.ogg"),
                assets.load("audio/sound_effects/hit3.ogg"),
            ],
            swish: vec![
                assets.load("audio/sound_effects/swish.ogg"),
                assets.load("audio/sound_effects/swish3.ogg"),
                assets.load("audio/sound_effects/swish2.ogg"),
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
        Name::new("ursor"),
        Transform::from_xyz(-300.0, 0.0, 0.0),
        RigidBody::KinematicPositionBased,
        Collider::ball(GLOVE_RADIUS),
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
    mut spawn_ew: EventWriter<SpawnEvent>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        if let Ok(mut state) = query.single_mut() {
            if state.is_punching {
                return;
            }
            state.timer.reset();
            state.is_punching = true;
            state.hit_entities.clear();
            spawn_ew.write(SpawnEvent::PunchSwish);
        }
    }
}

pub fn punch_sound(explosion_assets: &CursorAssets) -> impl Bundle {
    let rng = &mut rand::thread_rng();
    let random_punch = explosion_assets.sounds.choose(rng).unwrap().clone();
    sound_effect(random_punch)
}

pub fn punch_swish_sound(explosion_assets: &CursorAssets) -> impl Bundle {
    let rng = &mut rand::thread_rng();
    let random_punch = explosion_assets.swish.choose(rng).unwrap().clone();
    sound_effect(random_punch)
}

const PUNCH_FORCE: f32 = 40000.0;

fn punch_hit_system(
    mut events: EventReader<CollisionEvent>,
    mut glove_query: Query<(&Transform, &mut PunchState), With<Cursor>>,
    mut impulse_query: Query<(&mut ExternalImpulse, &Transform)>,
    enemy_query: Query<(), With<Enemy>>,
    food_query: Query<(), With<Food>>,
    mut spawn_ew: EventWriter<SpawnEvent>,
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
            spawn_ew.write(SpawnEvent::PunchSound);
        }
    }
}

const GLOVE_RADIUS: f32 = 20.0;

fn manual_punch_check_system(
    mut glove_query: Query<(&Transform, &mut PunchState), With<Cursor>>,
    mut impulse_query: Query<(&mut ExternalImpulse, &Transform)>,
    food_query: Query<(Entity, &Transform), With<Food>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut spawn_ew: EventWriter<SpawnEvent>,
) {
    for (glove_transform, mut punch_state) in &mut glove_query {
        if !punch_state.is_punching {
            continue;
        }

        let t = punch_state.timer.elapsed_secs() / punch_state.timer.duration().as_secs_f32();
        if t >= 0.5 {
            continue;
        }

        // Define helper closure to apply punch
        let mut try_punch = |target_entity: Entity, target_transform: &Transform| {
            if !punch_state.hit_entities.insert(target_entity) {
                return;
            }

            if let Ok((mut impulse, _)) = impulse_query.get_mut(target_entity) {
                let punch_direction = glove_transform.rotation * Vec3::Y;
                let offset_direction = (target_transform.translation - glove_transform.translation)
                    .truncate()
                    .normalize_or_zero();
                let punch_dir_2d = punch_direction.truncate().normalize_or_zero();

                let mut direction =
                    (punch_dir_2d * 0.8 + offset_direction * 0.2).normalize_or_zero();

                let mut rng = rand::thread_rng();
                let angle_variation = rng.gen_range(-0.2..0.2);
                direction = (Quat::from_rotation_z(angle_variation) * direction.extend(0.0))
                    .truncate()
                    .normalize_or_zero();

                impulse.impulse += direction * PUNCH_FORCE * 2.0;
                spawn_ew.write(SpawnEvent::PunchSound);
            }
        };

        let glove_pos = glove_transform.translation.truncate();

        for (entity, transform) in &food_query {
            let target_pos = transform.translation.truncate();
            if glove_pos.distance_squared(target_pos) <= GLOVE_RADIUS * GLOVE_RADIUS {
                try_punch(entity, transform);
            }
        }

        for (entity, transform) in &enemy_query {
            let target_pos = transform.translation.truncate();
            if glove_pos.distance_squared(target_pos) <= GLOVE_RADIUS * GLOVE_RADIUS {
                try_punch(entity, transform);
            }
        }
    }
}
