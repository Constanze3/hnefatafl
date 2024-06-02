use crate::game::camera::*;
use crate::game::tafl::*;

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum SelectedFigure {
    Some {
        board_entity: Entity,
        figure_entity: Entity,
        was_put_down_once: bool,
    },
    #[default]
    None,
}

// Should be spawned in spawning and have hidden visibility
#[derive(Component)]
pub struct SelectionIndicator;

#[derive(Component)]
pub struct Grabbed {
    pub z: f32,
}

/// Selects the figure at the mouse position.
pub fn on_mouse_pressed(
    buttons: Res<ButtonInput<MouseButton>>,
    q_mouse_position: Query<&MousePositionTracker, With<MainCamera>>,
    mut selected_figure: ResMut<SelectedFigure>,
    q_board: Query<(Entity, &Board, &Transform)>,
    q_figure: Query<&Figure>,
    mut commands: Commands,
    mut spawn_highlights_event: EventWriter<SpawnHighlightsEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let Some(mouse_position) = q_mouse_position.get_single().unwrap().mouse_world_position
        else {
            return;
        };

        // get board closest to mouse
        let Some((board_entity, board)) = ({
            let mut result = None;
            let mut result_distance = 0.;

            for (board_entity, board, board_transform) in &q_board {
                let distance = board_transform.translation.xy().distance(mouse_position);

                if result.is_none() || distance < result_distance {
                    result_distance = distance;
                    result = Some((board_entity, board));
                    continue;
                }
            }

            result
        }) else {
            return;
        };

        let Some(selected_field) = board.world_to_board(mouse_position) else {
            return;
        };

        let Some(selected_figure_entity_reference) = board.figures.get(&selected_field) else {
            return;
        };
        let selected_figure_entity = *selected_figure_entity_reference;

        commands.entity(selected_figure_entity).insert(Grabbed {
            z: board.figure_z + 1.,
        });

        if let SelectedFigure::Some { figure_entity, .. } = *selected_figure {
            if figure_entity == selected_figure_entity {
                return;
            }
        }

        *selected_figure = SelectedFigure::Some {
            board_entity,
            figure_entity: selected_figure_entity,
            was_put_down_once: false,
        };

        let figure = q_figure.get(selected_figure_entity).unwrap();
        spawn_highlights_event.send(SpawnHighlightsEvent {
            board_entity,
            positions: possible_moves(board, *figure),
        });
    }
}

/// Snaps the grabbed entities to the mouse position.
pub fn drag_grabbed(
    mut q_grabbed_transform: Query<(&Grabbed, &mut Transform)>,
    q_mouse_position: Query<&MousePositionTracker, With<MainCamera>>,
) {
    let Some(mouse_position) = q_mouse_position.get_single().unwrap().mouse_world_position else {
        return;
    };

    for (grabbed, mut transform) in &mut q_grabbed_transform {
        transform.translation = mouse_position.clone().extend(grabbed.z);
    }
}

// pub fn handle_selection_indicator(
//     selected_figure: Res<SelectedFigure>,
//     q_selection_indicator: Query<(Transform, Visibility), With<SelectionIndicator>>,
// ) {
// }

/// Lets go of (removes Grabbed from) the figure, if it is done so for the first time the selection remains
/// allowing for `slide_and_move`, otherwise the selection is set to none as well.
pub fn on_mouse_released(
    buttons: Res<ButtonInput<MouseButton>>,
    q_mouse_position: Query<&MousePositionTracker, With<MainCamera>>,
    mut selected_figure: ResMut<SelectedFigure>,
    q_board: Query<&Board>,
    mut q_figure: Query<(&Figure, &mut Transform, Option<&Grabbed>)>,
    mut commands: Commands,
    mut despawn_highlights_event: EventWriter<DespawnHighlightsEvent>,
    mut move_figure_event: EventWriter<MoveFigureEvent>,
) {
    if buttons.just_released(MouseButton::Left) {
        let SelectedFigure::Some {
            board_entity,
            figure_entity,
            was_put_down_once,
        } = *selected_figure
        else {
            return;
        };

        let board = q_board.get(board_entity).unwrap();
        let (figure, mut figure_transform, grabbed) = q_figure.get_mut(figure_entity).unwrap();

        // reset the figure's "visual" position
        figure_transform.translation = board.board_to_world(figure.position).extend(board.figure_z);

        let mut release_selected_figure = |selected_figure: &mut ResMut<SelectedFigure>| {
            **selected_figure = SelectedFigure::None;
            despawn_highlights_event.send(DespawnHighlightsEvent { board_entity });
        };

        let moved = 'blk: {
            let Some(mouse_position) = q_mouse_position.get_single().unwrap().mouse_world_position
            else {
                break 'blk false;
            };

            let Some(to) = board.world_to_board(mouse_position) else {
                break 'blk false;
            };

            let from = figure.position;

            if from == to {
                break 'blk false;
            }

            release_selected_figure(&mut selected_figure);

            if grabbed.is_some() {
                move_figure_event.send(MoveFigureEvent {
                    board_entity,
                    from,
                    to,
                });
            } else {
                // TODO call slide_and_move here instead
                move_figure_event.send(MoveFigureEvent {
                    board_entity,
                    from,
                    to,
                });
            }

            true
        };

        if !moved {
            match was_put_down_once {
                true => {
                    release_selected_figure(&mut selected_figure);
                }
                false => {
                    if let SelectedFigure::Some {
                        ref mut was_put_down_once,
                        ..
                    } = selected_figure.as_mut()
                    {
                        *was_put_down_once = true;
                    }
                }
            }
        }

        commands.entity(figure_entity).remove::<Grabbed>();
    }
}
