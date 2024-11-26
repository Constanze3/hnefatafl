use std::time::Duration;

use bevy::{
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
};

use crate::game::tafl::*;

#[derive(Resource, Default)]
pub struct BoardId(SimpleId);

impl BoardId {
    fn get(&mut self) -> SimpleId {
        let result = self.0;
        self.0 .0 += 1;
        return result;
    }
}

/// System for spawning a nice looking hnefatafl board.
pub fn spawn_hnefatafl(
    mut spawn_board_event: EventWriter<SpawnBoardEvent>,
    mut spawn_figures_event: EventWriter<SpawnFiguresEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut board_id: ResMut<BoardId>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut setup_game_ui_event: EventWriter<SetupGameUiEvent>,
) {
    let id = board_id.get();

    let board = {
        let mut colors = HashMap::<u8, Handle<ColorMaterial>>::new();
        colors.insert(0, materials.add(Color::rgb_u8(130, 162, 157)));
        colors.insert(1, materials.add(Color::rgb_u8(38, 53, 75)));
        colors.insert(2, materials.add(Color::rgb_u8(78, 112, 165)));
        colors.insert(3, materials.add(Color::rgb_u8(157, 79, 79)));
        colors.insert(4, materials.add(Color::rgb_u8(55, 65, 104)));

        let structure = "\
                     40033333004\n\
                     00000300000\n\
                     00000000000\n\
                     30000200003\n\
                     30002220003\n\
                     33022122033\n\
                     30002220003\n\
                     30000200003\n\
                     00000000000\n\
                     00000300000\n\
                     40033333004";

        let parsed = parse_board(structure).unwrap();

        let field_materials: HashMap<Position, Handle<ColorMaterial>> = parsed
            .structure
            .iter()
            .map(|(key, value)| (*key, colors.get(value).unwrap().clone()))
            .collect();

        let field_size = 50.;

        let board = Board::new(BoardOptions {
            cols: parsed.cols,
            rows: parsed.rows,
            throne_position: Position { x: 5, y: 5 },
            end_positions: vec![
                Position { x: 0, y: 0 },
                Position { x: 10, y: 0 },
                Position { x: 0, y: 10 },
                Position { x: 10, y: 10 },
            ],
            figures: HashMap::new(),
            field_size,
            border_width: 4.,
            outer_border_width: 12.,
            figure_z: 3.,
        });

        spawn_board_event.send(SpawnBoardEvent {
            id,
            position: Vec3::ZERO,
            board: board.clone(),
            border_material: materials.add(Color::rgb(0., 0., 0.)),
            border_z: -1.,
            field_materials,
            highlight_mesh: meshes.add(Circle::new(0.2 * field_size)),
            highlight_material: materials.add(Color::rgba(0., 0., 0., 0.6)),
            highlight_z: 1.,
        });

        board
    };

    // Figures
    {
        let starting_position = "
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

        let figures = parse_figures(starting_position, &board).unwrap();

        let mut figure_textures = HashMap::<FigureType, Handle<Image>>::new();
        figure_textures.insert(
            FigureType {
                side: Side::Defender,
                kind: FigureKind::King,
            },
            asset_server.load("figures/defender_king.png"),
        );
        figure_textures.insert(
            FigureType {
                side: Side::Defender,
                kind: FigureKind::Soldier,
            },
            asset_server.load("figures/defender_soldier.png"),
        );
        figure_textures.insert(
            FigureType {
                side: Side::Attacker,
                kind: FigureKind::Soldier,
            },
            asset_server.load("figures/attacker_soldier.png"),
        );

        spawn_figures_event.send(SpawnFiguresEvent {
            board_id: id,
            figures,
            textures: figure_textures,
        });
    };

    // selection indicator
    {
        let mesh = Mesh2dHandle(meshes.add(Rectangle::new(board.field_size, board.field_size)));
        commands.spawn((
            MaterialMesh2dBundle {
                mesh,
                material: materials.add(Color::rgb_u8(157, 71, 162)),
                transform: Transform::from_xyz(0., 0., 2.),
                visibility: Visibility::Hidden,
                ..default()
            },
            SelectionIndicator,
        ));
    }

    setup_game_ui_event.send(SetupGameUiEvent {
        side_with_initial_turn: Side::Attacker,
        timer_duration: Duration::from_secs(600),
    });
}

struct ParsedBoard {
    rows: usize,
    cols: usize,
    structure: HashMap<Position, u8>,
}

/// Helper function for parsing boards from strings.
fn parse_board(data: &str) -> Result<ParsedBoard, &str> {
    let mut rows = 0;
    let mut cols = 0;
    let mut structure: HashMap<Position, u8> = HashMap::new();

    let mut current_row: Vec<u8> = vec![];

    fn row_complete(
        rows: &mut usize,
        cols: &mut usize,
        current_row: &mut Vec<u8>,
    ) -> Result<(), &'static str> {
        if *rows == 0 {
            *cols = current_row.len()
        } else if current_row.len() != *cols {
            return Err("data should have consistent row length");
        }

        current_row.clear();
        *rows += 1;

        Ok(())
    }

    for (i, c) in data.chars().enumerate() {
        if i == 0 && c == '\n' {
            continue;
        }

        if c != '\n' {
            if c.is_ascii_digit() {
                let digit = c as u8 - '0' as u8;

                let x = current_row.len();
                let y = rows;
                structure.insert(Position { x, y }, digit);

                current_row.push(digit);
            } else {
                return Err("data should only consist of numbers and \\n");
            }
        } else {
            row_complete(&mut rows, &mut cols, &mut current_row)?;
        }
    }

    if 0 < current_row.len() {
        row_complete(&mut rows, &mut cols, &mut current_row)?;
    }

    Ok(ParsedBoard {
        rows,
        cols,
        structure,
    })
}

fn parse_figures(data: &str, board: &Board) -> Result<Vec<Figure>, &'static str> {
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

        let position = Position { x, y };

        if !board.is_on_board(position) {
            return Err("the figure should be on the board");
        }

        result.push(Figure {
            side,
            kind,
            position,
        });
    }

    Ok(result)
}
