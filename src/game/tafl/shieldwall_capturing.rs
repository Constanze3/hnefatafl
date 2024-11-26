use crate::game::tafl::*;
use std::collections::VecDeque;

/// Performs shieldwall capture checks from `initial_entity`'s position
///
/// A shieldwall capture is a type of capture where enemy figures
/// surrounded at the edge of the board get captured.
///
/// Returns `true` if something was captured `false` otherwise.
pub fn shieldwall_capture_check(
    q_board: &mut Query<&mut Board>,
    q_figure: &Query<&Figure>,
    board_entity: Entity,
    initial_entity: Entity,
    commands: &mut Commands,
) -> bool {
    let board = q_board.get(board_entity).unwrap();
    let to_capture = determine_shieldwall_capture(initial_entity, &board, &q_figure);

    let mut capture_happened = false;

    for figure_entity in &to_capture {
        let figure = q_figure.get(*figure_entity).unwrap();

        // the king can't be captured
        if figure.kind == FigureKind::King {
            continue;
        }

        capture(q_board, q_figure, board_entity, *figure_entity, commands);
        capture_happened |= true;
    }

    return capture_happened;
}

#[derive(Clone, Copy)]
enum EdgeSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// Determines whether the provided position is at the edge of the board and if yes then on
/// which side of the board the edge is on.
///
/// Pre:
/// - figures can't be on a corner square so they can only have one associated side at a time
fn determine_edge_side(board: &Board, position: Position) -> Option<EdgeSide> {
    if position.x == 0 {
        return Some(EdgeSide::Left);
    }

    if position.x == board.cols - 1 {
        return Some(EdgeSide::Right);
    }

    if position.y == 0 {
        return Some(EdgeSide::Top);
    }

    if position.y == board.rows - 1 {
        return Some(EdgeSide::Bottom);
    }

    return None;
}

/// Determines the vector of entities that are in a shieldwall capture.
/// The checking starts from the `initial_entity`.
fn determine_shieldwall_capture(
    initial_entity: Entity,
    board: &Board,
    q_figure: &Query<&Figure>,
) -> Vec<Entity> {
    let initial_figure = q_figure.get(initial_entity).unwrap();

    let Some(axis) = determine_edge_side(board, initial_figure.position) else {
        return Vec::new();
    };

    let mut to_check: VecDeque<Entity> = VecDeque::new();
    let mut result: Vec<Entity> = Vec::new();

    to_check.push_front(initial_entity);

    while 0 < to_check.len() {
        let entity = to_check.pop_back().unwrap();
        let figure = q_figure.get(entity).unwrap();
        let position = figure.position;

        if !handle_non_axis_neighbor_position(board, figure, axis, q_figure) {
            return Vec::new();
        }

        match axis {
            EdgeSide::Left => {
                if !handle_y_axis_neighbor_positions(
                    position,
                    board,
                    q_figure,
                    figure,
                    &result,
                    &mut to_check,
                ) {
                    return Vec::new();
                }
            }
            EdgeSide::Right => {
                if !handle_y_axis_neighbor_positions(
                    position,
                    board,
                    q_figure,
                    figure,
                    &result,
                    &mut to_check,
                ) {
                    return Vec::new();
                }
            }
            EdgeSide::Top => {
                if !handle_x_axis_neighbor_positions(
                    position,
                    board,
                    q_figure,
                    figure,
                    &result,
                    &mut to_check,
                ) {
                    return Vec::new();
                }
            }
            EdgeSide::Bottom => {
                if !handle_x_axis_neighbor_positions(
                    position,
                    board,
                    q_figure,
                    figure,
                    &result,
                    &mut to_check,
                ) {
                    return Vec::new();
                }
            }
        }

        result.push(entity);
    }

    return result;
}

