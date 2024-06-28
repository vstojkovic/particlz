use super::grid::Grid;
use super::{
    BeamTargetKind, Board, BoardCoords, Border, Direction, GridMap, GridSet, Manipulator, Piece,
    Tile, TileKind, Tint,
};

#[derive(Clone)]
pub struct MoveSolver<'b> {
    board: &'b Board,
    leader: BoardCoords,
    graph: GridMap<u8>,
}

impl<'b> MoveSolver<'b> {
    pub fn new(board: &'b Board, leader: BoardCoords) -> Self {
        let mut graph = GridMap::like(&board.pieces);
        gather(board, leader, &mut graph, &mut GridSet::like(&board.pieces));
        Self {
            board,
            leader,
            graph,
        }
    }

    pub fn can_move(mut self, direction: Direction) -> bool {
        self.prune(direction, Some(self.leader));
        self.graph.get(self.leader).is_some()
    }

    pub fn drag(mut self, direction: Direction) -> GridSet {
        self.prune(direction, None);

        let mut result = GridSet::like(&self.graph);
        for (coords, _) in self.graph.iter() {
            result.insert(coords);
        }
        result
    }

    fn prune(&mut self, drag_direction: Direction, stop_coords: Option<BoardCoords>) {
        let mut pruned = true;
        while pruned {
            pruned = false;
            for coords in self.graph.dims().iter() {
                let Some(&ref_count) = self.graph.get(coords) else {
                    continue;
                };
                if (ref_count == 0) || self.should_prune(coords, drag_direction) {
                    self.graph.set(coords, None);
                    if stop_coords == Some(coords) {
                        return;
                    }
                    if let Some(manipulator) = get_manipulator(self.board, coords) {
                        for target in manipulator.iter_targets() {
                            if target.kind == BeamTargetKind::Piece {
                                if let Some(target_ref_count) = self.graph.get_mut(target.coords) {
                                    *target_ref_count -= 1;
                                }
                            }
                        }
                    }
                    pruned = true;
                }
            }
        }
    }

    fn should_prune(&self, coords: BoardCoords, drag_direction: Direction) -> bool {
        if self.get_border(coords, drag_direction).is_some() {
            return true;
        }
        let Some(neighbor) = self.board.neighbor(coords, drag_direction) else {
            return true;
        };
        if let Some(Piece::Particle(particle)) = self.board.pieces.get(coords) {
            if let Some(tile) = self.board.tiles.get(neighbor) {
                if (tile.tint != Tint::White) && (tile.tint != particle.tint) {
                    return true;
                }
            }
            if let Some(Tile {
                kind: TileKind::Collector,
                ..
            }) = self.board.tiles.get(coords)
            {
                return true;
            }
        }
        if self.board.pieces.get(neighbor).is_none() {
            return false;
        }
        self.graph.get(neighbor).is_none()
    }

    fn get_border(&self, piece_coords: BoardCoords, direction: Direction) -> Option<&Border> {
        let border_coords = piece_coords.to_border_coords(direction);
        let border_orientation = direction.orientation().flip();
        self.board.borders(border_orientation).get(border_coords)
    }
}

fn gather(board: &Board, coords: BoardCoords, graph: &mut GridMap<u8>, visited: &mut GridSet) {
    if let Some(ref_count) = graph.get_mut(coords) {
        *ref_count += 1;
    } else {
        graph.set(coords, 1);
    }

    if visited.contains(coords) {
        return;
    }
    let mut visited = visited.scoped_insert(coords);

    if let Some(manipulator) = get_manipulator(board, coords) {
        for target in manipulator.iter_targets() {
            if target.kind == BeamTargetKind::Piece {
                gather(board, target.coords, graph, &mut visited);
            }
        }
    }
}

fn get_manipulator(board: &Board, coords: BoardCoords) -> Option<&Manipulator> {
    board
        .pieces
        .get(coords)
        .and_then(|piece| piece.as_manipulator())
}

#[cfg(test)]
mod tests {
    use crate::model::{Emitters, Particle, Tile, TileKind, Tint};

