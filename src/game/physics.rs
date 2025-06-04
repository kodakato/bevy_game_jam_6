use bevy::prelude::*;
use bevy_rapier2d::{
    plugin::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    prelude::Velocity,
    render::RapierDebugRenderPlugin,
};

use crate::{AppSystems, PausableSystems, Pause};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));
    #[cfg(debug)]
    app.add_plugins(RapierDebugRenderPlugin::default());
    app.add_systems(Startup, setup_rapier);
}

pub fn setup_rapier(mut config: Query<&mut RapierConfiguration>) {
    let mut rapier_config = config.single_mut().unwrap();
    rapier_config.gravity = Vec2::ZERO;
}
