use bevy::{
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
};

use crate::game::tafl::*;

#[derive(Event, Clone)]
pub struct SpawnBoardEvent {
    pub id: SimpleId,
    pub position: Vec3,
    pub board: Board,
    pub field_materials: HashMap<Position, Handle<ColorMaterial>>,
    pub border_z: f32,
    pub border_material: Handle<ColorMaterial>,
    pub highlight_mesh: Handle<Mesh>,
    pub highlight_material: Handle<ColorMaterial>,
    pub highlight_z: f32,
}

#[derive(Component)]
pub struct MainBoard;

pub fn spawn_board(
    mut event: EventReader<SpawnBoardEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
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
                        mesh: meshes.add(Rectangle::new(size_x, size_y)).into(),
                        material: ev.border_material.clone(),
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

            for i in 0..ev.board.cols {
                for j in 0..ev.board.rows {
                    let position = Position { x: i, y: j };

                    let mesh = mesh.clone();
                    let material = ev
                        .field_materials
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
                mesh: ev.highlight_mesh.clone(),
                color: ev.highlight_material.clone(),
                z: ev.highlight_z,
                entity: None,
            };

            let result = commands
                .spawn((
                    MainBoard,
                    Name::new("Board"),
                    ev.id,
                    SpatialBundle {
                        transform: Transform::from_translation(ev.position),
                        ..default()
                    },
                    ev.board.clone(),
                    TurnTracker {
                        side: Side::Attacker,
                    },
                    board_highlights,
                    OnCaptureCheckEndTracker::default(),
                ))
                .id();

            commands.entity(result).push_children(&[borders, fields]);

            result
        };

        _ = board;
    }
}

pub fn despawn_board(q_board: Query<Entity, With<MainBoard>>, mut commands: Commands) {
    let board_entity = q_board.single();
    commands.entity(board_entity).despawn_recursive();
}

#[derive(Event, Clone)]
pub struct SpawnFiguresEvent {
    pub board_id: SimpleId,
    pub figures: Vec<Figure>,
    pub textures: HashMap<FigureType, Handle<Image>>,
}

#[derive(Component)]
pub struct MainFigures;

pub fn spawn_figures(
    mut event: EventReader<SpawnFiguresEvent>,
    mut commands: Commands,
    mut q_board: Query<(&SimpleId, &mut Board)>,
) {
    for ev in event.read() {
        let mut board = 'blk: {
            for (id, b) in &mut q_board {
                if ev.board_id == *id {
                    break 'blk b;
                }
            }

            panic!("board_id should be a valid id");
        };

        let parent = commands
            .spawn((
                MainFigures,
                Name::new("Figures"),
                SpatialBundle {
                    transform: Transform::from_translation(Vec3::ZERO),
                    ..default()
                },
            ))
            .id();

        for figure in &ev.figures {
            let texture = ev
                .textures
                .get(&FigureType {
                    side: figure.side,
                    kind: figure.kind,
                })
                .expect("all used figure types should have a texture")
                .clone();

            let figure_entity = commands
                .spawn((
                    Name::new(format!(
                        "{} {}",
                        figure.side.to_string(),
                        figure.kind.to_string()
                    )),
                    SpriteBundle {
                        texture,
                        transform: Transform::from_translation(
                            board.board_to_world(figure.position).extend(board.figure_z),
                        ),
                        ..default()
                    },
                    *figure,
                ))
                .id();

            board.figures.insert(figure.position, figure_entity);
            commands.entity(parent).add_child(figure_entity);
        }
    }
}

pub fn despawn_figures(q_figures: Query<Entity, With<MainFigures>>, mut commands: Commands) {
    let figures_entity = q_figures.single();
    commands.entity(figures_entity).despawn_recursive();
}
