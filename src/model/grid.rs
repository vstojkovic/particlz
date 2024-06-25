use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use smallvec::{smallvec, SmallVec};

use super::{BoardCoords, Dimensions, Direction, MAX_BOARD_COLS, MAX_BOARD_ROWS};

const MAX_CAPACITY: usize = (MAX_BOARD_ROWS + 1) * (MAX_BOARD_COLS * 1);

pub trait Grid {
    fn dims(&self) -> &Dimensions;
}

#[derive(Clone)]
pub struct GridMap<T: Clone> {
    dims: Dimensions,
    cells: SmallVec<[Option<T>; MAX_CAPACITY]>,
}

#[derive(Clone)]
pub struct GridSet {
    dims: Dimensions,
    masks: SmallVec<[u8; MAX_CAPACITY / 8]>,
}

pub struct ScopedInsert<'s> {
    set: &'s mut GridSet,
    coords: BoardCoords,
}

impl<T: Clone> GridMap<T> {
    pub fn new(rows: usize, cols: usize) -> Self {
        let dims = Dimensions::new(rows, cols);
        let cells = smallvec![None; rows * cols];
        Self { dims, cells }
    }

    pub fn like<G: Grid>(other: &G) -> Self {
        Self::new(other.dims().rows, other.dims().cols)
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

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (BoardCoords, &T)> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(idx, opt)| Some((idx, opt.as_ref()?)))
            .map(|(idx, value)| (self.dims.coords(idx), value))
    }
}

impl<T: Clone> Grid for GridMap<T> {
    fn dims(&self) -> &Dimensions {
        &self.dims
    }
}

impl GridSet {
    pub fn new(rows: usize, cols: usize) -> Self {
        let dims = Dimensions::new(rows, cols);
        let masks = smallvec![0; (rows * cols + 7) / 8];
        Self { dims, masks }
    }

    pub fn like<G: Grid>(other: &G) -> Self {
        Self::new(other.dims().rows, other.dims().cols)
    }

    pub fn contains(&self, coords: BoardCoords) -> bool {
        let idx = self.dims.index(coords);
        self.masks[idx / 8] & (1 << (idx % 8)) != 0
    }

    pub fn insert(&mut self, coords: BoardCoords) {
        let idx = self.dims.index(coords);
        self.masks[idx / 8] |= 1 << (idx % 8);
    }

    pub fn remove(&mut self, coords: BoardCoords) {
        let idx = self.dims.index(coords);
        self.masks[idx / 8] &= !(1 << (idx % 8));
    }

    pub fn scoped_insert(&mut self, coords: BoardCoords) -> ScopedInsert {
        ScopedInsert::new(self, coords)
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = BoardCoords> + '_ {
        self.dims.iter().filter(|&coords| self.contains(coords))
    }

    pub fn for_each(&self, direction: Direction, func: impl FnMut(BoardCoords)) {
        match direction {
            Direction::Up | Direction::Left => self.iter().for_each(func),
            Direction::Down | Direction::Right => self.iter().rev().for_each(func),
        }
    }
}

impl Grid for GridSet {
    fn dims(&self) -> &Dimensions {
        &self.dims
    }
}

impl Debug for GridSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut iter = self.iter();
        if let Some(first) = iter.next() {
            write!(f, "{:?}", first)?;
            for coords in iter {
                write!(f, ", {:?}", coords)?;
            }
        }
        write!(f, "}}")
    }
}

impl<'s> ScopedInsert<'s> {
    fn new(set: &'s mut GridSet, coords: BoardCoords) -> Self {
        set.insert(coords);
        Self { set, coords }
    }
}

impl<'s> Drop for ScopedInsert<'s> {
    fn drop(&mut self) {
        self.set.remove(self.coords);
    }
}

impl<'s> Deref for ScopedInsert<'s> {
    type Target = GridSet;
    fn deref(&self) -> &Self::Target {
        self.set
    }
}

impl<'s> DerefMut for ScopedInsert<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.set
    }
}
