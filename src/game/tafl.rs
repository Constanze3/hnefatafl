use bevy::prelude::*;

pub use self::board::*;
use self::board_highlights::*;
use self::figure::*;
use self::move_validation::*;
pub use self::spawn::*;
use self::spawn_data::*;
use self::user_interaction::*;

mod board;
mod board_highlights;
mod figure;
mod move_validation;
mod spawn;
pub mod spawn_data;
mod user_interaction;

pub struct TaflPlugin;

impl Plugin for TaflPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<SpawnBoardEvent>()
            .add_event::<SpawnFiguresEvent>()
            .add_event::<SpawnHighlightsEvent>()
            .add_event::<DespawnHighlightsEvent>()
            .add_systems(Update, (spawn_board, spawn_figures).chain())
            .add_systems(
                Update,
                (
                    select_figure,
                    (spawn_highlights, despawn_highlights)
                        .chain()
                        .after(select_figure),
                ),
            )
            .insert_resource(BoardId::default());
    }
}
