use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, screens::Screen};

use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        move_camera
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

fn move_camera(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    camera_transform.translation = player_transform.translation;
}
