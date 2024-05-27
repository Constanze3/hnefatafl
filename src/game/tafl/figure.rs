use crate::game::tafl::board::Position;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Side {
    Attacker,
    Defender,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FigureKind {
    King,
    Soldier,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Figure {
    pub side: Side,
    pub kind: FigureKind,
    pub board_position: Position,
}
