use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;

use self::camera::*;
use self::tafl::*;

mod camera;
mod tafl;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin)
            .add_plugins(TaflPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut spawn_board_event: EventWriter<SpawnBoardEvent>) {
    let mut field_materials: HashMap<Position, ColorMaterial> = HashMap::new();
    field_materials.insert(Position { x: 0, y: 0 }, Color::rgb(1., 1., 1.).into());
    field_materials.insert(Position { x: 1, y: 0 }, Color::rgb(1., 0., 0.).into());

    let field_size = 50.;

    spawn_board_event.send(SpawnBoardEvent {
        position: Vec3::ZERO,
        board: Board::new(BoardOptions {
            cols: 2,
            rows: 1,
            throne_position: Position { x: 1, y: 1 },
            end_positions: vec![],
            figures: HashMap::new(),
            field_size,
            border_width: 4.,
            outer_border_width: 12.,
            figures_z: 2.,
        }),
        border_material: Color::rgb(0., 0., 0.).into(),
        border_z: -1.,
        field_materials,
        highlight_mesh: Circle::new(0.2 * field_size).into(),
        highlight_material: Color::rgba(0., 0., 0., 0.6).into(),
        highlight_z: 1.,
    });
}
