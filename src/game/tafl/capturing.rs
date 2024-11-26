use crate::game::tafl::*;

// Note: Surrounding the King is a win_condition.

/// Captures a figure.
pub fn capture(
    q_board: &mut Query<&mut Board>,
    q_figure: &Query<&Figure>,
    board_entity: Entity,
    figure_entity: Entity,
    commands: &mut Commands,
) {
    let mut board = q_board.get_mut(board_entity).unwrap();
    let figure = q_figure.get(figure_entity).unwrap();

    board.figures.remove(&figure.position);
    commands.entity(figure_entity).despawn();
}

/// Determines whether a "position" on the board is blocked for a "figure".
///
/// A blocked position means it either:
/// - contains an enemy
/// - is an end position (one of the 4 corners of the board)
/// - is the empty throne (the center position of the board)
fn is_blocked(
    q_board: &Query<&Board>,
    q_figure: &Query<&Figure>,
    board_entity: Entity,
    figure_entity: Entity,
    position: Position,
) -> bool {
    let board = q_board.get(board_entity).unwrap();
    let figure = q_figure.get(figure_entity).unwrap();

    let other = board.figures.get(&position);

    // contains enemy
    if let Some(other_entity) = other {
        let other_figure = q_figure.get(*other_entity).unwrap();

        if other_figure.side != figure.side {
            return true;
        }
    }

    // is end position
    if board.end_positions.contains(&position) {
        return true;
    }

    // is empty throne
    if board.throne_position == position && other == None {
        return true;
    }

    false
}

#[derive(Event)]
pub struct CaptureCheckEvent {
    pub board_entity: Entity,
    pub moved_figure_entity: Entity,
}

pub fn capture_check(
    mut event: EventReader<CaptureCheckEvent>,
    mut q_board: Query<&mut Board>,
    q_figure: Query<&Figure>,
    mut commands: Commands,
    mut king_surrounded_check_event: EventWriter<KingSurroundedCheckEvent>,
    mut end_move_event: EventWriter<EndMoveEvent>,
) {
    for ev in event.read() {
        let board_entity = ev.board_entity;
        let moved_figure_entity = ev.moved_figure_entity;

        let mut capture_happened = false;

        let board = q_board.get(board_entity).unwrap();
        let moved_figure = q_figure.get(moved_figure_entity).unwrap();

        let neighbors = board.get_neighbors(moved_figure.position);

        for figure_entity in neighbors {
            let figure = q_figure.get(figure_entity).unwrap();

            // only enemies can be captured
            if figure.side == moved_figure.side {
                continue;
            }

            // the king can't be captured, but may be part of a shieldwall capture
            //
            // every time the king is "part" of a capture it has to be checked
            // whether it is surrounded
            if figure.kind == FigureKind::King {
                capture_happened |= shieldwall_capture_check(
                    &mut q_board,
                    &q_figure,
                    ev.board_entity,
                    figure_entity,
                    &mut commands,
                );

                king_surrounded_check_event.send(KingSurroundedCheckEvent { board_entity });

                continue;
            }

            // if compared to the moved_figrure the opposite side of the figure is blocked than
            // it is captured

            // TODO implement addition/substraction trait for Position

            let mfx = moved_figure.position.x;
            let mfy = moved_figure.position.y;

            let fx = figure.position.x;
            let fy = figure.position.y;

            let diff_x = fx as i32 - mfx as i32;
            let diff_y = fy as i32 - mfy as i32;

            let x = fx as i32 + diff_x;
            let y = fy as i32 + diff_y;

            if x < 0 || y < 0 {
                continue;
            }

            let other_position = Position {
                x: x as usize,
                y: y as usize,
            };

            if is_blocked(
                &mut q_board.transmute_lens::<&Board>().query(),
                &q_figure,
                board_entity,
                figure_entity,
                other_position,
            ) {
                capture(
                    &mut q_board,
                    &q_figure,
                    board_entity,
                    figure_entity,
                    &mut commands,
                );

                capture_happened |= true;
            } else {
                capture_happened |= shieldwall_capture_check(
                    &mut q_board,
                    &q_figure,
                    board_entity,
                    figure_entity,
                    &mut commands,
                )
            }
        }

        end_move_event.send(EndMoveEvent {
            board_entity,
            capture_happened,
        });
    }
}

