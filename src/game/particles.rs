use bevy::prelude::*;
use bevy_enoki::{EnokiPlugin, Particle2dEffect};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(EnokiPlugin);
    app.init_asset::<Particle2dEffect>();
}
