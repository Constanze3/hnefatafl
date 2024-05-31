use std::collections::VecDeque;

use bevy::{
    math::Vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::{close_on_esc, PrimaryWindow},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SelectedFigure>()
        .init_resource::<MousePosition>()
        .insert_resource(board_data)
        .insert_resource(figure_data)
        .insert_resource(clear_color)
        .add_systems(Startup, (setup, (spawn_board, setup_board).chain()))
        .add_systems(
            Update,
            (
                update_mouse_position,
                (
                    close_on_esc,
                    visual_move_figure,
                    (
                        select_figure,
                        spawn_highlights,
                        move_figure,
                        despawn_highlights,
                    )
                        .chain(),
                    (capture_check_figures, shieldwall_capture)
                        .chain()
                        .after(move_figure),
                )
                    .after(update_mouse_position),
            ),
        )
        .add_event::<CreateBoardEvent>()
        .add_event::<SpawnHighlightsEvent>()
        .add_event::<DespawnHighlightsEvent>()
        .add_event::<CaptureCheckEvent>()
        .add_event::<ShieldwallCaptureEvent>()
        .run();
}

// SYSTEMS

fn select_figure(
    q_figures: Query<(Entity, &Figure)>,
    q_board: Query<(Entity, &Board)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut selected_figure: ResMut<SelectedFigure>,
    mut highlights_event: EventWriter<SpawnHighlightsEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let Some(mouse_position) = mouse_position.0 else {
            return;
        };

        let (board_entity, board) = q_board.get_single().unwrap();
        let Some(selected_field) = board.world_to_board(mouse_position) else {
            return;
        };

        *selected_figure = {
            let mut result = SelectedFigure::None;
            for (figure_entity, figure) in &q_figures {
                if figure.board_position == selected_field {
                    let possible_moves = board.possible_moves(*figure).unwrap();

                    result = SelectedFigure::Some(SelectedFigure_ {
                        entity: figure_entity,
                        possible_moves: possible_moves.clone(),
                    });

                    highlights_event.send(SpawnHighlightsEvent {
                        board_entity,
                        positions: possible_moves.clone(),
                    });

                    break;
                }
            }

            result
        };
    }
}

fn visual_move_figure(
    mut q_figure_transforms: Query<&mut Transform, With<Figure>>,
    mouse_position: Res<MousePosition>,
    selected_figure: Res<SelectedFigure>,
) {
    if let SelectedFigure::Some(SelectedFigure_ {
        entity: figure_entity,
        possible_moves: _,
    }) = *selected_figure
    {
        let Some(mouse_position) = mouse_position.0 else {
            return;
        };

        let mut figure_transform = q_figure_transforms.get_mut(figure_entity).unwrap();
        figure_transform.translation = mouse_position.clone().extend(5.);
    };
}

fn move_figure(
    mut q_figure: Query<(&mut Figure, &mut Transform)>,
    mut q_board: Query<(Entity, &mut Board)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut selected_figure: ResMut<SelectedFigure>,
    mut highlights_event: EventWriter<DespawnHighlightsEvent>,
    mut capture_event: EventWriter<CaptureCheckEvent>,
) {
    if buttons.just_released(MouseButton::Left) {
        let SelectedFigure::Some(SelectedFigure_ {
            entity: figure_entity,
            possible_moves,
        }) = selected_figure.clone()
        else {
            return;
        };

        let (board_entity, mut board) = q_board.get_single_mut().unwrap();

        _ = 'perform_move: {
            let Some(mouse_position) = mouse_position.0 else {
                break 'perform_move;
            };

            let Some(targeted_field) = board.world_to_board(mouse_position) else {
                break 'perform_move;
            };

            let at_target = board.figures.get(&targeted_field);
            if at_target != None {
                // if there is something at the target the figure can't be moved there
                break 'perform_move;
            }

            if !possible_moves.contains(&targeted_field) {
                break 'perform_move;
            }

            let (mut figure, _) = q_figure.get_mut(figure_entity).unwrap();

            // update board.figures
            if let Some(val) = board.figures.remove(&figure.board_position) {
                board.figures.insert(targeted_field, val);
            }

            // update figure
            figure.board_position = targeted_field;

            // check for captures
            let neighbor_entities = board.get_neighbors(figure.board_position);
            capture_event.send(CaptureCheckEvent(neighbor_entities));
        };

        let (figure, mut figure_transform) = q_figure.get_mut(figure_entity).unwrap();
        figure_transform.translation = board
            .board_to_world(figure.board_position)
            .extend(board.figure_display_z);

        *selected_figure = SelectedFigure::None;
        highlights_event.send(DespawnHighlightsEvent { board_entity });
    }
}