// pub fn capture_check(
//     mut event: EventReader<CaptureCheckEvent>,
//     q_figure: Query<&Figure>,
//     q_board: Query<&Board>,
//     mut capture_event: EventWriter<CaptureEvent>,
//     mut shieldwall_capture_check_event: EventWriter<ShieldwallCaptureCheckEvent>,
//     mut king_surrounded_check_event: EventWriter<KingSurroundedCheckEvent>,
//     mut on_capture_check_end_event: EventWriter<OnCaptureCheckEndEvent>,
// ) {
//     for ev in event.read() {
//         let board = q_board.get(ev.board_entity).unwrap();
//         let figure = q_figure.get(ev.figure_entity).unwrap();
//
//         // the king can't be captured, but may be part of a shieldwall capture
//         // we also send a check for the king surrounded win condition
//         if figure.kind == FigureKind::King {
//             shieldwall_capture_check_event.send(ShieldwallCaptureCheckEvent {
//                 board_entity: ev.board_entity,
//                 figure_entity: ev.figure_entity,
//             });
//
//             king_surrounded_check_event.send(KingSurroundedCheckEvent {
//                 board_entity: ev.board_entity,
//             });
//             return;
//         }
//
//         let position = figure.position;
//
//         if !board.figures.contains_key(&position) {
//             panic!("figure should be on the board");
//         }
//
//         // if a figure is blocked on two of its sides than it should be captured
//         let is_blocked = |position: &Position| -> bool {
//             let blocking_figure_entity = board.figures.get(position);
//
//             // contains enemy
//             if let Some(bfe) = blocking_figure_entity {
//                 let blocking_figure = q_figure.get(*bfe).unwrap();
//                 if blocking_figure.side != figure.side {
//                     return true;
//                 }
//             }
//
//             // is end position
//             if board.end_positions.contains(position) {
//                 return true;
//             }
//
//             // is empty throne
//             if board.throne_position == *position && blocking_figure_entity == None {
//                 return true;
//             }
//
//             false
//         };
//
//         let left_blocked = 1 <= position.x
//             && is_blocked(&Position {
//                 x: position.x - 1,
//                 y: position.y,
//             });
//
//         let right_blocked = position.x < board.cols - 1
//             && is_blocked(&Position {
//                 x: position.x + 1,
//                 y: position.y,
//             });
//
//         let bottom_blocked = 1 <= position.y
//             && is_blocked(&Position {
//                 x: position.x,
//                 y: position.y - 1,
//             });
//
//         let top_blocked = position.y < board.rows - 1
//             && is_blocked(&Position {
//                 x: position.x,
//                 y: position.y + 1,
//             });
//
//         let x_blocked = left_blocked && right_blocked;
//         let y_blocked = bottom_blocked && top_blocked;
//
//         let should_capture = x_blocked || y_blocked;
//
//         if should_capture {
//             capture_event.send(CaptureEvent {
//                 board_entity: ev.board_entity,
//                 figure_entity: ev.figure_entity,
//             });
//
//             on_capture_check_end_event.send(OnCaptureCheckEndEvent {
//                 board_entity: ev.board_entity,
//                 capture_happened: true,
//             });
//         } else {
//             // the figure may still be in a shieldwall capture
//             shieldwall_capture_check_event.send(ShieldwallCaptureCheckEvent {
//                 board_entity: ev.board_entity,
//                 figure_entity: ev.figure_entity,
//             });
//         }
//     }
// }

