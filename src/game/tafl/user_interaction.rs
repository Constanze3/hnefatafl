use crate::game::camera::*;
use crate::game::tafl::*;

#[derive(Component, Debug, Default, Clone)]
pub struct SelectedFigure(pub Option<Entity>);

/// System that allows the users to select figures.
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

        let Some(selected_field) = board.world_to_board(mouse_position) else {
            return;
        };

        let Some(selected_figure_entity_reference) = board.figures.get(&selected_field) else {
            return;
        };
        let selected_figure_entity = *selected_figure_entity_reference;

        selected_figure.0 = Some(selected_figure_entity);

        let figure = q_figure.get(selected_figure_entity).unwrap();
        spawn_highlights_event.send(SpawnHighlightsEvent {
            board_entity,
            positions: possible_moves(board, *figure),
        });
    }
}

//
// fn visual_move_figure(
//     mut q_figure_transforms: Query<&mut Transform, With<Figure>>,
//     mouse_position: Res<MousePosition>,
//     selected_figure: Res<SelectedFigure>,
// ) {
//     if let SelectedFigure::Some(SelectedFigure_ {
//         entity: figure_entity,
//         possible_moves: _,
//     }) = *selected_figure
//     {
//         let Some(mouse_position) = mouse_position.0 else {
//             return;
//         };
//
//         let mut figure_transform = q_figure_transforms.get_mut(figure_entity).unwrap();
//         figure_transform.translation = mouse_position.clone().extend(5.);
//     };
// }
