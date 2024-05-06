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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Board::<11, 11>::new(BOARD))
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_systems(Startup, (setup, create_board::<11, 11>))
        .run();
}

#[derive(Resource)]
struct Board<const WIDTH: usize, const HEIGHT: usize> {
    structure: [[u8; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> Board<WIDTH, HEIGHT> {
    fn new(data: &str) -> Board<WIDTH, HEIGHT> {
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

        return Board { structure };
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn create_board<const WIDTH: usize, const HEIGHT: usize>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board: Res<Board<WIDTH, HEIGHT>>,
) {
    let size = 50.;
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

            commands.spawn(MaterialMesh2dBundle {
                mesh: square.clone(),
                material: materials.add(color),
                transform: Transform::from_xyz(
                    j as f32 * size - (size * WIDTH as f32 / 2.),
                    (HEIGHT as f32 - i as f32) * size - (size * HEIGHT as f32 / 2.),
                    0.0,
                ),
                ..default()
            });
        }
    }
}
