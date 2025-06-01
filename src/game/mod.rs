use bevy::prelude::*;

pub mod level;
mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((player::plugin, level::plugin));
}
