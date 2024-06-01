use bevy::prelude::*;

use self::board::*;
use self::board_highlights::*;
use self::figure::*;
use self::moving::*;
use self::player_interaction::*;
use self::spawn_data::*;
use self::spawning::*;

mod board;
mod board_highlights;
mod capturing;
mod figure;
mod moving;
mod player_interaction;
pub mod spawn_data;
mod spawning;

pub struct TaflPlugin;

impl Plugin for TaflPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<SpawnBoardEvent>()
            .add_event::<SpawnFiguresEvent>()
            .add_event::<SpawnHighlightsEvent>()
            .add_event::<DespawnHighlightsEvent>()
            .add_event::<MoveFigureEvent>()
            .add_systems(Update, (spawn_board, spawn_figures).chain())
            .add_systems(
                Update,
                (
                    (
                        select_figure,
                        drag_selected_figure,
                        release_selected_figure,
                        move_figure,
                    )
                        .chain(),
                    spawn_highlights.after(select_figure),
                    despawn_highlights.after(release_selected_figure),
                ),
            )
            .insert_resource(BoardId::default());
    }
}