    use super::*;

    #[test]
    fn cycles() {
        let mut board = empty_board(4, 4);
        add_manipulator(&mut board, (1, 1).into(), Emitters::RightDown);
        add_manipulator(&mut board, (1, 2).into(), Emitters::LeftDown);
        add_manipulator(&mut board, (2, 1).into(), Emitters::RightUp);
        add_manipulator(&mut board, (2, 2).into(), Emitters::LeftUp);
        board.retarget_beams();

        let solver = MoveSolver::new(&board, (1, 1).into());
        assert!(solver.clone().can_move(Direction::Up));
        assert!(solver.clone().can_move(Direction::Left));
        assert!(solver.clone().can_move(Direction::Down));
        assert!(solver.clone().can_move(Direction::Right));
    }

    #[test]
    fn tint_mismatch() {
        let mut board = empty_board(1, 3);
        add_manipulator(&mut board, (0, 0).into(), Emitters::Right);
        board.pieces.set((0, 1).into(), Particle::new(Tint::Green));
        add_tile(&mut board, (0, 2).into(), TileKind::Platform, Tint::Red);
        board.retarget_beams();

        assert!(!MoveSolver::new(&board, (0, 0).into()).can_move(Direction::Right));
    }

    #[test]
    fn collected_particles() {
        let mut board = empty_board(1, 3);
        add_manipulator(&mut board, (0, 0).into(), Emitters::Right);
        board.pieces.set((0, 1).into(), Particle::new(Tint::Green));
        add_tile(&mut board, (0, 1).into(), TileKind::Collector, Tint::White);
        board.retarget_beams();

        assert!(!MoveSolver::new(&board, (0, 0).into()).can_move(Direction::Right));
    }

    #[test]
    fn smoke_test() {
        let mut board = empty_board(5, 6);
        add_manipulator(&mut board, (1, 1).into(), Emitters::Right);
        board.pieces.set((1, 2).into(), Particle::new(Tint::Green));
        board.pieces.set((1, 3).into(), Particle::new(Tint::Green));
        add_manipulator(&mut board, (2, 1).into(), Emitters::Up);
        add_manipulator(&mut board, (2, 2).into(), Emitters::RightDown);
        add_manipulator(&mut board, (2, 3).into(), Emitters::RightUp);
        board.pieces.set((2, 4).into(), Particle::new(Tint::Green));
        add_manipulator(&mut board, (3, 1).into(), Emitters::Up);
        add_manipulator(&mut board, (3, 2).into(), Emitters::LeftRight);
        add_manipulator(&mut board, (3, 4).into(), Emitters::Up);
        board.horz_borders.set((1, 3).into(), Border::Wall);
        board.horz_borders.set((3, 4).into(), Border::Window);
        board.retarget_beams();

        let set = MoveSolver::new(&board, (2, 2).into()).drag(Direction::Up);
        assert!(set.contains((1, 1).into()));
        assert!(set.contains((1, 2).into()));
        assert!(!set.contains((1, 3).into()));
        assert!(set.contains((2, 1).into()));
        assert!(set.contains((2, 2).into()));
        assert!(!set.contains((2, 3).into()));
        assert!(!set.contains((2, 4).into()));
        assert!(set.contains((3, 1).into()));
        assert!(set.contains((3, 2).into()));
        assert!(!set.contains((3, 4).into()));
    }

    fn empty_board(rows: usize, cols: usize) -> Board {
        let mut board = Board::new(rows, cols);
        for coords in board.dims.iter() {
            add_tile(&mut board, coords, TileKind::Platform, Tint::White);
        }
        board
    }

    fn add_tile(board: &mut Board, coords: BoardCoords, kind: TileKind, tint: Tint) {
        board.tiles.set(coords, Tile::new(kind, tint));
    }

    fn add_manipulator(board: &mut Board, coords: BoardCoords, emitters: Emitters) {
        board.pieces.set(coords, Manipulator::new(emitters));
    }
}
