use core::fmt;

use crate::game::tafl::board::Position;
use bevy::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Side {
    Attacker,
    Defender,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Attacker => write!(f, "Attacker"),
            Side::Defender => write!(f, "Defender"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FigureKind {
    King,
    Soldier,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FigureType {
    pub side: Side,
    pub kind: FigureKind,
}

// TODO make a proc macro for this to experience them
impl fmt::Display for FigureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FigureKind::King => write!(f, "King"),
            FigureKind::Soldier => write!(f, "Soldier"),
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Figure {
    pub side: Side,
    pub kind: FigureKind,
    pub position: Position,
}
