//! The game over screen that appears after losing.

use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GameOver), open_game_over_menu);
    app.add_systems(OnExit(Screen::GameOver), close_menu);
}

fn open_game_over_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::GameOver);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
