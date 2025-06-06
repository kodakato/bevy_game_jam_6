use bevy::prelude::*;

mod camera;
mod cursor;
mod enemy;
mod explosion;
mod food;
pub mod level;
mod physics;
mod player;
mod spawner;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        camera::plugin,
        player::plugin,
        level::plugin,
        cursor::plugin,
        physics::plugin,
        explosion::plugin,
        enemy::plugin,
        food::plugin,
        spawner::plugin,
    ));
}
