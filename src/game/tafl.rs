use bevy::prelude::*;

use self::board::*;
use self::board_highlights::*;
use self::capturing::*;
use self::figure::*;
use self::moving::*;
use self::player_interaction::*;
use self::shieldwall_capturing::*;
use self::spawn_data::*;
use self::spawning::*;

mod board;
mod board_highlights;
mod capturing;
mod figure;
mod moving;
mod player_interaction;
mod shieldwall_capturing;
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
            .add_event::<CaptureEvent>()
            .add_event::<CaptureCheckEvent>()
            .add_event::<ShieldwallCaptureCheckEvent>()
            .add_systems(Update, (spawn_board, spawn_figures).chain())
            .add_systems(
                Update,
                (
                    (
                        on_mouse_pressed,
                        drag_grabbed,
                        on_mouse_released,
                        move_figure,
                        capture_check,
                        shieldwall_capture_check,
                        capture,
                    )
                        .chain(),
                    spawn_highlights.after(on_mouse_pressed),
                    despawn_highlights.after(on_mouse_released),
                ),
            )
            .insert_resource(BoardId::default())
            .insert_resource(SelectedFigure::default());
    }
}
