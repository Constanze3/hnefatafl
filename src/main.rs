#![windows_subsystem = "windows"]

use bevy::{prelude::*, window::PrimaryWindow, DefaultPlugins};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use game::GamePlugin;

mod game;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_title)
        .run();
}

fn set_window_title(mut q_window: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = q_window.single_mut();
    window.title = "".to_string();
}

pub fn close_on_esc(
    input: Res<ButtonInput<KeyCode>>,
    q_window: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::Escape) {
        let window_entity = q_window.single();
        commands.entity(window_entity).despawn();
    }
}
