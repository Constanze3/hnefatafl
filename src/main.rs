use core::panic;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
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

const ARRANGEMENT: &'static str = "
0 5 1
";

fn main() {
    let structure = Board::parse_structure(BOARD);
    let board = Board::<11, 11> {
        structure,
        field_size: 50.,
        border: 4.,
        padding: 8.,
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(board)
        .insert_resource(Arrangement::from_data(ARRANGEMENT))
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .add_systems(
            Startup,
            (setup, create_board::<11, 11>, spawn_figures::<11, 11>),
        )
        .run();
}

#[derive(Resource)]
struct Board<const WIDTH: usize, const HEIGHT: usize> {
    structure: [[u8; WIDTH]; HEIGHT],
    field_size: f32,
    border: f32,
    padding: f32,
}

impl<const WIDTH: usize, const HEIGHT: usize> Board<WIDTH, HEIGHT> {
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
}

struct BoardPosition {
    x: usize,
    y: usize,
}

#[derive(Debug)]
enum FigureType {
    King,
    Soldier,
}

#[derive(Component)]
struct Figure {
    kind: FigureType,
}

#[derive(Resource)]
struct Arrangement {
    data: Vec<(FigureType, BoardPosition)>,
}

impl Arrangement {
    fn from_data(data: &str) -> Self {
        let mut arrangement: Vec<(FigureType, BoardPosition)> = vec![];

        for line in data.lines() {
            if line == "" {
                continue;
            }

            let mut token_iter = line.split(' ');

            let kind = if let Some(kind_unparsed) = token_iter.next() {
                let kind_parsed = kind_unparsed
                    .parse::<u8>()
                    .expect("kind described by data should be either 0 or 1");

                match kind_parsed {
                    0 => FigureType::King,
                    1 => FigureType::Soldier,
                    _ => panic!("kind described by data should be either 0 or 1"),
                }
            } else {
                panic!("a figure should be described by 3, space-separated values");
            };

            let x = if let Some(x_unparsed) = token_iter.next() {
                let x_parsed = x_unparsed
                    .parse::<usize>()
                    .expect("position x should be a usize");
                x_parsed
            } else {
                panic!("a figure should be described by 3, space-separated values");
            };

            let y = if let Some(y_unparsed) = token_iter.next() {
                let y_parsed = y_unparsed
                    .parse::<usize>()
                    .expect("position y should be a usize");
                y_parsed
            } else {
                panic!("a figure should be described by 3, space-separated values");
            };

            arrangement.push((kind, BoardPosition { x, y }));
        }

        return Arrangement { data: arrangement };
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn create_board<const WIDTH: usize, const HEIGHT: usize>(
    board: Res<Board<WIDTH, HEIGHT>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let width = WIDTH as f32;
    let height = HEIGHT as f32;

    let size = board.field_size;
    let border = board.border;
    let padding = board.padding;

    let offset = -(((size + border) * width + border) / 2.);

    let background_size = (size + border) * width + border + 2. * padding;
    let background = Mesh2dHandle(meshes.add(Rectangle::new(background_size, background_size)));
    commands.spawn(MaterialMesh2dBundle {
        mesh: background,
        material: materials.add(Color::rgb(0., 0., 0.)),
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });

    let square = Mesh2dHandle(meshes.add(Rectangle::new(size, size)));
    for (i, row) in board.structure.iter().enumerate() {
        for (j, element) in row.iter().enumerate() {
            let color = match element {
                0 => Color::rgb(0.7, 0.7, 0.7),
                1 => Color::rgb(0.3, 0.4, 1.0),
                2 => Color::rgb(0.2, 0.2, 0.7),
                3 => Color::rgb(0.4, 0.2, 0.2),
                4 => Color::rgb(0.6, 0.2, 0.2),
                _ => panic!("no color assoicated with the provided element"),
            };

            let i = i as f32;
            let j = j as f32;

            commands.spawn(MaterialMesh2dBundle {
                mesh: square.clone(),
                material: materials.add(color),
                transform: Transform::from_xyz(
                    offset + j * (size + border) + border + size / 2.,
                    offset + (height - i) * (size + border) - size / 2.,
                    1.0,
                ),
                ..default()
            });
        }
    }
}

fn spawn_figures<const WIDTH: usize, const HEIGHT: usize>(
    arrangement: Res<Arrangement>,
    board: Res<Board<WIDTH, HEIGHT>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let circle = Mesh2dHandle(meshes.add(Circle::new(board.field_size / 2. - 0.1)));

    for figure in &arrangement.data {
        let x = figure.1.x as f32;
        let y = figure.1.y as f32;

        commands.spawn(MaterialMesh2dBundle {
            mesh: circle.clone(),
            material: materials.add(Color::rgb(1., 1., 1.)),
            transform: Transform::from_xyz(x * 10., y * 10., 2.),
            ..default()
        });
        print!("{:?}", figure.0);
    }
}