#[derive(Event, Clone)]
struct CaptureCheckEvent(Vec<Entity>);

/// FigureKind::Soldier is captured if it surrounded on 2 opposite sides
/// For FigureKind::Kind if it is surrounded on all 4 sides
/// (by different side figures/walls/end positions)
fn capture_check_figures(
    q_figure: Query<&Figure>,
    mut q_board: Query<&mut Board>,
    mut commands: Commands,
    mut event: EventReader<CaptureCheckEvent>,
    mut shieldwall_capture_event: EventWriter<ShieldwallCaptureEvent>,
) {
    let mut board = q_board.get_single_mut().unwrap();

    for ev in event.read() {
        for figure_entity in &ev.0 {
            let figure = q_figure.get(*figure_entity).unwrap();
            let position = figure.board_position;

            if !board.figures.contains_key(&position) {
                panic!("figure isn't on the board");
            }

            let is_blocked = |position: &BoardPosition| -> bool {
                let blocking_figure_entity = board.figures.get(position);

                // contains enemy
                if let Some(bfe) = blocking_figure_entity {
                    let blocking_figure = q_figure.get(*bfe).unwrap();
                    if blocking_figure.side != figure.side {
                        return true;
                    }
                }

                // is end position
                if board.end_positions.contains(position) {
                    return true;
                }

                // is empty throne
                if board.throne_position == *position && blocking_figure_entity == None {
                    return true;
                }

                false
            };

            let left_blocked = 0 <= position.x as isize - 1
                && is_blocked(&BoardPosition {
                    x: position.x - 1,
                    y: position.y,
                });

            let right_blocked = position.x + 1 < board.cols
                && is_blocked(&BoardPosition {
                    x: position.x + 1,
                    y: position.y,
                });

            let bottom_blocked = 0 <= position.y as isize - 1
                && is_blocked(&BoardPosition {
                    x: position.x,
                    y: position.y - 1,
                });

            let top_blocked = position.y + 1 < board.rows
                && is_blocked(&BoardPosition {
                    x: position.x,
                    y: position.y + 1,
                });

            let x_blocked = left_blocked && right_blocked;
            let y_blocked = bottom_blocked && top_blocked;

            let should_capture = match figure.kind {
                FigureKind::Soldier => x_blocked || y_blocked,
                FigureKind::King => x_blocked && y_blocked,
            };

            if should_capture {
                board.figures.remove(&figure.board_position);
                commands.entity(*figure_entity).despawn();
            } else {
                shieldwall_capture_event.send(ShieldwallCaptureEvent(*figure_entity));
            }
        }
    }
}

#[derive(Event, Clone)]
struct ShieldwallCaptureEvent(Entity);

