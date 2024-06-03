use crate::game::tafl::*;

use self::win_conditions::KingOnCornerCheckEvent;

#[derive(Event)]
pub struct MoveFigureEvent {
    pub board_entity: Entity,
    pub from: Position,
    pub to: Position,
}

#[derive(Component)]
pub struct TurnTracker {
    pub side: Side,
}

impl TurnTracker {
    pub fn next_turn(&mut self) {
        match self.side {
            Side::Attacker => self.side = Side::Defender,
            Side::Defender => self.side = Side::Attacker,
        }
    }
}

/// Moves a figures on a board.
pub fn move_figure(
    mut event: EventReader<MoveFigureEvent>,
    mut q_board: Query<(&mut Board, &mut TurnTracker)>,
    mut q_figure: Query<(&mut Figure, &mut Transform)>,
    mut capture_check_event: EventWriter<CaptureCheckEvent>,
    mut king_on_corner_check_event: EventWriter<KingOnCornerCheckEvent>,
) {
    for ev in event.read() {
        let (mut board, mut turn_tracker) = q_board.get_mut(ev.board_entity).unwrap();
        let Some(figure_entity) = board.figures.get(&ev.from) else {
            // no figure to move
            panic!("no figure on from position");
        };

        let (mut figure, mut figure_transform) = q_figure.get_mut(*figure_entity).unwrap();

        if figure.side != turn_tracker.side {
            panic!("figure's side should be matching the turn");
        }

        let possible_moves = possible_moves(&board, *figure);

        if possible_moves.contains(&ev.to) {
            // update board
            if let Some(val) = board.figures.remove(&ev.from) {
                board.figures.insert(ev.to, val);
            }

            // update figure
            figure.position = ev.to;
            figure_transform.translation =
                board.board_to_world(figure.position).extend(board.figure_z);

            if figure.side == Side::Defender && figure.kind == FigureKind::King {
                king_on_corner_check_event.send(KingOnCornerCheckEvent {
                    board_entity: ev.board_entity,
                });
            }

            for neighbor in board.get_neighbors(ev.to) {
                capture_check_event.send(CaptureCheckEvent {
                    board_entity: ev.board_entity,
                    figure_entity: neighbor,
                });
            }

            // update game state
            turn_tracker.next_turn();
        }
    }
}

/// Returns the possible moves for a figure on a board.
/// Pre:
/// - figure is on the board
pub fn possible_moves(board: &Board, figure: Figure) -> Vec<Position> {
    let position = figure.position;
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
    let position = figure.position;
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
