use crate::model::grid::GridQueue;

use super::{BeamTargetKind, Board, GridSet, Piece};

pub fn unsupported_pieces(board: &Board) -> GridSet {
    let mut unsupported = GridSet::like(&board.pieces);
    let mut support_queue = GridQueue::for_grid(&unsupported);

    for (coords, _) in board.pieces.iter() {
        unsupported.insert(coords);
        if board.tiles.get(coords).is_some() {
            support_queue.push(coords);
        }
    }

    while let Some(coords) = support_queue.pop() {
        unsupported.remove(coords);
        if let Some(Piece::Manipulator(manipulator)) = board.pieces.get(coords) {
            for target in manipulator.iter_targets() {
                if (target.kind == BeamTargetKind::Piece) && unsupported.contains(target.coords) {
                    support_queue.push(target.coords);
                }
            }
        }
    }

    unsupported
}

#[cfg(test)]
mod tests {
    use crate::model::{BoardCoords, Emitters, Manipulator, Particle, Tile, TileKind, Tint};

    use super::*;

    #[test]
    fn smoke_test() {
        let mut board = Board::new(3, 2);
        add_tile(&mut board, (1, 1).into(), TileKind::Platform, Tint::White);
        add_manipulator(&mut board, (0, 0).into(), Emitters::RightDown);
        board.pieces.set((0, 1).into(), Particle::new(Tint::Red));
        add_manipulator(&mut board, (1, 0).into(), Emitters::UpDown);
        add_manipulator(&mut board, (2, 0).into(), Emitters::RightUp);
        board.pieces.set((2, 1).into(), Particle::new(Tint::Green));

        add_manipulator(&mut board, (1, 1).into(), Emitters::Down);
        board.retarget_beams();
        let set = unsupported_pieces(&board);
        assert!(set.contains((0, 0).into()));
        assert!(set.contains((0, 1).into()));
        assert!(set.contains((1, 0).into()));
        assert!(!set.contains((1, 1).into()));
        assert!(set.contains((2, 0).into()));
        assert!(!set.contains((2, 1).into()));

        add_manipulator(&mut board, (1, 1).into(), Emitters::Left);
        board.retarget_beams();
        let set = unsupported_pieces(&board);
        assert!(!set.contains((0, 0).into()));
        assert!(!set.contains((0, 1).into()));
        assert!(!set.contains((1, 0).into()));
        assert!(!set.contains((1, 1).into()));
        assert!(!set.contains((2, 0).into()));
        assert!(!set.contains((2, 1).into()));
    }

    fn add_tile(board: &mut Board, coords: BoardCoords, kind: TileKind, tint: Tint) {
        board.tiles.set(coords, Tile::new(kind, tint));
    }

    fn add_manipulator(board: &mut Board, coords: BoardCoords, emitters: Emitters) {
        board.pieces.set(coords, Manipulator::new(emitters));
    }
}
