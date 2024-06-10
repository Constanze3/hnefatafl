use bevy::prelude::*;

use self::camera::*;
use self::main_menu::*;
use self::tafl::*;

mod camera;
mod main_menu;
mod tafl;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            .add_plugins(MainMenuPlugin)
            .add_plugins(TaflPlugin)
            .add_systems(OnEnter(GameState::InGame), spawn_data::spawn_hnefatafl)
            .init_state::<GameState>();
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    MainMenu,
    InGame,
}
