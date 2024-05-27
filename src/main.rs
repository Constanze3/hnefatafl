use bevy::{prelude::*, window::close_on_esc, DefaultPlugins};
use game::GamePlugin;

mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .add_systems(Update, close_on_esc)
        .run();
}