/// Handles the non-axis neighbor position of the checked figure.
/// Returns:
/// true - if the checking should continue
/// false - if the evaluated figures are not in captured by a shieldwall
fn handle_non_axis_neighbor_position(
    board: &Board,
    figure: &Figure,
    edge_side: EdgeSide,
    q_figure: &Query<&Figure>,
) -> bool {
    let position = figure.position;

    let neighbor_position = 'blk: {
        fn board_size_panic() {
            panic!("board should be at least be 2 x 2");
        }

        match edge_side {
            EdgeSide::Left => {
                if position.x < board.cols - 1 {
                    break 'blk Position {
                        x: position.x + 1,
                        y: position.y,
                    };
                } else {
                    board_size_panic();
                }
            }
            EdgeSide::Right => {
                if 1 <= board.cols - 1 {
                    break 'blk Position {
                        x: position.x - 1,
                        y: position.y,
                    };
                } else {
                    board_size_panic();
                }
            }
            EdgeSide::Top => {
                if position.y < board.rows - 1 {
                    break 'blk Position {
                        x: position.x,
                        y: position.y + 1,
                    };
                } else {
                    board_size_panic();
                }
            }
            EdgeSide::Bottom => {
                if 1 <= position.y {
                    break 'blk Position {
                        x: position.x,
                        y: position.y - 1,
                    };
                } else {
                    board_size_panic();
                }
            }
        }

        panic!("should never be able to get here");
    };

    if let Some(neighbor_entity) = board.figures.get(&neighbor_position) {
        let neighbor_figure = q_figure.get(*neighbor_entity).unwrap();

        if figure.side == neighbor_figure.side {
            return false;
        }
    } else {
        return false;
    }

    // the field is occupied by a figure of the other side
    true
}

/// Handles an axis neighbor position of the checked figure.
/// Returns:
/// true - if the checking should continue
/// false - if the evaluated figures are not in captured by a shieldwall
fn handle_axis_neighbor_position(
    neighbor_position: Position,
    board: &Board,
    q_figure: &Query<&Figure>,
    figure: &Figure,
    result: &Vec<Entity>,
    to_check: &mut VecDeque<Entity>,
) -> bool {
    if let Some(neighbor_entity) = board.figures.get(&neighbor_position) {
        let neighbor_figure = q_figure.get(*neighbor_entity).unwrap();

        // if there is a neighbor on the axis with with the same "color"
        // it should also be checked
        if figure.side == neighbor_figure.side && !result.contains(neighbor_entity) {
            to_check.push_front(*neighbor_entity);
        }

        return true;
    }

    if board.end_positions.contains(&neighbor_position) {
        return true;
    }

    // there is a regular field at either end of the "row" that is empty
    return false;
}

/// Handles the x-axis neighbor positions of the checked figure.
/// Returns:
/// true - if the checking should continue
/// false - if the evaluated figures are not in captured by a shieldwall
fn handle_x_axis_neighbor_positions(
    position: Position,
    board: &Board,
    q_figure: &Query<&Figure>,
    figure: &Figure,
    result: &Vec<Entity>,
    to_check: &mut VecDeque<Entity>,
) -> bool {
    if 0 <= position.x as isize - 1 {
        let should_continue = handle_axis_neighbor_position(
            Position {
                x: position.x - 1,
                y: position.y,
            },
            board,
            q_figure,
            figure,
            result,
            to_check,
        );

        if !should_continue {
            return false;
        }
    }

    if position.x + 1 < board.cols {
        let should_continue = handle_axis_neighbor_position(
            Position {
                x: position.x + 1,
                y: position.y,
            },
            board,
            q_figure,
            figure,
            result,
            to_check,
        );

        if !should_continue {
            return false;
        }
    }

    return true;
}

/// Handles the y-axis neighbor positions of the checked figure.
/// Returns:
/// true - if the checking should continue
/// false - if the evaluated figures are not in captured by a shieldwall
fn handle_y_axis_neighbor_positions(
    position: Position,
    board: &Board,
    q_figure: &Query<&Figure>,
    figure: &Figure,
    result: &Vec<Entity>,
    to_check: &mut VecDeque<Entity>,
) -> bool {
    if 0 <= position.y as isize - 1 {
        let should_continue = handle_axis_neighbor_position(
            Position {
                x: position.x,
                y: position.y - 1,
            },
            board,
            q_figure,
            figure,
            result,
            to_check,
        );

        if !should_continue {
            return false;
        }
    }

    if position.y + 1 < board.rows {
        let should_continue = handle_axis_neighbor_position(
            Position {
                x: position.x,
                y: position.y + 1,
            },
            board,
            q_figure,
            figure,
            result,
            to_check,
        );

        if !should_continue {
            return false;
        }
    }

    return true;
}
