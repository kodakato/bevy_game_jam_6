use bevy::prelude::*;

mod cursor;
pub mod level;
mod physics;
mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        player::plugin,
        level::plugin,
        cursor::plugin,
        physics::plugin,
    ));
}
