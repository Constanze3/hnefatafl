use bevy::prelude::*;

use self::board::*;
use self::board_highlights::*;
use self::figure::*;
use self::move_validation::*;

mod board;
mod board_highlights;
mod figure;
mod move_validation;

pub struct TaflPlugin;

impl Plugin for TaflPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<SetPossibleMovesEvent>()
            .add_event::<SpawnHighlightsEvent>()
            .add_event::<DespawnHighlightsEvent>()
            .add_systems(
                Update,
                (set_possible_moves, spawn_highlights, despawn_highlights).chain(),
            );
    }
}
