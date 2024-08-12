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

#[derive(Debug, Default, Clone)]
pub struct LevelMetadata {
    pub id: Option<usize>,
    pub name: Option<String>,
    pub next: Option<usize>,
}

pub struct LevelCampaign {
    pub levels: Vec<CampaignLevel>,
    pub tiers: Vec<CampaignTier>,
}

pub struct CampaignLevel {
    pub name: String,
    pub board: Board,
}

pub struct CampaignTier {
    pub name: String,
    pub levels: Vec<usize>,
}

pub type CampaignData<'d> = &'d [(&'d str, &'d [(&'d str, &'d str)])];

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

impl LevelCampaign {
    pub fn from_static(tier_data: CampaignData) -> Self {
        let mut levels = vec![];
        let mut tiers = Vec::with_capacity(tier_data.len());

        for (name, level_data) in tier_data {
            let mut tier_levels = Vec::with_capacity(level_data.len());
            for (name, pbc) in *level_data {
                let board = Board::from_pbc1(pbc).unwrap();
                tier_levels.push(levels.len());
                levels.push(CampaignLevel {
                    name: name.to_string(),
                    board,
                });
            }
            tiers.push(CampaignTier {
                name: name.to_string(),
                levels: tier_levels,
            });
        }

        Self { levels, tiers }
    }

    pub fn metadata(&self, level_idx: usize) -> LevelMetadata {
        let next_idx = level_idx + 1;
        LevelMetadata {
            id: Some(level_idx),
            name: Some(self.levels[level_idx].name.clone()),
            next: (next_idx < self.levels.len()).then_some(next_idx),
        }
    }
}
