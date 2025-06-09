//! The game's menus and transitions between them.

mod game_over;
mod main;
mod pause;
mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        main::plugin,
        settings::plugin,
        pause::plugin,
        game_over::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum Menu {
    #[default]
    None,
    Main,
    Settings,
    Pause,
    GameOver,
}
