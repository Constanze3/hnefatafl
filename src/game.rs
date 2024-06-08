use bevy::prelude::*;

use self::camera::*;
use self::tafl::*;

mod camera;
mod tafl;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            .add_plugins(TaflPlugin)
            .add_systems(Startup, spawn_data::spawn_hnefatafl);
    }
}
