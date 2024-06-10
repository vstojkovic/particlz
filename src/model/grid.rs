use smallvec::{smallvec, SmallVec};

use super::{BoardCoords, Dimensions, MAX_BOARD_COLS, MAX_BOARD_ROWS};

const MAX_CAPACITY: usize = (MAX_BOARD_ROWS + 1) * (MAX_BOARD_COLS * 1);

#[derive(Clone)]
pub struct GridMap<T: Clone> {
    dims: Dimensions,
    cells: SmallVec<[Option<T>; MAX_CAPACITY]>,
}

impl<T: Clone> GridMap<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        let dims = Dimensions::new(rows, cols);
        let cells = smallvec![None; rows * cols];
        Self { dims, cells }
    }

    pub fn like<V: Clone>(other: &GridMap<V>) -> Self {
        Self::new(other.dims.rows, other.dims.cols)
    }

    pub fn get(&self, coords: BoardCoords) -> Option<&T> {
        self.cells[self.dims.index(coords)].as_ref()
    }

    pub fn get_mut(&mut self, coords: BoardCoords) -> Option<&mut T> {
        self.cells[self.dims.index(coords)].as_mut()
    }

    pub fn set<V: Into<Option<T>>>(&mut self, coords: BoardCoords, value: V) {
        self.cells[self.dims.index(coords)] = value.into();
    }

    pub fn take(&mut self, coords: BoardCoords) -> Option<T> {
        self.cells[self.dims.index(coords)].take()
    }

    pub fn mirror(&mut self, other: &Self) {
        assert_eq!(self.dims, other.dims);
        self.cells.clear();
        self.cells.extend(other.cells.iter().cloned());
    }

    pub fn iter(&self) -> impl Iterator<Item = (BoardCoords, &T)> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(idx, opt)| Some((idx, opt.as_ref()?)))
            .map(|(idx, value)| (self.dims.coords(idx), value))
    }
}
