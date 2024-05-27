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

//fn to_board(data: &BoardData, upper_left: Vec2) -> Board {
//    let mut throne_position: Option<BoardPosition> = None;
//    let mut end_positions: Vec<BoardPosition> = vec![];
//    for (y, row) in data.structure.iter().enumerate() {
//        for (x, element) in row.iter().enumerate() {
//            let position = BoardPosition { x, y };
//
//            match *element {
//                1 => throne_position = Some(position),
//                4 => end_positions.push(position),
//                _ => {}
//            }
//        }
//    }
//
//    let throne_position = throne_position.expect("there should be a throne position");
//
//    Self {
//        rows: data.rows,
//        cols: data.cols,
//        starting_position: data.starting_position.to_vec(),
//        throne_position,
//        end_positions,
//        figures: HashMap::new(),
//
//        field_size: data.field_size,
//        border_width: data.border_width,
//        outer_border_width: data.outer_border_width,
//
//        field_offset: data.field_size + data.border_width,
//        upper_left_field_position: upper_left,
//        upper_left_corner_position: Vec2 {
//            x: upper_left.x - data.field_size / 2.,
//            y: upper_left.y + data.field_size / 2.,
//        },
//
//        width: data.cols as f32 * (data.field_size + data.border_width),
//        height: data.rows as f32 * (data.field_size + data.border_width),
//
//        figure_display_z: data.figure_display_z,
//    }
//}

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
