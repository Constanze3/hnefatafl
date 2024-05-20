use core::panic;

use bevy::{
    math::Vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::{close_on_esc, PrimaryWindow},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const BOARD: &'static str = "
40033333004
00000300000
00000000000
30000200003
30002220003
33022122033
30002220003
30000200003
00000000000
00000300000
40033333004
";

const STARTING_POSITION: &'static str = "
a s 0 3
a s 0 4
a s 0 5
a s 0 6
a s 0 7
a s 1 5
a s 3 0
d s 3 5
a s 3 10
a s 4 0
d s 4 4
d s 4 5
d s 4 6
a s 4 10
a s 5 0
a s 5 1
d s 5 3
d s 5 4
d k 5 5
d s 5 6
d s 5 7
a s 5 9
a s 5 10
a s 6 0 
d s 6 4
d s 6 5
d s 6 6
a s 6 10
a s 7 0
d s 7 5
a s 7 10
a s 9 5
a s 10 3
a s 10 4
a s 10 5
a s 10 6
a s 10 7
";

fn main() {
    let board_data = {
        let (rows, cols, structure) = BoardData::parse_structure(BOARD).unwrap();

        let mut colors = HashMap::<u8, Color>::new();
        colors.insert(0, Color::rgb(0.5, 0.5, 0.5));
        colors.insert(1, Color::rgb(0.3, 0.4, 1.0));
        colors.insert(2, Color::rgb(0.2, 0.2, 0.7));
        colors.insert(3, Color::rgb(0.5, 0.3, 0.3));
        colors.insert(4, Color::rgb(0.6, 0.2, 0.2));

        let starting_position =
            BoardData::parse_arrangement(rows, cols, STARTING_POSITION).unwrap();

        BoardData {
            rows,
            cols,
            structure,
            starting_position,
            field_size: 50.,
            field_colors: colors,
            border_width: 4.,
            outer_border_width: 12.,
            border_color: Color::rgb_u8(42, 37, 37),
            display_z: 0.,
            figure_display_z: 2.,
        }
    };

    let figure_data = {
        let mut colors = HashMap::<Side, Color>::new();
        colors.insert(Side::Attacker, Color::rgb(0., 0., 0.));
        colors.insert(Side::Defender, Color::rgb(1., 1., 1.));

        let mut shapes = HashMap::<FigureKind, Mesh>::new();
        let square_size = 0.65 * board_data.field_size;
        shapes.insert(
            FigureKind::Soldier,
            Rectangle::new(square_size, square_size).into(),
        );
        shapes.insert(
            FigureKind::King,
            Circle::new(0.4 * board_data.field_size).into(),
        );

        FigureData {
            colors,
            meshes: shapes,
        }
    };

    let clear_color = ClearColor(Color::rgb(0., 0., 0.));

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .init_resource::<SelectedFigure>()
        .init_resource::<MousePosition>()
        .insert_resource(board_data)
        .insert_resource(figure_data)
        .insert_resource(clear_color)
        .add_systems(Startup, (setup, (spawn_board, setup_board).chain()))
        .add_systems(
            Update,
            (
                update_mouse_position,
                (
                    close_on_esc,
                    (select_figure, spawn_highlights).chain(),
                    visual_move_figure,
                    (move_figure, despawn_highlights).chain(),
                )
                    .after(update_mouse_position),
            ),
        )
        .add_event::<CreateBoardEvent>()
        .add_event::<SpawnHighlightsEvent>()
        .add_event::<DespawnHighlightsEvent>()
        .run();
}

// RESOURCES

/// Data on how exactly to initialize the board.
#[derive(Resource)]
struct BoardData {
    rows: usize,
    cols: usize,
    structure: Vec<Vec<u8>>,
    starting_position: Vec<Figure>,
    field_size: f32,
    field_colors: HashMap<u8, Color>,
    border_width: f32,
    outer_border_width: f32,
    border_color: Color,
    figure_display_z: f32,
    display_z: f32,
}

impl BoardData {
    fn parse_structure(data: &str) -> Result<(usize, usize, Vec<Vec<u8>>), &str> {
        let mut structure: Vec<Vec<u8>> = vec![];

        let mut rows = 0;
        let mut cols = 0;

        let mut current_row: Vec<u8> = vec![];
        for (i, c) in data.chars().enumerate() {
            if i == 0 && c == '\n' {
                continue;
            }

            if c != '\n' {
                if c.is_ascii_digit() {
                    let digit = c as u8 - '0' as u8;
                    current_row.push(digit);
                } else {
                    return Err("board data should only consist of numbers and \\n");
                }
            } else {
                if rows == 0 {
                    cols = current_row.len()
                } else if current_row.len() != cols {
                    return Err("board should have consistent row length");
                }

                structure.push(current_row.clone());
                current_row.clear();
                rows += 1;
            }
        }

        Ok((rows, cols, structure))
    }

    fn parse_arrangement(rows: usize, cols: usize, data: &str) -> Result<Vec<Figure>, &str> {
        let not_enough_tokens = "a figure should be described by 4, space-separated values";

        let mut result: Vec<Figure> = vec![];

        for line in data.lines() {
            if line == "" {
                continue;
            }

            let mut token_iter = line.split(' ');

            let side = if let Some(side_unparsed) = token_iter.next() {
                match side_unparsed {
                    "a" => Side::Attacker,
                    "d" => Side::Defender,
                    _ => return Err("side described by data should be either a or d"),
                }
            } else {
                return Err(not_enough_tokens);
            };

            let kind = if let Some(kind_unparsed) = token_iter.next() {
                match kind_unparsed {
                    "k" => FigureKind::King,
                    "s" => FigureKind::Soldier,
                    _ => return Err("kind described by data should be either k or s"),
                }
            } else {
                return Err(not_enough_tokens);
            };

            let x = if let Some(x_unparsed) = token_iter.next() {
                let x_parsed = x_unparsed
                    .parse::<usize>()
                    .expect("position x should be a usize");
                x_parsed
            } else {
                return Err(not_enough_tokens);
            };

            let y = if let Some(y_unparsed) = token_iter.next() {
                let y_parsed = y_unparsed
                    .parse::<usize>()
                    .expect("position y should be a usize");
                y_parsed
            } else {
                return Err(not_enough_tokens);
            };

            if cols <= x {
                return Err("x coordinate of figure should be less then the width of the board");
            }

            if rows <= y {
                return Err("y coordinate of figure should be less then the height of the board");
            }

            result.push(Figure {
                side,
                kind,
                board_position: BoardPosition { x, y },
            });
        }

        Ok(result)
    }
}

#[derive(Resource)]
struct FigureData {
    colors: HashMap<Side, Color>,
    meshes: HashMap<FigureKind, Mesh>,
}

#[derive(Debug, Clone)]
struct SelectedFigure_ {
    entity: Entity,
    possible_moves: Vec<BoardPosition>,
}

#[derive(Resource, Default, Debug, Clone)]
enum SelectedFigure {
    Some(SelectedFigure_),
    #[default]
    None,
}

#[derive(Resource, Default, Debug)]
struct MousePosition(Option<Vec2>);

// COMPONENTS
#[derive(Component)]
struct Board {
    rows: usize,
    cols: usize,
    starting_position: Vec<Figure>,
    end_positions: Vec<BoardPosition>,
    figures: HashMap<BoardPosition, Entity>,

    field_size: f32,
    border_width: f32,
    outer_border_width: f32,
    field_offset: f32,
    upper_left_field_position: Vec2,
    upper_left_corner_position: Vec2,
    // width excluding outer border
    width: f32,
    // height exluding outer border
    height: f32,
    figure_display_z: f32,
}

#[derive(Component, Clone)]
struct BoardHighlights {
    mesh: Handle<Mesh>,
    color: Handle<ColorMaterial>,
    z: f32,
    entity: Option<Entity>,
}

impl Board {
    fn new(data: &BoardData, upper_left: Vec2) -> Self {
        let mut end_positions: Vec<BoardPosition> = vec![];
        for (y, row) in data.structure.iter().enumerate() {
            for (x, element) in row.iter().enumerate() {
                if *element == 4 {
                    end_positions.push(BoardPosition { x, y })
                }
            }
        }

        Self {
            rows: data.rows,
            cols: data.cols,
            starting_position: data.starting_position.to_vec(),
            end_positions,
            figures: HashMap::new(),

            field_size: data.field_size,
            border_width: data.border_width,
            outer_border_width: data.outer_border_width,

            field_offset: data.field_size + data.border_width,
            upper_left_field_position: upper_left,
            upper_left_corner_position: Vec2 {
                x: upper_left.x - data.field_size / 2.,
                y: upper_left.y + data.field_size / 2.,
            },

            width: data.cols as f32 * (data.field_size + data.border_width),
            height: data.rows as f32 * (data.field_size + data.border_width),

            figure_display_z: data.figure_display_z,
        }
    }

    fn add_figure(&mut self, position: BoardPosition, figure: Entity) {
        self.figures.insert(position, figure);
    }

    /// Converts a world position to the position of a field on the board.
    fn world_to_board(&self, position: Vec2) -> Option<BoardPosition> {
        let ulc = self.upper_left_corner_position;

        let x_adjusted = position.x - ulc.x;
        let y_adjusted = -(position.y - ulc.y);

        if x_adjusted < 0.
            || self.width <= x_adjusted
            || y_adjusted < 0.
            || self.height <= y_adjusted
        {
            return None;
        }

        let x = x_adjusted as usize / self.field_offset as usize;
        let y = y_adjusted as usize / self.field_offset as usize;

        Some(BoardPosition { x, y })
    }

    /// Converts a positon on the board to a world position.
    fn board_to_world(&self, position: BoardPosition) -> Vec2 {
        let upper_left = self.upper_left_field_position;
        let field_offset = self.field_offset;

        let x = upper_left.x + position.x as f32 * field_offset;
        let y = upper_left.y - position.y as f32 * field_offset;

        Vec2 { x, y }
    }

    fn possible_moves(&self, figure: Figure) -> Result<Vec<BoardPosition>, &str> {
        let position = figure.board_position;

        if self.figures.get(&position) == None {
            return Err("figure isn't on the board");
        }

        let is_valid = |target_position: &BoardPosition| -> bool {
            let figure_at_target = self.figures.get(target_position) != None;
            let is_end_pos = self.end_positions.contains(target_position);
            let is_king = figure.kind == FigureKind::King;

            return !figure_at_target && (!is_end_pos || is_king);
        };

        let mut result: Vec<BoardPosition> = vec![];

        for x in (0..position.x).rev() {
            let target_position = BoardPosition { x, y: position.y };

            if is_valid(&target_position) {
                result.push(target_position);
            } else {
                break;
            }
        }

        for x in (position.x + 1)..self.cols {
            let target_position = BoardPosition { x, y: position.y };

            if is_valid(&target_position) {
                result.push(target_position);
            } else {
                break;
            }
        }

        for y in (0..position.y).rev() {
            let target_position = BoardPosition { x: position.x, y };

            if is_valid(&target_position) {
                result.push(target_position);
            } else {
                break;
            }
        }

        for y in (position.y + 1)..self.rows {
            let target_position = BoardPosition { x: position.x, y };

            if is_valid(&target_position) {
                result.push(target_position);
            } else {
                break;
            }
        }

        Ok(result)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct BoardPosition {
    x: usize,
    y: usize,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Side {
    Attacker,
    Defender,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum FigureKind {
    King,
    Soldier,
}

#[derive(Component, Debug, Copy, Clone)]
struct Figure {
    side: Side,
    kind: FigureKind,
    board_position: BoardPosition,
}

#[derive(Component)]
struct MainCamera;

// EVENTS

#[derive(Event)]
struct CreateBoardEvent(Entity);

// SYSTEMS

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

fn update_mouse_position(
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_position: ResMut<MousePosition>,
) {
    let (camera, camera_transform) = q_camera.single();
    *mouse_position = MousePosition(
        q_windows
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor)),
    );
}

fn spawn_board(
    board_data: Res<BoardData>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut event: EventWriter<CreateBoardEvent>,
) {
    let rows = board_data.rows as f32;
    let cols = board_data.cols as f32;

    let field_size = board_data.field_size;
    let border_width = board_data.border_width;
    let outer_border_width = board_data.outer_border_width;

    let field_offset = field_size + border_width;

    let total_width = field_offset * cols + border_width;
    let total_height = field_offset * rows + border_width;

    let offset_x = -(total_width / 2. - field_offset / 2.);
    let offset_y = total_height / 2. - field_offset / 2.;

    let background = {
        let size_x = total_width - 2. * border_width + 2. * outer_border_width;
        let size_y = total_height - 2. * border_width + 2. * outer_border_width;

        commands
            .spawn((
                Name::new("Background"),
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(size_x, size_y))),
                    material: materials.add(board_data.border_color),
                    transform: Transform::from_xyz(0., 0., board_data.display_z - 1.),
                    ..default()
                },
            ))
            .id()
    };

    let fields = {
        let result = commands
            .spawn((Name::new("Fields"), SpatialBundle::default()))
            .id();

        let color_map: HashMap<u8, Handle<ColorMaterial>> = board_data
            .field_colors
            .iter()
            .map(|(key, color)| (*key, materials.add(*color)))
            .collect();

        let field = Mesh2dHandle(meshes.add(Rectangle::new(field_size, field_size)));
        for (i, row) in board_data.structure.iter().enumerate() {
            for (j, element) in row.iter().enumerate() {
                let color = color_map
                    .get(element)
                    .expect("there should be a color associated with the provided element")
                    .clone();

                let i = i as f32;
                let j = j as f32;

                let field = commands
                    .spawn((
                        Name::new("Field"),
                        MaterialMesh2dBundle {
                            mesh: field.clone(),
                            material: color,
                            transform: Transform::from_xyz(
                                offset_x + j * field_offset,
                                offset_y - i * field_offset,
                                board_data.display_z,
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

    let board_highlight = BoardHighlights {
        mesh: meshes.add(Circle::new(0.2 * board_data.field_size)),
        color: materials.add(Color::rgba(0., 0., 0., 0.6)),
        z: 4.,
        entity: None,
    };

    let board = {
        let upper_left = Vec2 {
            x: offset_x,
            y: offset_y,
        };

        let result = commands
            .spawn((
                Name::new("Board"),
                SpatialBundle::default(),
                Board::new(&board_data, upper_left),
                board_highlight,
            ))
            .id();

        commands.entity(result).push_children(&[background, fields]);

        result
    };

    event.send(CreateBoardEvent(board));
}

fn setup_board(
    mut q_board: Query<&mut Board>,
    figure_data: Res<FigureData>,
    mut event: EventReader<CreateBoardEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for CreateBoardEvent(entity) in event.read() {
        let mut board = q_board.get_mut(*entity).unwrap();

        let mesh_map: HashMap<FigureKind, Mesh2dHandle> = figure_data
            .meshes
            .iter()
            .map(|(kind, mesh)| (*kind, Mesh2dHandle(meshes.add(mesh.clone()))))
            .collect();

        let material_map: HashMap<Side, Handle<ColorMaterial>> = figure_data
            .colors
            .iter()
            .map(|(side, color)| (*side, materials.add(*color)))
            .collect();

        for figure in &board.starting_position.clone() {
            let mesh = mesh_map
                .get(&figure.kind)
                .expect("there should be a mesh associated with the specified figure kind")
                .clone();

            let material = material_map
                .get(&figure.side)
                .expect("there should be a material associated with the specified side")
                .clone();

            let figure_entity = commands
                .spawn((
                    Name::new("Figure"),
                    MaterialMesh2dBundle {
                        mesh,
                        material,
                        transform: Transform::from_translation(
                            board
                                .board_to_world(figure.board_position)
                                .extend(board.figure_display_z),
                        ),
                        ..default()
                    },
                    *figure,
                ))
                .id();

            board.add_figure(figure.board_position, figure_entity);
        }
    }
}

fn select_figure(
    q_figures: Query<(Entity, &Figure)>,
    q_board: Query<(Entity, &Board)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut selected_figure: ResMut<SelectedFigure>,
    mut highlights_event: EventWriter<SpawnHighlightsEvent>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let Some(mouse_position) = mouse_position.0 else {
            return;
        };

        let (board_entity, board) = q_board.get_single().unwrap();
        let Some(selected_field) = board.world_to_board(mouse_position) else {
            return;
        };

        *selected_figure = {
            let mut result = SelectedFigure::None;
            for (figure_entity, figure) in &q_figures {
                if figure.board_position == selected_field {
                    let possible_moves = board.possible_moves(*figure).unwrap();

                    result = SelectedFigure::Some(SelectedFigure_ {
                        entity: figure_entity,
                        possible_moves: possible_moves.clone(),
                    });

                    highlights_event.send(SpawnHighlightsEvent {
                        board_entity,
                        positions: possible_moves.clone(),
                    });

                    break;
                }
            }

            result
        };
    }
}

#[derive(Event)]
struct SpawnHighlightsEvent {
    board_entity: Entity,
    positions: Vec<BoardPosition>,
}

#[derive(Event)]
struct DespawnHighlightsEvent {
    board_entity: Entity,
}

/// Spawns highlights for the specified positions of the board
///
/// Unchecked pre: The positions in the SpawnHighightEvent are on the board
fn spawn_highlights(
    mut q_board_highlights: Query<(&Board, &mut BoardHighlights)>,
    mut event: EventReader<SpawnHighlightsEvent>,
    mut commands: Commands,
) {
    for ev in event.read() {
        let Ok((board, mut highlights)) = q_board_highlights.get_mut(ev.board_entity) else {
            return;
        };

        let parent = commands
            .spawn((Name::new("Highlights"), SpatialBundle::default()))
            .id();
        commands.entity(ev.board_entity).push_children(&[parent]);

        let mesh = &highlights.mesh;
        let material = &highlights.color;

        for position in &ev.positions {
            let highlight = commands
                .spawn((
                    Name::new("highlight"),
                    MaterialMesh2dBundle {
                        mesh: mesh.clone().into(),
                        material: material.clone(),
                        transform: Transform::from_translation(
                            board.board_to_world(*position).extend(highlights.z),
                        ),
                        ..default()
                    },
                ))
                .id();

            commands.entity(parent).push_children(&[highlight]);
        }

        highlights.entity = Some(parent);
    }
}

fn despawn_highlights(
    mut q_highlights: Query<&mut BoardHighlights, With<Board>>,
    mut event: EventReader<DespawnHighlightsEvent>,
    mut commands: Commands,
) {
    for ev in event.read() {
        let Ok(mut highlights) = q_highlights.get_mut(ev.board_entity) else {
            return;
        };

        let e = highlights
            .entity
            .expect("non-existent highlights shouldn't be deleted");

        commands.entity(e).despawn_recursive();
        highlights.entity = None;
    }
}

fn visual_move_figure(
    mut q_figure_transforms: Query<&mut Transform, With<Figure>>,
    mouse_position: Res<MousePosition>,
    selected_figure: Res<SelectedFigure>,
) {
    if let SelectedFigure::Some(SelectedFigure_ {
        entity: figure_entity,
        possible_moves: _,
    }) = *selected_figure
    {
        let Some(mouse_position) = mouse_position.0 else {
            return;
        };

        let mut figure_transform = q_figure_transforms.get_mut(figure_entity).unwrap();
        figure_transform.translation = mouse_position.clone().extend(5.);
    };
}

fn move_figure(
    mut q_figure: Query<(&mut Figure, &mut Transform)>,
    mut q_board: Query<(Entity, &mut Board)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mouse_position: Res<MousePosition>,
    mut selected_figure: ResMut<SelectedFigure>,
    mut highlights_event: EventWriter<DespawnHighlightsEvent>,
) {
    if buttons.just_released(MouseButton::Left) {
        let SelectedFigure::Some(SelectedFigure_ {
            entity: figure_entity,
            possible_moves,
        }) = selected_figure.clone()
        else {
            return;
        };

        let (board_entity, mut board) = q_board.get_single_mut().unwrap();
        let (mut figure, mut figure_transform) = q_figure.get_mut(figure_entity).unwrap();

        _ = 'perform_move: {
            let Some(mouse_position) = mouse_position.0 else {
                break 'perform_move;
            };

            let Some(targeted_field) = board.world_to_board(mouse_position) else {
                break 'perform_move;
            };

            let at_target = board.figures.get(&targeted_field);
            if at_target != None {
                // if there is something at the target the figure can't be moved there
                break 'perform_move;
            }

            if !possible_moves.contains(&targeted_field) {
                break 'perform_move;
            }

            // update board.figures
            if let Some(val) = board.figures.remove(&figure.board_position) {
                board.figures.insert(targeted_field, val);
            }
            // update figure
            figure.board_position = targeted_field;
        };

        figure_transform.translation = board
            .board_to_world(figure.board_position)
            .extend(board.figure_display_z);

        *selected_figure = SelectedFigure::None;
        highlights_event.send(DespawnHighlightsEvent { board_entity });
    }
}
