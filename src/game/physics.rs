use bevy::prelude::*;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::Velocity,
    render::RapierDebugRenderPlugin,
};

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));
    #[cfg(debug)]
    app.add_plugins(RapierDebugRenderPlugin::default());
    app.add_systems(
        Update,
        (apply_drag)
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

pub const DRAG: f32 = 10.0;

pub fn apply_drag(mut body_query: Query<&mut Velocity>) {
    for mut velocity in body_query {
        // Get its velocity, reverse it, and divide by a factor
        let new_vel = velocity.linvel + velocity.linvel * -1.0 / DRAG;
        //velocity.linvel = new_vel;
    }
}
