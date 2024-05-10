use core::panic;

use bevy::{
    math::Vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::close_on_esc,
};

const BOARD: &'static str = "
4003333300400
0000030000000
0000000000000
3000020000300
3000222000300
3302212203300
3000222000300
3000020000300
0000000000000
0000030000000
4003333300400
0120012304000
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
        let structure = BoardData::<11, 12>::parse_structure(BOARD);

        let mut colors = HashMap::<u8, Color>::new();
        colors.insert(0, Color::rgb(0.7, 0.7, 0.7));
        colors.insert(1, Color::rgb(0.3, 0.4, 1.0));
        colors.insert(2, Color::rgb(0.2, 0.2, 0.7));
        colors.insert(3, Color::rgb(0.4, 0.2, 0.2));
        colors.insert(4, Color::rgb(0.6, 0.2, 0.2));

        let starting_position = BoardData::<11, 12>::parse_arrangement(STARTING_POSITION);

        BoardData::<11, 12> {
            structure,
            colors,
            starting_position,
            field_size: 50.,
            border_width: 4.,
            outer_border_width: 12.,
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

    let clear_color = ClearColor(Color::hex("19335E").unwrap());

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(board_data)
        .insert_resource(figure_data)
        .insert_resource(clear_color)
        .add_systems(
            Startup,
            (
                setup,
                (spawn_board::<11, 12>, setup_board::<11, 12>).chain(),
            ),
        )
        .add_systems(Update, close_on_esc)
        .add_event::<CreateBoardEvent>()
        .run();
}

// RESOURCES

/// Data on how exactly to initialize the board.
#[derive(Resource)]
struct BoardData<const WIDTH: usize, const HEIGHT: usize> {
    structure: [[u8; WIDTH]; HEIGHT],
    starting_position: Vec<Figure>,
    colors: HashMap<u8, Color>,
    field_size: f32,
    border_width: f32,
    outer_border_width: f32,
}

#[derive(Resource)]
struct FigureData {
    colors: HashMap<Side, Color>,
    meshes: HashMap<FigureKind, Mesh>,
}

impl<const WIDTH: usize, const HEIGHT: usize> BoardData<WIDTH, HEIGHT> {
    fn parse_structure(data: &str) -> [[u8; WIDTH]; HEIGHT] {
        let mut structure: [[u8; WIDTH]; HEIGHT] = [[0; WIDTH]; HEIGHT];

        let mut row = 0;
        let mut col = 0;
        for (i, c) in data.chars().enumerate() {
            if i == 0 && c == '\n' {
                continue;
            }

            if c != '\n' {
                if c.is_ascii_digit() {
                    structure[row][col] = c as u8 - '0' as u8;
                } else {
                    panic!("board data should only have numbers and /n")
                }
                col += 1;
            } else {
                row += 1;
                col = 0;
            }
        }

        return structure;
    }

    fn parse_arrangement(data: &str) -> Vec<Figure> {
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
                    _ => panic!("side described by data should be either a or d"),
                }
            } else {
                panic!("a figure should be described by 4, space-separated values");
            };

            let kind = if let Some(kind_unparsed) = token_iter.next() {
                match kind_unparsed {
                    "k" => FigureKind::King,
                    "s" => FigureKind::Soldier,
                    _ => panic!("kind described by data should be either k or s"),
                }
            } else {
                panic!("a figure should be described by 4, space-separated values");
            };

            let x = if let Some(x_unparsed) = token_iter.next() {
                let x_parsed = x_unparsed
                    .parse::<usize>()
                    .expect("position x should be a usize");
                x_parsed
            } else {
                panic!("a figure should be described by 4, space-separated values");
            };

            let y = if let Some(y_unparsed) = token_iter.next() {
                let y_parsed = y_unparsed
                    .parse::<usize>()
                    .expect("position y should be a usize");
                y_parsed
            } else {
                panic!("a figure should be described by 4, space-separated values");
            };

            if WIDTH <= x {
                panic!("x coordinate of figure should be less then the width of the board");
            }

            if HEIGHT <= y {
                panic!("y coordinate of figure should be less then the height of the board");
            }

            result.push(Figure {
                side,
                kind,
                board_position: BoardPosition { x, y },
            });
        }

        return result;
    }
}

