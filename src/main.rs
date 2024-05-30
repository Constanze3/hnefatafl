use bevy::{prelude::*, window::close_on_esc, DefaultPlugins};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::GamePlugin;

mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GamePlugin)
        .add_systems(Update, close_on_esc)
        .run();
}
