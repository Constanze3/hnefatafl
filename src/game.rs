use bevy::prelude::*;
use bevy::window::close_on_esc;

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
            .add_systems(
                Update,
                (
                    close_on_esc.run_if(in_state(GameState::MainMenu)),
                    main_menu_on_esc.run_if(not(in_state(GameState::MainMenu))),
                ),
            )
            .init_state::<GameState>();
    }
}

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    MainMenu,
    InGame,
}

fn main_menu_on_esc(
    input: Res<ButtonInput<KeyCode>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Escape) {
        next_game_state.set(GameState::MainMenu);
    }
}
