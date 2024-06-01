use crate::game::camera::*;
use crate::game::tafl::*;

#[derive(Component, Debug, Default, Clone)]
pub struct SelectedFigure {
    pub entity: Option<Entity>,
}

/// Selects the figure at the mouse position.
pub fn select_figure(
    buttons: Res<ButtonInput<MouseButton>>,
    q_mouse_position: Query<&MousePositionTracker, With<MainCamera>>,
    mut q_board: Query<(Entity, &Board, &mut SelectedFigure, &Transform)>,
    q_figure: Query<&Figure>,
    mut spawn_highlights_event: EventWriter<SpawnHighlightsEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let Some(mouse_position) = q_mouse_position.get_single().unwrap().mouse_world_position
        else {
            return;
        };

        // get closest board
        let Some((board_entity, board, mut selected_figure)) = ({
            let mut result = None;
            let mut result_distance = 0.;

            for components in &mut q_board {
                let distance = components.3.translation.xy().distance(mouse_position);

                if result.is_none() || distance < result_distance {
                    result_distance = distance;
                    result = Some((components.0, components.1, components.2));
                    continue;
                }
            }

            result
        }) else {
            return;
        };

        // the selection has to be cleared before selecting a new figure
        if selected_figure.entity.is_some() {
            return;
        }

        let Some(selected_field) = board.world_to_board(mouse_position) else {
            return;
        };

        let Some(selected_figure_entity_reference) = board.figures.get(&selected_field) else {
            return;
        };
        let selected_figure_entity = *selected_figure_entity_reference;

        selected_figure.entity = Some(selected_figure_entity);

        let figure = q_figure.get(selected_figure_entity).unwrap();
        spawn_highlights_event.send(SpawnHighlightsEvent {
            board_entity,
            positions: possible_moves(board, *figure),
        });
    }
}

/// Snaps the selected figure to the mouse position.
pub fn drag_selected_figure(
    q_mouse_position: Query<&MousePositionTracker, With<MainCamera>>,
    q_board: Query<(&Board, &SelectedFigure)>,
    mut q_figure_transform: Query<&mut Transform, With<Figure>>,
) {
    let Some(mouse_position) = q_mouse_position.get_single().unwrap().mouse_world_position else {
        return;
    };

    for (board, selected_figure) in &q_board {
        let Some(figure_entity) = selected_figure.entity else {
            continue;
        };

        let mut figure_transform = q_figure_transform.get_mut(figure_entity).unwrap();
        figure_transform.translation = mouse_position.clone().extend(board.figure_z + 1.);
    }
}

/// Releases selected figure and attempts to move it to the field corresponding to the mouse
/// position.
pub fn release_selected_figure(
    buttons: Res<ButtonInput<MouseButton>>,
    q_mouse_position: Query<&MousePositionTracker, With<MainCamera>>,
    mut q_board: Query<(Entity, &Board, &mut SelectedFigure)>,
    mut q_figure: Query<(&Figure, &mut Transform)>,
    mut despawn_highlights_event: EventWriter<DespawnHighlightsEvent>,
    mut move_figure_event: EventWriter<MoveFigureEvent>,
) {
    if buttons.just_released(MouseButton::Left) {
        for (board_entity, board, mut selected_figure) in &mut q_board {
            let Some(figure_entity) = selected_figure.entity else {
                continue;
            };

            let (figure, mut figure_transform) = q_figure.get_mut(figure_entity).unwrap();

            // reset the figure's "visual" position
            figure_transform.translation =
                board.board_to_world(figure.position).extend(board.figure_z);

            let from = figure.position;
            selected_figure.entity = None;

            despawn_highlights_event.send(DespawnHighlightsEvent { board_entity });

            let Some(mouse_position) = q_mouse_position.get_single().unwrap().mouse_world_position
            else {
                return;
            };

            let Some(to) = board.world_to_board(mouse_position) else {
                return;
            };

            move_figure_event.send(MoveFigureEvent {
                board_entity,
                from,
                to,
            });
        }
    }
}
