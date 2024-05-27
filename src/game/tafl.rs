use bevy::prelude::*;

pub use self::board::*;
use self::board_highlights::*;
use self::figure::*;
use self::move_validation::*;
pub use self::spawn::*;

mod board;
mod board_highlights;
mod figure;
mod move_validation;
mod spawn;

pub struct TaflPlugin;

impl Plugin for TaflPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<SpawnTaflEvent>()
            .add_event::<SpawnBoardEvent>()
            .add_event::<SpawnFiguresEvent>()
            .add_event::<SetPossibleMovesEvent>()
            .add_event::<SpawnHighlightsEvent>()
            .add_event::<DespawnHighlightsEvent>()
            .add_systems(Update, (spawn_tafl, spawn_board).chain())
            .add_systems(
                Update,
                (set_possible_moves, spawn_highlights, despawn_highlights).chain(),
            );
    }
}
