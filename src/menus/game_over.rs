//! Game Over menu UI.

use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::GameOver), spawn_game_over_ui);
}

fn spawn_game_over_ui(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Game Over UI"),
        GlobalZIndex(2),
        StateScoped(Menu::GameOver),
        children![
            widget::label("Game Over"),
            widget::button("Return to Menu", return_to_menu),
        ],
    ));
}

fn return_to_menu(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
