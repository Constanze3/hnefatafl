use crate::game::tafl::*;

// Note: Surrounding the King is a win_condition.

#[derive(Event)]
/// Event for "actually" capturing figures.
/// `CaptureCheckEvent` should be used for performing a check instead.
pub struct CaptureEvent {
    pub board_entity: Entity,
    pub figure_entity: Entity,
}

pub fn capture(
    mut event: EventReader<CaptureEvent>,
    mut q_board: Query<&mut Board>,
    q_figure: Query<&Figure>,
    mut commands: Commands,
) {
    for ev in event.read() {
        let mut board = q_board.get_mut(ev.board_entity).unwrap();
        let figure = q_figure.get(ev.figure_entity).unwrap();

        board.figures.remove(&figure.position);
        commands.entity(ev.figure_entity).despawn();
    }
}

#[derive(Event)]
pub struct CaptureCheckEvent {
    pub board_entity: Entity,
    pub figure_entities: Vec<Entity>,
}

fn capture_check(mut event: EventReader<CaptureCheckEvent>) {}

#[derive(Event)]
pub struct ShieldwallCaptureCheckEvent {
    pub board_entity: Entity,
    pub figure_entity: Entity,
}

/// Checks for "Shieldwall Captures". A shieldwall capture is a type of capture where (possibly
/// multiple) surrounded units at the edge of the board are captured.
fn shieldwall_capture_check(mut event: EventReader<ShieldwallCaptureCheckEvent>) {}

