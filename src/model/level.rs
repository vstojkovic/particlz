use super::{Board, Piece, Tile, TileKind};

#[derive(Debug)]
pub struct LevelProgress {
    manipulators_left: usize,
    uncollected_particles: usize,
    pub outcome: Option<LevelOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelOutcome {
    NoManipulatorsLeft,
    ParticleLost,
    Victory,
}

impl LevelProgress {
    pub fn new(board: &Board) -> Self {
        let mut manipulators_left = 0;
        let mut uncollected_particles = 0;
        for (coords, piece) in board.pieces.iter() {
            match piece {
                Piece::Particle(_) => {
                    match board.tiles.get(coords) {
                        Some(Tile {
                            kind: TileKind::Collector,
                            ..
                        }) => (),
                        _ => uncollected_particles += 1,
                    };
                }
                Piece::Manipulator(_) => manipulators_left += 1,
            }
        }
        Self {
            manipulators_left,
            uncollected_particles,
            outcome: None,
        }
    }

    pub fn particle_collected(&mut self) {
        self.uncollected_particles -= 1;
        if self.uncollected_particles == 0 {
            self.update_outcome(LevelOutcome::Victory);
        }
    }

    pub fn piece_lost(&mut self, piece: &Piece) {
        match piece {
            Piece::Particle(_) => self.update_outcome(LevelOutcome::ParticleLost),
            Piece::Manipulator(_) => self.manipulators_left -= 1,
        }
        if self.manipulators_left == 0 {
            self.update_outcome(LevelOutcome::NoManipulatorsLeft);
        }
    }

    fn update_outcome(&mut self, outcome: LevelOutcome) {
        self.outcome = self.outcome.max(Some(outcome));
    }
}
