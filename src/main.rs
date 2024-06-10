use bevy::{
    prelude::*,
    window::{close_on_esc, PrimaryWindow},
    DefaultPlugins,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::GamePlugin;

mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_title)
        .add_systems(Update, close_on_esc)
        .run();
}

fn set_window_title(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = q_window.get_single_mut() {
        window.title = "".to_string();
    }
}