//
// /// FigureKind::Soldier is captured if it surrounded on 2 opposite sides
// /// For FigureKind::Kind if it is surrounded on all 4 sides
// /// (by different side figures/walls/end positions)
// fn capture_check_figures(
//     q_figure: Query<&Figure>,
//     mut q_board: Query<&mut Board>,
//     mut commands: Commands,
//     mut event: EventReader<CaptureCheckEvent>,
//     mut shieldwall_capture_event: EventWriter<ShieldwallCaptureEvent>,
// ) {
//     let mut board = q_board.get_single_mut().unwrap();
//
//     for ev in event.read() {
//         for figure_entity in &ev.0 {
//             let figure = q_figure.get(*figure_entity).unwrap();
//             let position = figure.board_position;
//
//             if !board.figures.contains_key(&position) {
//                 panic!("figure isn't on the board");
//             }
//
//             let is_blocked = |position: &BoardPosition| -> bool {
//                 let blocking_figure_entity = board.figures.get(position);
//
//                 // contains enemy
//                 if let Some(bfe) = blocking_figure_entity {
//                     let blocking_figure = q_figure.get(*bfe).unwrap();
//                     if blocking_figure.side != figure.side {
//                         return true;
//                     }
//                 }
//
//                 // is end position
//                 if board.end_positions.contains(position) {
//                     return true;
//                 }
//
//                 // is empty throne
//                 if board.throne_position == *position && blocking_figure_entity == None {
//                     return true;
//                 }
//
//                 false
//             };
//
//             let left_blocked = 0 <= position.x as isize - 1
//                 && is_blocked(&BoardPosition {
//                     x: position.x - 1,
//                     y: position.y,
//                 });
//
//             let right_blocked = position.x + 1 < board.cols
//                 && is_blocked(&BoardPosition {
//                     x: position.x + 1,
//                     y: position.y,
//                 });
//
//             let bottom_blocked = 0 <= position.y as isize - 1
//                 && is_blocked(&BoardPosition {
//                     x: position.x,
//                     y: position.y - 1,
//                 });
//
//             let top_blocked = position.y + 1 < board.rows
//                 && is_blocked(&BoardPosition {
//                     x: position.x,
//                     y: position.y + 1,
//                 });
//
//             let x_blocked = left_blocked && right_blocked;
//             let y_blocked = bottom_blocked && top_blocked;
//
//             let should_capture = match figure.kind {
//                 FigureKind::Soldier => x_blocked || y_blocked,
//                 FigureKind::King => x_blocked && y_blocked,
//             };
//
//             if should_capture {
//                 board.figures.remove(&figure.board_position);
//                 commands.entity(*figure_entity).despawn();
//             } else {
//                 shieldwall_capture_event.send(ShieldwallCaptureEvent(*figure_entity));
//             }
//         }
//     }
// }
//
// #[derive(Event, Clone)]
// struct ShieldwallCaptureEvent(Entity);
//
// /// Preconditions:
// /// - checked figures can't be on a corner square
// /// - the board has minimum size 2 x 2
// fn shieldwall_capture(
//     mut event: EventReader<ShieldwallCaptureEvent>,
//     mut q_board: Query<&mut Board>,
//     q_figure: Query<&Figure>,
//     mut commands: Commands,
// ) {
//     enum EdgeSide {
//         Left,
//         Right,
//         Top,
//         Bottom,
//     }
//
//     /// Determines whether the provided position is at the edge of the board and if yes then on
//     /// which side of the board the edge is on.
//     ///
//     /// Precondition:
//     /// - figures can't be on a corner square so they can only have one associated side at a time
//     fn determine_edge_side(board: &Board, position: BoardPosition) -> Option<EdgeSide> {
//         if position.x == 0 {
//             return Some(EdgeSide::Left);
//         }
//
//         if position.x == board.cols - 1 {
//             return Some(EdgeSide::Right);
//         }
//
//         if position.y == 0 {
//             return Some(EdgeSide::Top);
//         }
//
//         if position.y == board.rows - 1 {
//             return Some(EdgeSide::Bottom);
//         }
//
//         return None;
//     }
//
//     /// Determines the vector of entities that are in a shieldwall capture.
//     /// The checking starts from the `initial_entity` so if the vector isn't empty it includes the
//     /// provided `initial_entity`.
//     fn shieldwall_check(
//         initial_entity: Entity,
//         board: &Board,
//         q_figure: &Query<&Figure>,
//     ) -> Vec<Entity> {
//         let initial_figure = q_figure.get(initial_entity).unwrap();
//
//         let Some(axis) = determine_edge_side(board, initial_figure.board_position) else {
//             return Vec::new();
//         };
//
//         let mut to_check: VecDeque<Entity> = VecDeque::new();
//         let mut result: Vec<Entity> = Vec::new();
//
//         to_check.push_front(initial_entity);
//
//         /// Handles the non-axis neighbor position of the checked figure.
//         /// Returns:
//         /// true - if the checking should continue
//         /// false - if the evaluated figures are not in captured by a shieldwall
//         fn handle_non_axis_neighbor_position(
//             neighbor_position: BoardPosition,
//             board: &Board,
//             q_figure: &Query<&Figure>,
//             figure: &Figure,
//         ) -> bool {
//             if let Some(neighbor_entity) = board.figures.get(&neighbor_position) {
//                 let neighbor_figure = q_figure.get(*neighbor_entity).unwrap();
//
//                 if figure.side == neighbor_figure.side {
//                     return false;
//                 }
//             } else {
//                 return false;
//             }
//
//             true
//         }
//
//         /// Handles an axis neighbor position of the checked figure.
//         /// Returns:
//         /// true - if the checking should continue
//         /// false - if the evaluated figures are not in captured by a shieldwall
//         fn handle_axis_neighbor_position(
//             neighbor_position: BoardPosition,
//             board: &Board,
//             q_figure: &Query<&Figure>,
//             figure: &Figure,
//             result: &Vec<Entity>,
//             to_check: &mut VecDeque<Entity>,
//         ) -> bool {
//             if let Some(neighbor_entity) = board.figures.get(&neighbor_position) {
//                 let neighbor_figure = q_figure.get(*neighbor_entity).unwrap();
//
//                 // if there is a neighbor on the axis with with the same "color"
//                 // it should also be checked
//                 if figure.side == neighbor_figure.side && !result.contains(neighbor_entity) {
//                     to_check.push_front(*neighbor_entity);
//                 }
//
//                 return true;
//             }
//
//             if board.end_positions.contains(&neighbor_position) {
//                 return true;
//             }
//
//             // there is a regular field at either end of the "row" that is empty
//             return false;
//         }
//
//         /// Handles the x-axis neighbor positions of the checked figure.
//         /// Returns:
//         /// true - if the checking should continue
//         /// false - if the evaluated figures are not in captured by a shieldwall
//         fn handle_x_axis_neighbor_positions(
//             position: BoardPosition,
//             board: &Board,
//             q_figure: &Query<&Figure>,
//             figure: &Figure,
//             result: &Vec<Entity>,
//             to_check: &mut VecDeque<Entity>,
//         ) -> bool {
//             if 0 <= position.x as isize - 1 {
//                 let should_continue = handle_axis_neighbor_position(
//                     BoardPosition {
//                         x: position.x - 1,
//                         y: position.y,
//                     },
//                     board,
//                     q_figure,
//                     figure,
//                     result,
//                     to_check,
//                 );
//
//                 if !should_continue {
//                     return false;
//                 }
//             }
//
//             if position.x + 1 < board.cols {
//                 let should_continue = handle_axis_neighbor_position(
//                     BoardPosition {
//                         x: position.x + 1,
//                         y: position.y,
//                     },
//                     board,
//                     q_figure,
//                     figure,
//                     result,
//                     to_check,
//                 );
//
//                 if !should_continue {
//                     return false;
//                 }
//             }
//
//             return true;
//         }
//
//         /// Handles the y-axis neighbor positions of the checked figure.
//         /// Returns:
//         /// true - if the checking should continue
//         /// false - if the evaluated figures are not in captured by a shieldwall
//         fn handle_y_axis_neighbor_positions(
//             position: BoardPosition,
//             board: &Board,
//             q_figure: &Query<&Figure>,
//             figure: &Figure,
//             result: &Vec<Entity>,
//             to_check: &mut VecDeque<Entity>,
//         ) -> bool {
//             if 0 <= position.y as isize - 1 {
//                 let should_continue = handle_axis_neighbor_position(
//                     BoardPosition {
//                         x: position.x,
//                         y: position.y - 1,
//                     },
//                     board,
//                     q_figure,
//                     figure,
//                     result,
//                     to_check,
//                 );
//
//                 if !should_continue {
//                     return false;
//                 }
//             }
//
//             if position.y + 1 < board.rows {
//                 let should_continue = handle_axis_neighbor_position(
//                     BoardPosition {
//                         x: position.x,
//                         y: position.y + 1,
//                     },
//                     board,
//                     q_figure,
//                     figure,
//                     result,
//                     to_check,
//                 );
//
//                 if !should_continue {
//                     return false;
//                 }
//             }
//
//             return true;
//         }
//
//         while 0 < to_check.len() {
//             let entity = to_check.pop_back().unwrap();
//             let figure = q_figure.get(entity).unwrap();
//             let position = figure.board_position;
//
//             match axis {
//                 EdgeSide::Left => {
//                     if position.x + 1 < board.cols {
//                         let non_axis_neighbor_position = BoardPosition {
//                             x: position.x + 1,
//                             y: position.y,
//                         };
//
//                         if !handle_non_axis_neighbor_position(
//                             non_axis_neighbor_position,
//                             board,
//                             q_figure,
//                             figure,
//                         ) {
//                             return Vec::new();
//                         }
//                     } else {
//                         panic!("board should be at least size 2x2");
//                     }
//
//                     if !handle_y_axis_neighbor_positions(
//                         position,
//                         board,
//                         q_figure,
//                         figure,
//                         &result,
//                         &mut to_check,
//                     ) {
//                         return Vec::new();
//                     }
//                 }
//                 EdgeSide::Right => {
//                     if 0 <= position.x as isize - 1 {
//                         let non_axis_neighbor_position = BoardPosition {
//                             x: position.x - 1,
//                             y: position.y,
//                         };
//
//                         if !handle_non_axis_neighbor_position(
//                             non_axis_neighbor_position,
//                             board,
//                             q_figure,
//                             figure,
//                         ) {
//                             return Vec::new();
//                         }
//                     } else {
//                         panic!("board should be at least size 2x2");
//                     }
//
//                     if !handle_y_axis_neighbor_positions(
//                         position,
//                         board,
//                         q_figure,
//                         figure,
//                         &result,
//                         &mut to_check,
//                     ) {
//                         return Vec::new();
//                     }
//                 }
//                 EdgeSide::Top => {
//                     if position.y + 1 < board.rows {
//                         let non_axis_neighbor_position = BoardPosition {
//                             x: position.x,
//                             y: position.y + 1,
//                         };
//
//                         if !handle_non_axis_neighbor_position(
//                             non_axis_neighbor_position,
//                             board,
//                             q_figure,
//                             figure,
//                         ) {
//                             return Vec::new();
//                         }
//                     } else {
//                         panic!("board should be at least size 2x2");
//                     }
//
//                     if !handle_x_axis_neighbor_positions(
//                         position,
//                         board,
//                         q_figure,
//                         figure,
//                         &result,
//                         &mut to_check,
//                     ) {
//                         return Vec::new();
//                     }
//                 }
//                 EdgeSide::Bottom => {
//                     if 0 <= position.y as isize - 1 {
//                         let non_axis_neighbor_position = BoardPosition {
//                             x: position.x,
//                             y: position.y - 1,
//                         };
//
//                         if !handle_non_axis_neighbor_position(
//                             non_axis_neighbor_position,
//                             board,
//                             q_figure,
//                             figure,
//                         ) {
//                             return Vec::new();
//                         }
//                     } else {
//                         panic!("board should be at least size 2x2");
//                     }
//
//                     if !handle_x_axis_neighbor_positions(
//                         position,
//                         board,
//                         q_figure,
//                         figure,
//                         &result,
//                         &mut to_check,
//                     ) {
//                         return Vec::new();
//                     }
//                 }
//             }
//
//             result.push(entity);
//         }
//
//         return result;
//     }
//
//     let mut board = q_board.get_single_mut().unwrap();
//     for ev in event.read() {
//         let initial_entity = ev.0;
//         let to_capture = shieldwall_check(initial_entity, &board, &q_figure);
//
//         for entity in to_capture {
//             // TODO move actual capturing to separate system
//
//             let figure = q_figure.get(entity).unwrap();
//
//             // King can't be captured with a shieldwall capture
//             if figure.kind != FigureKind::King {
//                 board.figures.remove(&figure.board_position);
//                 commands.entity(entity).despawn();
//             }
//         }
//     }
// }