// COMPONENTS

#[derive(Component)]
struct Board<const WIDTH: usize, const HEIGHT: usize> {
    structure: [[u8; WIDTH]; HEIGHT],
    starting_position: Vec<Figure>,
    field_size: f32,
    border_width: f32,
    outer_border_width: f32,
    upper_left_field_position: Vec2,
}

impl<const WIDTH: usize, const HEIGHT: usize> Board<WIDTH, HEIGHT> {
    fn new(data: &BoardData<WIDTH, HEIGHT>, upper_left: Vec2) -> Self {
        return Self {
            structure: data.structure,
            starting_position: data.starting_position.to_vec(),
            field_size: data.field_size,
            border_width: data.border_width,
            outer_border_width: data.outer_border_width,
            upper_left_field_position: upper_left,
        };
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Component)]
struct Figure {
    side: Side,
    kind: FigureKind,
    board_position: BoardPosition,
}

// EVENTS

#[derive(Event)]
struct CreateBoardEvent(Entity);

// SYSTEMS

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_board<const WIDTH: usize, const HEIGHT: usize>(
    board_data: Res<BoardData<WIDTH, HEIGHT>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut event: EventWriter<CreateBoardEvent>,
) {
    let width = WIDTH as f32;
    let height = HEIGHT as f32;

    let field_size = board_data.field_size;
    let border_width = board_data.border_width;
    let outer_border_width = board_data.outer_border_width;

    let field_offset = field_size + border_width;

    let total_width = field_offset * width + border_width;
    let total_height = field_offset * height + border_width;

    let offset_x = -(total_width / 2. - field_offset / 2.);
    let offset_y = total_height / 2. - field_offset / 2.;

    let background = {
        let size_x = total_width - 2. * border_width + 2. * outer_border_width;
        let size_y = total_height - 2. * border_width + 2. * outer_border_width;

        commands
            .spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(size_x, size_y))),
                material: materials.add(Color::rgb(0., 0., 0.)),
                transform: Transform::from_xyz(0., 0., 0.),
                ..default()
            })
            .id()
    };

    let fields = {
        let mut result: Vec<Entity> = vec![];

        let color_map: HashMap<u8, Handle<ColorMaterial>> = board_data
            .colors
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
                    .spawn(MaterialMesh2dBundle {
                        mesh: field.clone(),
                        material: color,
                        transform: Transform::from_xyz(
                            offset_x + j * field_offset,
                            offset_y - i * field_offset,
                            1.0,
                        ),
                        ..default()
                    })
                    .id();

                result.push(field);
            }
        }

        result
    };

    let board = {
        let upper_left = Vec2 {
            x: offset_x,
            y: offset_y,
        };

        commands
            .spawn((
                SpatialBundle::default(),
                Board::<WIDTH, HEIGHT>::new(&board_data, upper_left),
            ))
            .id()
    };

    // add all fields and the background as child entities to the board
    let mut board_entity_commands = commands.entity(board);
    board_entity_commands.push_children(&[background]);
    board_entity_commands.push_children(&fields[..]);

    // send event so figures for this board can be spawned through spawn_figures
    event.send(CreateBoardEvent(board));
}

fn setup_board<const WIDTH: usize, const HEIGHT: usize>(
    query: Query<&Board<WIDTH, HEIGHT>>,
    figure_data: Res<FigureData>,
    mut event: EventReader<CreateBoardEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for CreateBoardEvent(entity) in event.read() {
        let board = query.get(*entity).unwrap();

        let upper_left = board.upper_left_field_position;
        let field_offset = board.field_size + board.border_width;

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

        for figure in &board.starting_position {
            let x = figure.board_position.x as f32;
            let y = figure.board_position.y as f32;

            let mesh = mesh_map.get(&figure.kind).unwrap().clone();
            let material = material_map.get(&figure.side).unwrap().clone();

            commands.spawn(MaterialMesh2dBundle {
                mesh,
                material,
                transform: Transform::from_xyz(
                    upper_left.x + x * field_offset,
                    upper_left.y - y * field_offset,
                    2.,
                ),
                ..default()
            });
        }
    }
}