/// Preconditions:
/// - checked figures can't be on a corner square
/// - the board has minimum size 2 x 2
fn shieldwall_capture(
    mut event: EventReader<ShieldwallCaptureEvent>,
    mut q_board: Query<&mut Board>,
    q_figure: Query<&Figure>,
    mut commands: Commands,
) {
    enum EdgeSide {
        Left,
        Right,
        Top,
        Bottom,
    }

    /// Determines whether the provided position is at the edge of the board and if yes then on
    /// which side of the board the edge is on.
    ///
    /// Precondition:
    /// - figures can't be on a corner square so they can only have one associated side at a time
    fn determine_edge_side(board: &Board, position: BoardPosition) -> Option<EdgeSide> {
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
    /// The checking starts from the `initial_entity` so if the vector isn't empty it includes the
    /// provided `initial_entity`.
    fn shieldwall_check(
        initial_entity: Entity,
        board: &Board,
        q_figure: &Query<&Figure>,
    ) -> Vec<Entity> {
        let initial_figure = q_figure.get(initial_entity).unwrap();

        let Some(axis) = determine_edge_side(board, initial_figure.board_position) else {
            return Vec::new();
        };

        let mut to_check: VecDeque<Entity> = VecDeque::new();
        let mut result: Vec<Entity> = Vec::new();

        to_check.push_front(initial_entity);

        /// Handles the non-axis neighbor position of the checked figure.
        /// Returns:
        /// true - if the checking should continue
        /// false - if the evaluated figures are not in captured by a shieldwall
        fn handle_non_axis_neighbor_position(
            neighbor_position: BoardPosition,
            board: &Board,
            q_figure: &Query<&Figure>,
            figure: &Figure,
        ) -> bool {
            if let Some(neighbor_entity) = board.figures.get(&neighbor_position) {
                let neighbor_figure = q_figure.get(*neighbor_entity).unwrap();

                if figure.side == neighbor_figure.side {
                    return false;
                }
            } else {
                return false;
            }

            true
        }

        /// Handles an axis neighbor position of the checked figure.
        /// Returns:
        /// true - if the checking should continue
        /// false - if the evaluated figures are not in captured by a shieldwall
        fn handle_axis_neighbor_position(
            neighbor_position: BoardPosition,
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
            position: BoardPosition,
            board: &Board,
            q_figure: &Query<&Figure>,
            figure: &Figure,
            result: &Vec<Entity>,
            to_check: &mut VecDeque<Entity>,
        ) -> bool {
            if 0 <= position.x as isize - 1 {
                let should_continue = handle_axis_neighbor_position(
                    BoardPosition {
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
                    BoardPosition {
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
            position: BoardPosition,
            board: &Board,
            q_figure: &Query<&Figure>,
            figure: &Figure,
            result: &Vec<Entity>,
            to_check: &mut VecDeque<Entity>,
        ) -> bool {
            if 0 <= position.y as isize - 1 {
                let should_continue = handle_axis_neighbor_position(
                    BoardPosition {
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
                    BoardPosition {
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

        while 0 < to_check.len() {
            let entity = to_check.pop_back().unwrap();
            let figure = q_figure.get(entity).unwrap();
            let position = figure.board_position;

            match axis {
                EdgeSide::Left => {
                    if position.x + 1 < board.cols {
                        let non_axis_neighbor_position = BoardPosition {
                            x: position.x + 1,
                            y: position.y,
                        };

                        if !handle_non_axis_neighbor_position(
                            non_axis_neighbor_position,
                            board,
                            q_figure,
                            figure,
                        ) {
                            return Vec::new();
                        }
                    } else {
                        panic!("board should be at least size 2x2");
                    }

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
                    if 0 <= position.x as isize - 1 {
                        let non_axis_neighbor_position = BoardPosition {
                            x: position.x - 1,
                            y: position.y,
                        };

                        if !handle_non_axis_neighbor_position(
                            non_axis_neighbor_position,
                            board,
                            q_figure,
                            figure,
                        ) {
                            return Vec::new();
                        }
                    } else {
                        panic!("board should be at least size 2x2");
                    }

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
                    if position.y + 1 < board.rows {
                        let non_axis_neighbor_position = BoardPosition {
                            x: position.x,
                            y: position.y + 1,
                        };

                        if !handle_non_axis_neighbor_position(
                            non_axis_neighbor_position,
                            board,
                            q_figure,
                            figure,
                        ) {
                            return Vec::new();
                        }
                    } else {
                        panic!("board should be at least size 2x2");
                    }

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
                    if 0 <= position.y as isize - 1 {
                        let non_axis_neighbor_position = BoardPosition {
                            x: position.x,
                            y: position.y - 1,
                        };

                        if !handle_non_axis_neighbor_position(
                            non_axis_neighbor_position,
                            board,
                            q_figure,
                            figure,
                        ) {
                            return Vec::new();
                        }
                    } else {
                        panic!("board should be at least size 2x2");
                    }

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

    let mut board = q_board.get_single_mut().unwrap();
    for ev in event.read() {
        let initial_entity = ev.0;
        let to_capture = shieldwall_check(initial_entity, &board, &q_figure);

        for entity in to_capture {
            // TODO move actual capturing to separate system

            let figure = q_figure.get(entity).unwrap();

            // King can't be captured with a shieldwall capture
            if figure.kind != FigureKind::King {
                board.figures.remove(&figure.board_position);
                commands.entity(entity).despawn();
            }
        }
    }
}
