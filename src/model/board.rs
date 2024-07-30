use enumset::EnumSet;
use strum::IntoEnumIterator;

use super::grid::{GridMap, GridSet};
use super::movement::MoveSolver;
use super::pbc1::Pbc1DecodeError;
use super::{BeamTarget, BoardCoords, Border, Dimensions, Direction, Orientation, Piece, Tile};

#[derive(Clone)]
pub struct Board {
    pub dims: Dimensions,
    pub tiles: GridMap<Tile>,
    pub horz_borders: GridMap<Border>,
    pub vert_borders: GridMap<Border>,
    pub pieces: GridMap<Piece>,
}

impl Board {
    #[cfg(test)]
    pub fn new(rows: usize, cols: usize) -> Self {
        let dims = Dimensions::new(rows, cols);
        let tiles = GridMap::new(rows, cols);
        let horz_borders = GridMap::new(rows + 1, cols);
        let vert_borders = GridMap::new(rows, cols + 1);
        let pieces = GridMap::new(rows, cols);

        Self {
            dims,
            tiles,
            horz_borders,
            vert_borders,
            pieces,
        }
    }

    pub fn from_pbc1(code: &str) -> Result<Self, Pbc1DecodeError> {
        super::pbc1::decode(code)
    }

    pub fn copy_state_from(&mut self, other: &Self) {
        assert_eq!(self.dims.rows, other.dims.rows);
        assert_eq!(self.dims.cols, other.dims.cols);

        self.tiles.mirror(&other.tiles);
        self.horz_borders.mirror(&other.horz_borders);
        self.vert_borders.mirror(&other.vert_borders);
        self.pieces.mirror(&other.pieces);
    }

    pub fn neighbor(&self, coords: BoardCoords, direction: Direction) -> Option<BoardCoords> {
        match direction {
            Direction::Up => coords
                .row
                .checked_add_signed(-1)
                .map(|row| (row, coords.col).into()),
            Direction::Left => coords
                .col
                .checked_add_signed(-1)
                .map(|col| (coords.row, col).into()),
            Direction::Down => Some(coords.row + 1)
                .filter(|&row| row < self.dims.rows)
                .map(|row| (row, coords.col).into()),
            Direction::Right => Some(coords.col + 1)
                .filter(|&col| col < self.dims.cols)
                .map(|col| (coords.row, col).into()),
        }
    }

    pub fn borders(&self, orientation: Orientation) -> &GridMap<Border> {
        match orientation {
            Orientation::Horizontal => &self.horz_borders,
            Orientation::Vertical => &self.vert_borders,
        }
    }

    pub fn move_piece(&mut self, from_coords: BoardCoords, to_coords: BoardCoords) {
        let piece = self.pieces.take(from_coords);
        self.pieces.set(to_coords, piece);
    }

    pub fn move_pieces(&mut self, move_set: &GridSet, direction: Direction) {
        move_set.for_each(direction, |from_coords| {
            let to_coords = self.neighbor(from_coords, direction).unwrap();
            self.move_piece(from_coords, to_coords);
        });
    }

    pub fn retarget_beams(&mut self) {
        for coords in self.dims.iter() {
            let emitters = match self.pieces.get(coords) {
                Some(Piece::Manipulator(manipulator)) => manipulator.emitters,
                _ => continue,
            };
            for direction in emitters.directions() {
                let target = self.find_beam_target(coords, direction);
                let manipulator = self
                    .pieces
                    .get_mut(coords)
                    .unwrap()
                    .as_manipulator_mut()
                    .unwrap();
                manipulator.set_target(direction, target);
            }
        }
    }

    pub fn compute_allowed_moves(&self, coords: BoardCoords) -> EnumSet<Direction> {
        let solver = MoveSolver::new(self, coords);
        Direction::iter()
            .filter(|&direction| solver.clone().can_move(direction))
            .collect()
    }

    pub fn compute_move_set(&self, piece_coords: BoardCoords, direction: Direction) -> GridSet {
        MoveSolver::new(self, piece_coords).drag(direction)
    }

    pub fn prev_manipulator(&self, coords: Option<BoardCoords>) -> Option<BoardCoords> {
        // NOTE: An active board should never have 0 manipulators
        let mut coords = coords.unwrap_or_default();
        let mut remaining = self.dims.rows * self.dims.cols;
        while remaining > 0 {
            if coords.col > 0 {
                coords.col -= 1;
            } else {
                coords.col = self.dims.cols - 1;
                if coords.row > 0 {
                    coords.row -= 1;
                } else {
                    coords.row = self.dims.rows - 1;
                }
            }
            if let Some(Piece::Manipulator(_)) = self.pieces.get(coords) {
                return Some(coords);
            }
            remaining -= 1;
        }
        None
    }

    pub fn next_manipulator(&self, coords: Option<BoardCoords>) -> Option<BoardCoords> {
        // NOTE: An active board should never have 0 manipulators
        let max_row = self.dims.rows - 1;
        let max_col = self.dims.cols - 1;
        let mut coords = coords.unwrap_or_else(|| BoardCoords::new(max_row, max_col));
        let mut remaining = self.dims.rows * self.dims.cols;
        while remaining > 0 {
            if coords.col < max_col {
                coords.col += 1;
            } else {
                coords.col = 0;
                if coords.row < max_row {
                    coords.row += 1;
                } else {
                    coords.row = 0;
                }
            }
            if let Some(Piece::Manipulator(_)) = self.pieces.get(coords) {
                return Some(coords);
            }
            remaining -= 1;
        }
        None
    }

    pub fn unsupported_pieces(&self) -> GridSet {
        super::support::unsupported_pieces(self)
    }

    pub fn remove_piece(&mut self, coords: BoardCoords) {
        self.pieces.take(coords);
    }

    fn find_beam_target(&self, coords: BoardCoords, direction: Direction) -> BeamTarget {
        let mut piece_coords = coords;
        let border_orientation = direction.orientation().flip();

        loop {
            let border_coords = piece_coords.to_border_coords(direction);
            if let Some(Border::Wall) = self.borders(border_orientation).get(border_coords) {
                return BeamTarget::border(border_coords);
            }
            piece_coords = match self.neighbor(piece_coords, direction) {
                Some(neighbor) => neighbor,
                None => return BeamTarget::border(border_coords),
            };
            if self.pieces.get(piece_coords).is_some() {
                return BeamTarget::piece(piece_coords);
            }
        }
    }
}
