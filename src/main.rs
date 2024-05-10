use core::panic;

use bevy::{
    math::Vec2,
    prelude::*,
    reflect::Map,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::close_on_esc,
};

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
    let board = {
        let structure = BoardData::<11, 11>::parse_structure(BOARD);
        let mut colors = HashMap::<u8, Color>::new();
        colors.insert(0, Color::rgb(0.7, 0.7, 0.7));
        colors.insert(1, Color::rgb(0.3, 0.4, 1.0));
        colors.insert(2, Color::rgb(0.2, 0.2, 0.7));
        colors.insert(3, Color::rgb(0.4, 0.2, 0.2));
        colors.insert(4, Color::rgb(0.6, 0.2, 0.2));

        let starting_position = BoardData::<11, 11>::parse_arrangement(STARTING_POSITION);

        BoardData::<11, 11> {
            structure,
            colors,
            starting_position,
            field_size: 50.,
            border_width: 4.,
            outer_border_width: 12.,
        }
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(board)
        .insert_resource(ClearColor(Color::hex("19335E").unwrap()))
        .add_systems(
            Startup,
            (
                setup,
                (create_board::<11, 11>, spawn_figures::<11, 11>).chain(),
            ),
        )
        .add_systems(Update, close_on_esc)
        .add_event::<CreateBoardEvent>()
        .run();
}

#[derive(Component)]
struct Board<const WIDTH: usize, const HEIGHT: usize> {
    structure: [[u8; WIDTH]; HEIGHT],
    starting_position: Vec<Figure>,
    field_size: f32,
    border_width: f32,
    outer_border_width: f32,
    upper_left_field_position: Vec2,
}

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
    shapes: HashMap<FigureType, Color>,
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
                    "k" => FigureType::King,
                    "s" => FigureType::Soldier,
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

#[derive(Debug, Clone)]
enum FigureType {
    King,
    Soldier,
}

#[derive(Debug, Clone)]
enum Side {
    Attacker,
    Defender,
}

#[derive(Debug, Clone)]
struct BoardPosition {
    x: usize,
    y: usize,
}

#[derive(Clone, Component)]
struct Figure {
    side: Side,
    kind: FigureType,
    board_position: BoardPosition,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Event)]
struct CreateBoardEvent(Entity);

fn create_board<const WIDTH: usize, const HEIGHT: usize>(
    board_data: Res<BoardData<WIDTH, HEIGHT>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut event: EventWriter<CreateBoardEvent>,
) {
    let width = WIDTH as f32;

    let size = board_data.field_size;
    let border = board_data.border_width;
    let padding = board_data.outer_border_width;

    let offset = ((size + border) * width + border) / 2. - (border + size / 2.);

    let board = commands
        .spawn((
            SpatialBundle::default(),
            Board::<WIDTH, HEIGHT> {
                structure: board_data.structure,
                starting_position: board_data.starting_position.to_vec(),
                field_size: board_data.field_size,
                border_width: board_data.border_width,
                outer_border_width: board_data.outer_border_width,
                upper_left_field_position: Vec2 {
                    x: -offset,
                    y: offset,
                },
            },
        ))
        .id();

    let background_size = (size + border) * width - border + 2. * padding;
    let background = commands
        .spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(background_size, background_size))),
            material: materials.add(Color::rgb(0., 0., 0.)),
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        .id();

    let colors: HashMap<u8, Handle<ColorMaterial>> = board_data
        .colors
        .iter()
        .map(|(key, color)| (*key, materials.add(*color)))
        .collect();

    let mut fields: Vec<Entity> = vec![];

    let square = Mesh2dHandle(meshes.add(Rectangle::new(size, size)));
    for (i, row) in board_data.structure.iter().enumerate() {
        for (j, element) in row.iter().enumerate() {
            let color = colors
                .get(element)
                .expect("no color assoicated with the provided element")
                .clone();

            let i = i as f32;
            let j = j as f32;

            let field = commands
                .spawn(MaterialMesh2dBundle {
                    mesh: square.clone(),
                    material: color,
                    transform: Transform::from_xyz(
                        -offset + j * (size + border),
                        offset - i * (size + border),
                        1.0,
                    ),
                    ..default()
                })
                .id();

            fields.push(field);
        }
    }

    // add all fields and the background as child entities to the board
    let mut board_entity_commands = commands.entity(board);
    board_entity_commands.push_children(&[background]);
    board_entity_commands.push_children(&fields[..]);

    event.send(CreateBoardEvent(board));
}

fn spawn_figures<const WIDTH: usize, const HEIGHT: usize>(
    query: Query<&Board<WIDTH, HEIGHT>>,
    mut event: EventReader<CreateBoardEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for CreateBoardEvent(entity) in event.read() {
        let board = query.get(*entity).unwrap();

        let upper_left = board.upper_left_field_position;
        let offset = board.field_size + board.border_width;

        let square_size = board.field_size * 0.65;
        let square = Mesh2dHandle(meshes.add(Rectangle::new(square_size, square_size)));

        let circle = Mesh2dHandle(meshes.add(Circle::new(board.field_size * 0.4)));

        for figure in &board.starting_position {
            let x = figure.board_position.x as f32;
            let y = figure.board_position.y as f32;

            let color = match &figure.side {
                Side::Attacker => Color::rgb(0., 0., 0.),
                Side::Defender => Color::rgb(1., 1., 1.),
            };

            let shape = match &figure.kind {
                FigureType::Soldier => square.clone(),
                FigureType::King => circle.clone(),
            };

            commands.spawn(MaterialMesh2dBundle {
                mesh: shape,
                material: materials.add(color),
                transform: Transform::from_xyz(
                    upper_left.x + x * offset,
                    upper_left.y - y * offset,
                    2.,
                ),
                ..default()
            });
        }
    }
}
