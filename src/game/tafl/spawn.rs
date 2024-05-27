use bevy::{
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
};

use crate::game::tafl::*;

#[derive(Event)]
pub struct SpawnTaflEvent(SpawnBoardEvent, SpawnFiguresEvent);

#[derive(Event, Clone)]
pub struct SpawnBoardEvent {
    pub position: Vec3,
    pub board: Board,
    pub field_materials: HashMap<Position, ColorMaterial>,
    pub border_z: f32,
    pub border_material: ColorMaterial,
    pub highlight_mesh: Mesh,
    pub highlight_material: ColorMaterial,
    pub highlight_z: f32,
}

#[derive(Event, Clone)]
pub struct SpawnFiguresEvent {}

/// Spawns a "tafl" board and figures.
pub fn spawn_tafl(
    mut event: EventReader<SpawnTaflEvent>,
    mut spawn_board_event: EventWriter<SpawnBoardEvent>,
    mut spawn_figures_event: EventWriter<SpawnFiguresEvent>,
) {
    for ev in event.read() {
        spawn_board_event.send(ev.0.clone());
        spawn_figures_event.send(ev.1.clone());
    }
}

pub fn spawn_board(
    mut event: EventReader<SpawnBoardEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for ev in event.read() {
        let borders = {
            let size_x =
                ev.board.width - 2. * ev.board.border_width + 2. * ev.board.outer_border_width;
            let size_y =
                ev.board.height - 2. * ev.board.border_width + 2. * ev.board.outer_border_width;

            commands
                .spawn((
                    Name::new("Background"),
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Rectangle::new(size_x, size_y))),
                        material: materials.add(ev.border_material.clone()),
                        transform: Transform::from_translation(
                            ev.position.xy().extend(ev.border_z),
                        ),
                        ..default()
                    },
                ))
                .id()
        };

        let fields = {
            let result = commands
                .spawn((
                    Name::new("Fields"),
                    SpatialBundle {
                        transform: Transform::from_translation(ev.position),
                        ..default()
                    },
                ))
                .id();

            let mesh =
                Mesh2dHandle(meshes.add(Rectangle::new(ev.board.field_size, ev.board.field_size)));

            let materials: HashMap<Position, Handle<ColorMaterial>> = ev
                .field_materials
                .iter()
                .map(|(key, value)| (*key, materials.add(value.clone())))
                .collect();

            for i in 0..ev.board.cols {
                for j in 0..ev.board.rows {
                    let position = Position { x: i, y: j };

                    let mesh = mesh.clone();
                    let material = materials
                        .get(&position)
                        .expect("every position should have an associated material")
                        .clone();

                    let field = commands
                        .spawn((
                            Name::new("Field"),
                            MaterialMesh2dBundle {
                                mesh,
                                material,
                                transform: Transform::from_translation(
                                    ev.position + ev.board.board_to_world(position).extend(0.),
                                ),
                                ..default()
                            },
                        ))
                        .id();

                    commands.entity(result).add_child(field);
                }
            }

            result
        };

        let board = {
            let board_highlights = BoardHighlights {
                mesh: meshes.add(ev.highlight_mesh.clone()),
                color: materials.add(ev.highlight_material.clone()),
                z: ev.highlight_z,
                entity: None,
            };

            let result = commands
                .spawn((
                    Name::new("Board"),
                    SpatialBundle {
                        transform: Transform::from_translation(ev.position),
                        ..default()
                    },
                    ev.board.clone(),
                    board_highlights,
                ))
                .id();

            commands.entity(result).push_children(&[borders, fields]);

            result
        };

        _ = board;
    }
}

// fn setup_board(
//     mut q_board: Query<&mut Board>,
//     figure_data: Res<FigureData>,
//     mut event: EventReader<CreateBoardEvent>,
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     for CreateBoardEvent(entity) in event.read() {
//         let mut board = q_board.get_mut(*entity).unwrap();
//
//         let mesh_map: HashMap<FigureKind, Mesh2dHandle> = figure_data
//             .meshes
//             .iter()
//             .map(|(kind, mesh)| (*kind, Mesh2dHandle(meshes.add(mesh.clone()))))
//             .collect();
//
//         let material_map: HashMap<Side, Handle<ColorMaterial>> = figure_data
//             .colors
//             .iter()
//             .map(|(side, color)| (*side, materials.add(*color)))
//             .collect();
//
//         for figure in &board.starting_position.clone() {
//             let mesh = mesh_map
//                 .get(&figure.kind)
//                 .expect("there should be a mesh associated with the specified figure kind")
//                 .clone();
//
//             let material = material_map
//                 .get(&figure.side)
//                 .expect("there should be a material associated with the specified side")
//                 .clone();
//
//             let figure_entity = commands
//                 .spawn((
//                     Name::new("Figure"),
//                     MaterialMesh2dBundle {
//                         mesh,
//                         material,
//                         transform: Transform::from_translation(
//                             board
//                                 .board_to_world(figure.board_position)
//                                 .extend(board.figure_display_z),
//                         ),
//                         ..default()
//                     },
//                     *figure,
//                 ))
//                 .id();
//
//             board.add_figure(figure.board_position, figure_entity);
//         }
//     }
// }
