use crate::game::tafl::*;

use self::victory_ui::SpawnVictoryUiEvent;

#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Playing,
    End,
}

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
    mut end_game_event: EventWriter<EndGameEvent>,
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
            end_game_event.send(EndGameEvent {
                winner: Side::Defender,
            });
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
    mut end_game_event: EventWriter<EndGameEvent>,
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
            end_game_event.send(EndGameEvent {
                winner: Side::Attacker,
            });
        }
    }
}

pub fn game_timer_check(
    mut event: EventReader<OnGameTimerFinishedEvent>,
    mut end_game_event: EventWriter<EndGameEvent>,
) {
    for ev in event.read() {
        if ev.side == Side::Attacker {
            end_game_event.send(EndGameEvent {
                winner: Side::Defender,
            });
        } else {
            end_game_event.send(EndGameEvent {
                winner: Side::Attacker,
            });
        }
    }
}

#[derive(Event)]
pub struct EndGameEvent {
    winner: Side,
}

pub fn on_game_end(
    mut event: EventReader<EndGameEvent>,
    mut game_state: ResMut<GameState>,
    mut indicate_turn_event: EventWriter<IndicateTurnEvent>,
    mut spawn_victory_ui_event: EventWriter<SpawnVictoryUiEvent>,
) {
    for ev in event.read() {
        indicate_turn_event.send(IndicateTurnEvent { side: None });
        spawn_victory_ui_event.send(SpawnVictoryUiEvent { winner: ev.winner });
        *game_state = GameState::End;
    }
}