// #[derive(Event)]
// pub struct CaptureChecksEvent {
//     pub board_entity: Entity,
//     pub figure_entities: Vec<Entity>,
// }
//
// pub fn capture_checks(
//     mut event: EventReader<CaptureChecksEvent>,
//     mut q_board: Query<&mut OnCaptureCheckEndTracker, With<Board>>,
//     mut capture_check_event: EventWriter<CaptureCheckEvent>,
// ) {
//     for ev in event.read() {
//         let mut tracker = q_board.get_mut(ev.board_entity).unwrap();
//         tracker.todo_capture_count = ev.figure_entities.len() as u8;
//
//         for figure_entity in &ev.figure_entities {
//             capture_check_event.send(CaptureCheckEvent {
//                 board_entity: ev.board_entity,
//                 figure_entity: *figure_entity,
//             });
//         }
//     }
// }
//
// #[derive(Event)]
// /// Event for "actually" capturing figures.
// /// `CaptureCheckEvent` should be used for performing a check instead.
// pub struct CaptureEvent {
//     pub board_entity: Entity,
//     pub capturing_entity: Entity,
//     pub captured_entity: Entity,
// }
//
// pub fn capture(
//     mut event: EventReader<CaptureEvent>,
//     mut q_board: Query<&mut Board>,
//     q_figure: Query<&Figure>,
//     mut commands: Commands,
// ) {
//     for ev in event.read() {
//         let mut board = q_board.get_mut(ev.board_entity).unwrap();
//         let figure = q_figure.get(ev.figure_entity).unwrap();
//
//         board.figures.remove(&figure.position);
//         commands.entity(ev.figure_entity).despawn();
//     }
// }
//
// #[derive(Event)]
// pub struct OnCaptureCheckEndEvent {
//     pub board_entity: Entity,
//     pub capture_happened: bool,
// }
//
// #[derive(Component, Default)]
// pub struct OnCaptureCheckEndTracker {
//     pub todo_capture_count: u8,
//     pub ended_capture_check_count: u8,
//     pub capture_happened: bool,
// }
//
// pub fn collect_on_capture_check_end(
//     mut event: EventReader<OnCaptureCheckEndEvent>,
//     mut q_board: Query<&mut OnCaptureCheckEndTracker, With<Board>>,
//     mut end_move_event: EventWriter<EndMoveEvent>,
// ) {
//     for ev in event.read() {
//         let mut tracker = q_board.get_mut(ev.board_entity).unwrap();
//         tracker.ended_capture_check_count += 1;
//         tracker.capture_happened |= ev.capture_happened;
//
//         if tracker.ended_capture_check_count == tracker.todo_capture_count {
//             end_move_event.send(EndMoveEvent {
//                 board_entity: ev.board_entity,
//                 capture_happened: tracker.capture_happened,
//             });
//
//             *tracker = OnCaptureCheckEndTracker::default();
//         }
//     }
// }
//
// #[derive(Event)]
// pub struct CaptureCheckEvent {
//     pub board_entity: Entity,
//     pub figure_entity: Entity,
// }
//
// /// Checks whether figures on a board should be captured.
// pub fn capture_check(
//     mut event: EventReader<CaptureCheckEvent>,
//     q_figure: Query<&Figure>,
//     q_board: Query<&Board>,
//     mut capture_event: EventWriter<CaptureEvent>,
//     mut shieldwall_capture_check_event: EventWriter<ShieldwallCaptureCheckEvent>,
//     mut king_surrounded_check_event: EventWriter<KingSurroundedCheckEvent>,
//     mut on_capture_check_end_event: EventWriter<OnCaptureCheckEndEvent>,
// ) {
//     for ev in event.read() {
//         let board = q_board.get(ev.board_entity).unwrap();
//         let figure = q_figure.get(ev.figure_entity).unwrap();
//
//         // the king can't be captured, but may be part of a shieldwall capture
//         // we also send a check for the king surrounded win condition
//         if figure.kind == FigureKind::King {
//             shieldwall_capture_check_event.send(ShieldwallCaptureCheckEvent {
//                 board_entity: ev.board_entity,
//                 figure_entity: ev.figure_entity,
//             });
//
//             king_surrounded_check_event.send(KingSurroundedCheckEvent {
//                 board_entity: ev.board_entity,
//             });
//             return;
//         }
//
//         let position = figure.position;
//
//         if !board.figures.contains_key(&position) {
//             panic!("figure should be on the board");
//         }
//
//         // if a figure is blocked on two of its sides than it should be captured
//         let is_blocked = |position: &Position| -> bool {
//             let blocking_figure_entity = board.figures.get(position);
//
//             // contains enemy
//             if let Some(bfe) = blocking_figure_entity {
//                 let blocking_figure = q_figure.get(*bfe).unwrap();
//                 if blocking_figure.side != figure.side {
//                     return true;
//                 }
//             }
//
//             // is end position
//             if board.end_positions.contains(position) {
//                 return true;
//             }
//
//             // is empty throne
//             if board.throne_position == *position && blocking_figure_entity == None {
//                 return true;
//             }
//
//             false
//         };
//
//         let left_blocked = 1 <= position.x
//             && is_blocked(&Position {
//                 x: position.x - 1,
//                 y: position.y,
//             });
//
//         let right_blocked = position.x < board.cols - 1
//             && is_blocked(&Position {
//                 x: position.x + 1,
//                 y: position.y,
//             });
//
//         let bottom_blocked = 1 <= position.y
//             && is_blocked(&Position {
//                 x: position.x,
//                 y: position.y - 1,
//             });
//
//         let top_blocked = position.y < board.rows - 1
//             && is_blocked(&Position {
//                 x: position.x,
//                 y: position.y + 1,
//             });
//
//         let x_blocked = left_blocked && right_blocked;
//         let y_blocked = bottom_blocked && top_blocked;
//
//         let should_capture = x_blocked || y_blocked;
//
//         if should_capture {
//             capture_event.send(CaptureEvent {
//                 board_entity: ev.board_entity,
//                 figure_entity: ev.figure_entity,
//             });
//
//             on_capture_check_end_event.send(OnCaptureCheckEndEvent {
//                 board_entity: ev.board_entity,
//                 capture_happened: true,
//             });
//         } else {
//             // the figure may still be in a shieldwall capture
//             shieldwall_capture_check_event.send(ShieldwallCaptureCheckEvent {
//                 board_entity: ev.board_entity,
//                 figure_entity: ev.figure_entity,
//             });
//         }
//     }
// }
