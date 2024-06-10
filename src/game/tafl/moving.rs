use crate::game::tafl::*;

use self::win_conditions::KingOnCornerCheckEvent;

#[derive(Resource)]
pub struct MoveFigureOptions {
    pub slide_duration: f32,
}

impl Default for MoveFigureOptions {
    fn default() -> Self {
        Self {
            slide_duration: 0.2,
        }
    }
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

#[derive(Event, Clone, Copy)]
pub struct MoveFigureEvent {
    board_entity: Entity,
    from: Position,
    to: Position,
}

// Pre: The move is valid.
pub fn move_figure(
    mut event: EventReader<MoveFigureEvent>,
    mut q_board: Query<&mut Board>,
    mut q_figure: Query<(&mut Figure, &mut Transform)>,
    mut king_on_corner_check_event: EventWriter<KingOnCornerCheckEvent>,
    mut end_move_event: EventWriter<EndMoveEvent>,
    mut capture_checks_event: EventWriter<CaptureChecksEvent>,
) {
    for ev in event.read() {
        let mut board = q_board.get_mut(ev.board_entity).unwrap();

        let Some(figure_entity) = board.figures.get(&ev.from) else {
            panic!("`from` should contain a figure");
        };
        let (mut figure, mut figure_transform) = q_figure.get_mut(*figure_entity).unwrap();

        if let Some(val) = board.figures.remove(&figure.position) {
            board.figures.insert(ev.to, val);
        }
        figure.position = ev.to;

        figure_transform.translation = board.board_to_world(figure.position).extend(board.figure_z);

        if figure.side == Side::Defender && figure.kind == FigureKind::King {
            king_on_corner_check_event.send(KingOnCornerCheckEvent {
                board_entity: ev.board_entity,
            });
        }

        let neighbors = board.get_neighbors(ev.to);

        if neighbors.is_empty() {
            end_move_event.send(EndMoveEvent {
                board_entity: ev.board_entity,
                capture_happened: false,
            });
        }

        capture_checks_event.send(CaptureChecksEvent {
            board_entity: ev.board_entity,
            figure_entities: neighbors,
        });
    }
}

#[derive(Event)]
pub struct EndMoveEvent {
    pub board_entity: Entity,
    pub capture_happened: bool,
}

pub fn end_move(
    mut event: EventReader<EndMoveEvent>,
    mut q_board: Query<&mut TurnTracker, With<Board>>,
    mut indicate_turn_event: EventWriter<IndicateTurnEvent>,
) {
    for ev in event.read() {
        let mut turn_tracker = q_board.get_mut(ev.board_entity).unwrap();
        turn_tracker.next_turn();
        indicate_turn_event.send(IndicateTurnEvent {
            side: Some(turn_tracker.side),
        });
    }
}

#[derive(Component)]
pub struct FigureToSlideAndMove {
    event: MoveFigureEvent,
    interpolation: f32,
}

impl FigureToSlideAndMove {
    pub fn new(event: MoveFigureEvent) -> Self {
        Self {
            event,
            interpolation: 0.,
        }
    }
}

pub fn slide_and_move_figure(
    move_figure_options: Res<MoveFigureOptions>,
    q_board: Query<&Board>,
    mut q_figure: Query<(Entity, &mut Transform, &mut FigureToSlideAndMove), With<Figure>>,
    time: Res<Time>,
    mut commands: Commands,
    mut move_figure_event: EventWriter<MoveFigureEvent>,
    mut selection_options: ResMut<SelectionOptions>,
) {
    for (figure_entity, mut figure_transform, mut figure_to_slide_and_move) in &mut q_figure {
        let FigureToSlideAndMove {
            event,
            interpolation,
        } = figure_to_slide_and_move.as_mut();

        let board = q_board.get(event.board_entity).unwrap();

        let from_world_position = board.board_to_world(event.from);
        let to_world_position = board.board_to_world(event.to);

        *interpolation += time.delta_seconds() / move_figure_options.slide_duration;
        *interpolation = f32::min(1., *interpolation);

        if 1.0 <= *interpolation {
            commands
                .entity(figure_entity)
                .remove::<FigureToSlideAndMove>();
            selection_options.selection_locked = false;

            move_figure_event.send(*event);
            return;
        }

        let interpolation_vector = (to_world_position - from_world_position) * *interpolation;
        let interpolated_position = from_world_position + interpolation_vector;

        let z = figure_transform.translation.z;
        figure_transform.translation = interpolated_position.extend(z);
    }
}

#[derive(Event)]
pub struct TryMoveFigureEvent {
    pub board_entity: Entity,
    pub from: Position,
    pub to: Position,
    pub slide: bool,
}

/// Validates moves and moves figures on a board.
pub fn try_move_figure(
    mut event: EventReader<TryMoveFigureEvent>,
    mut q_board: Query<(&mut Board, &mut TurnTracker)>,
    mut q_figure: Query<&mut Figure>,
    mut release_selected_figure_event: EventWriter<ReleaseSelectedFigureEvent>,
    mut move_figure_event: EventWriter<MoveFigureEvent>,
    mut commands: Commands,
    mut selection_options: ResMut<SelectionOptions>,
) {
    for ev in event.read() {
        let (board, turn_tracker) = q_board.get_mut(ev.board_entity).unwrap();
        let Some(figure_entity) = board.figures.get(&ev.from) else {
            // no figure to move
            panic!("no figure on from position");
        };

        let figure = q_figure.get_mut(*figure_entity).unwrap();

        if figure.side != turn_tracker.side {
            panic!("figure's side should be matching the turn");
        }

        let event = MoveFigureEvent {
            board_entity: ev.board_entity,
            from: ev.from,
            to: ev.to,
        };

        if possible_moves(&board, *figure).contains(&ev.to) {
            release_selected_figure_event.send(ReleaseSelectedFigureEvent {
                board_entity: ev.board_entity,
            });

            if ev.slide {
                commands
                    .entity(*figure_entity)
                    .insert(FigureToSlideAndMove::new(event));
                selection_options.selection_locked = true;
            } else {
                move_figure_event.send(event);
            }
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
