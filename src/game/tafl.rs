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
use self::ui::*;
use self::victory_ui::VictoryUiPlugin;
use self::win_conditions::*;

mod board;
mod board_highlights;
mod capturing;
mod figure;
mod moving;
mod player_interaction;
mod shieldwall_capturing;
pub mod spawn_data;
mod spawning;
mod ui;
mod victory_ui;
mod win_conditions;

pub struct TaflPlugin;

impl Plugin for TaflPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(UiPlugin)
            .add_plugins(VictoryUiPlugin)
            .add_event::<SpawnBoardEvent>()
            .add_event::<SpawnFiguresEvent>()
            .add_event::<SpawnHighlightsEvent>()
            .add_event::<DespawnHighlightsEvent>()
            .add_event::<TryMoveFigureEvent>()
            .add_event::<MoveFigureEvent>()
            .add_event::<ReleaseSelectedFigureEvent>()
            .add_event::<CaptureEvent>()
            .add_event::<CaptureCheckEvent>()
            .add_event::<ShieldwallCaptureCheckEvent>()
            .add_event::<KingOnCornerCheckEvent>()
            .add_event::<KingSurroundedCheckEvent>()
            .add_event::<EndGameEvent>()
            .add_systems(Update, (spawn_board, spawn_figures).chain())
            .add_systems(
                Update,
                (
                    (
                        on_mouse_pressed,
                        drag_grabbed,
                        on_mouse_released,
                        try_move_figure,
                        slide_and_move_figure,
                        move_figure,
                        release_selected_figure,
                        capture_check,
                        shieldwall_capture_check,
                        capture,
                        king_on_corner_check,
                        king_surrounded_check,
                        game_timer_check,
                        on_game_end,
                    )
                        .chain(),
                    spawn_highlights.after(on_mouse_pressed),
                    despawn_highlights.after(on_mouse_released),
                ),
            )
            .insert_resource(BoardId::default())
            .insert_resource(SelectionOptions::default())
            .insert_resource(SelectedFigure::default())
            .insert_resource(MoveFigureOptions::default())
            .insert_resource(GameState::default());
    }
}
