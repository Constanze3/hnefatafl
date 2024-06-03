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
        let mut win = true;
        for figure_entity in board.figures.values() {
            let figure = q_figure.get(*figure_entity).unwrap();
            if figure.side == Side::Defender && figure.kind == FigureKind::King {
                let neighbor_entities = board.get_neighbors(figure.position);
                if neighbor_entities.len() < 4 {
                    win = false;
                    break;
                }

                for neighbor_entity in neighbor_entities {
                    let neighbor = q_figure.get(neighbor_entity).unwrap();
                    if neighbor.side != Side::Attacker {
                        win = false;
                        break;
                    }
                }
            }
        }

        if win {
            println!("attacker wins!");
        }
    }
}
