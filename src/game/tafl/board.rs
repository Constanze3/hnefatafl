use bevy::{prelude::*, utils::HashMap};

pub enum Axis2 {
    X,
    Y,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Component, Debug, Default, Clone)]
pub struct SelectedFigure {
    pub entity: Option<Entity>,
    pub possible_moves: Option<Vec<Position>>,
}

pub struct BoardOptions {
    pub rows: usize,
    pub cols: usize,
    pub throne_position: Position,
    pub end_positions: Vec<Position>,
    pub figures: HashMap<Position, Entity>,
    pub field_size: f32,
    pub border_width: f32,
    pub outer_border_width: f32,
    pub figures_z: f32,
}

#[derive(Component, Clone)]
pub struct Board {
    pub rows: usize,
    pub cols: usize,

    pub throne_position: Position,
    pub end_positions: Vec<Position>,

    // position - figure_entity map
    pub figures: HashMap<Position, Entity>,

    // the width/height of a field of the board
    pub field_size: f32,
    // the width of the border inbetween the fields
    pub border_width: f32,
    // the width of the outer border of the board
    pub outer_border_width: f32,
    // distance of 2 neighbor fields, field_size + border_width
    pub field_offset: f32,
    // width of the board excluding outer border
    pub width: f32,
    // height of the board exluding outer border
    pub height: f32,
    // the position of the upper-left corner of the board excluding the outer border
    pub upper_left_corner_position: Vec2,
    // the position of the field at the upper left corner
    pub upper_left_field_position: Vec2,

    // the z-axis coordinate figures displayed on the board should have
    pub figures_z: f32,
}

impl Board {
    /// Creates a new Board.
    pub fn new(options: BoardOptions) -> Self {
        let rows = options.rows;
        let cols = options.cols;

        let throne_position = options.throne_position;
        let end_positions = options.end_positions;

        let figures = options.figures;

        let field_size = options.field_size;
        let border_width = options.border_width;
        let outer_border_width = options.outer_border_width;
        let figures_z = options.figures_z;

        let field_offset = field_size + border_width;
        let width = cols as f32 * field_offset - border_width;
        let height = rows as f32 * field_offset - border_width;

        let upper_left_corner_position = Vec2 {
            x: -width / 2.,
            y: height / 2.,
        };

        let upper_left_field_position = Vec2 {
            x: upper_left_corner_position.x + field_size / 2.,
            y: upper_left_corner_position.y - field_size / 2.,
        };

        Self {
            rows,
            cols,

            throne_position,
            end_positions,

            figures,

            field_size,
            border_width,
            outer_border_width,

            field_offset,
            width,
            height,
            upper_left_corner_position,
            upper_left_field_position,

            figures_z,
        }
    }

    /// Converts a world position to the position of a field on the board.
    pub fn world_to_board(&self, position: Vec2) -> Option<Position> {
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

        Some(Position { x, y })
    }

    /// Converts a position on the board to a world position.
    pub fn board_to_world(&self, position: Position) -> Vec2 {
        let upper_left = self.upper_left_field_position;
        let field_offset = self.field_offset;

        let x = upper_left.x + position.x as f32 * field_offset;
        let y = upper_left.y - position.y as f32 * field_offset;

        Vec2 { x, y }
    }

    /// Determines whether the provided `position` is on the board or not.
    pub fn is_on_board(&self, position: Position) -> bool {
        return 0 <= position.x
            || position.x < self.cols
            || 0 <= position.y
            || position.y < self.rows;
    }

    /// Gets all neighboring figure entities of a certain BoardPosition.
    /// Neighboring means that either x +/- 1 or y +/- 1 relative to the position.
    pub fn get_neighbors(&self, position: Position) -> Vec<Entity> {
        let mut result: Vec<Entity> = vec![];

        // left
        if 0 <= position.x as isize - 1 {
            if let Some(figure_entity) = self.figures.get(&Position {
                x: position.x - 1,
                y: position.y,
            }) {
                result.push(*figure_entity);
            }
        }

        // right
        if position.x + 1 < self.rows {
            if let Some(figure_entity) = self.figures.get(&Position {
                x: position.x + 1,
                y: position.y,
            }) {
                result.push(*figure_entity);
            }
        }

        // bottom
        if 0 <= position.y as isize - 1 {
            if let Some(figure_entity) = self.figures.get(&Position {
                x: position.x,
                y: position.y - 1,
            }) {
                result.push(*figure_entity);
            }
        }

        // top
        if position.y + 1 < self.cols {
            if let Some(figure_entity) = self.figures.get(&Position {
                x: position.x,
                y: position.y + 1,
            }) {
                result.push(*figure_entity);
            }
        }

        result
    }
}
