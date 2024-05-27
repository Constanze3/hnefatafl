use bevy::ecs::{
    entity::Entity,
    event::{Event, EventReader},
    system::Query,
};

use crate::game::tafl::*;

/// Event for setting the possible moves of the selected figure for a certain board (the provided entity).
#[derive(Event)]
pub struct SetPossibleMovesEvent(pub Entity);

/// Sets the possible moves of a selected figures of boards when a SetPossibleMovesEvent is received.
pub fn set_possible_moves(
    mut event: EventReader<SetPossibleMovesEvent>,
    mut q_board: Query<(&Board, &mut SelectedFigure)>,
    q_figures: Query<&Figure>,
) {
    for ev in event.read() {
        let board_entity = ev.0;
        let (board, mut selected_figure) = q_board.get_mut(board_entity).unwrap();

        let Some(selected_figure_entity) = selected_figure.entity else {
            return;
        };

        let figure = q_figures.get(selected_figure_entity).unwrap();

        selected_figure.possible_moves = Some(possible_moves(board, *figure));
    }
}

/// Returns the possible moves for a figure on a board.
/// Pre:
/// - figure is on the board
fn possible_moves(board: &Board, figure: Figure) -> Vec<Position> {
    let position = figure.board_position;
    let mut result: Vec<Position> = vec![];

    result.extend(possible_moves_in_range(
        board,
        figure,
        (0..position.x).rev(),
        Axis2::X,
    ));
    result.extend(possible_moves_in_range(
        board,
        figure,
        (position.x + 1)..board.cols,
        Axis2::X,
    ));

    result.extend(possible_moves_in_range(
        board,
        figure,
        (0..position.y).rev(),
        Axis2::Y,
    ));
    result.extend(possible_moves_in_range(
        board,
        figure,
        (position.y + 1)..board.rows,
        Axis2::Y,
    ));

    result
}

/// Helper function for possible_moves.
/// It checks on an axis sequentially whether a figure can be placed on the positions in the range,
/// when the figure can't be placed on a positon the checking stops.
fn possible_moves_in_range<T>(board: &Board, figure: Figure, range: T, axis: Axis2) -> Vec<Position>
where
    T: IntoIterator<Item = usize>,
{
    let position = figure.board_position;
    let mut result: Vec<Position> = vec![];

    for i in range {
        let targeted_position = match axis {
            Axis2::X => Position {
                x: i,
                y: position.y,
            },
            Axis2::Y => Position {
                x: position.x,
                y: i,
            },
        };

        if can_be_placed_on(board, figure, targeted_position) {
            result.push(targeted_position);
        } else {
            break;
        }
    }

    return result;
}

/// Validates whether a figure may be placed on a certain field or not.
/// Pre:
/// - to_position is on the board
fn can_be_placed_on(board: &Board, figure: Figure, to_position: Position) -> bool {
    let is_king = figure.kind == FigureKind::King;

    let target_contains_figure = board.figures.get(&to_position) != None;
    let target_is_end_pos = board.end_positions.contains(&to_position);
    let target_is_throne = board.throne_position == to_position;

    !target_contains_figure && ((!target_is_end_pos && !target_is_throne) || is_king)
}
