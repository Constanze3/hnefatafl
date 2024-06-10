use crate::game::tafl::*;

#[derive(Event)]
pub struct KingOnCornerCheckEvent {
    pub board_entity: Entity,
}

// TODO work on this more later
// especially after defining a clear model
// with rules such as
// - there may exist only one king

pub fn king_on_corner_check(
    mut event: EventReader<KingOnCornerCheckEvent>,
    q_board: Query<&Board>,
    q_figure: Query<&Figure>,
) {
    for ev in event.read() {
        let board = q_board.get(ev.board_entity).unwrap();

        // all defender kings are on an end position
        let mut win = true;
        for figure_entity in board.figures.values() {
            let figure = q_figure.get(*figure_entity).unwrap();
            if figure.side == Side::Defender && figure.kind == FigureKind::King {
                if !board.end_positions.contains(&figure.position) {
                    win = false;
                    break;
                }
            }
        }

        if win {
            println!("defender wins!");
        }
    }
}

#[derive(Event)]
pub struct KingSurroundedCheckEvent {
    pub board_entity: Entity,
}

pub fn king_surrounded_check(
    mut event: EventReader<KingSurroundedCheckEvent>,
    q_board: Query<&Board>,
    q_figure: Query<&Figure>,
) {
    for ev in event.read() {
        let board = q_board.get(ev.board_entity).unwrap();

        // all defender kings are surrounded
        // being next to a wall, end_position or throne also counts as being surrounded.
        let mut win = true;
        for figure_entity in board.figures.values() {
            let figure = q_figure.get(*figure_entity).unwrap();

            if figure.side == Side::Defender && figure.kind == FigureKind::King {
                let neighbors = board.get_neighbors2(figure.position);

                for neighbor in neighbors.to_vec() {
                    if let Neighbor::Empty { position } = neighbor {
                        if !board.end_positions.contains(&position)
                            && board.throne_position != position
                        {
                            win = false;
                            break;
                        }
                    }

                    if let Neighbor::Some {
                        entity: neighbor_entity,
                        ..
                    } = neighbor
                    {
                        let neighbor_figure = q_figure.get(neighbor_entity).unwrap();
                        if neighbor_figure.side != Side::Attacker {
                            win = false;
                            break;
                        }
                    }
                }
            }
        }

        if win {
            println!("attacker wins!");
        }
    }
}
