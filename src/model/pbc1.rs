use base64::Engine;
use bitter::{BitReader, LittleEndianReader};
use thiserror::Error;

use super::grid::GridMap;
use super::{
    Board, BoardCoords, Border, Dimensions, Emitters, Manipulator, Particle, Piece, Tile, TileKind,
    Tint,
};

#[derive(Error, Debug)]
pub enum Pbc1DecodeError {
    #[error("not a PBC1 code")]
    Signature,

    #[error("invalid base64 encoding")]
    Base64(#[from] base64::DecodeError),

    #[error("expected more data")]
    UnexpectedEnd,

    #[error("invalid version {0}, expected 1")]
    Version(u8),

    #[error("invalid piece value {0}")]
    InvalidPiece(u8),

    #[error("invalid border value {0}")]
    InvalidBorder(u8),
}

pub fn decode(code: &str) -> Result<Board, Pbc1DecodeError> {
    if !code.starts_with(":PBC1:") {
        return Err(Pbc1DecodeError::Signature);
    }

    let bytes = base64::engine::general_purpose::STANDARD.decode(&code[6..])?;
    let mut bits = LittleEndianReader::new(&bytes);

    let version = bits.read_bits(4).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;
    if version != 1 {
        return Err(Pbc1DecodeError::Version(version));
    }

    let _flags = bits.read_bits(4).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;
    let cols = bits.read_bits(4).ok_or(Pbc1DecodeError::UnexpectedEnd)? as usize;
    let rows = bits.read_bits(4).ok_or(Pbc1DecodeError::UnexpectedEnd)? as usize;

    let dims = Dimensions::new(rows, cols);
    let mut tiles = GridMap::new(rows, cols);
    let mut horz_borders = GridMap::new(rows + 1, cols);
    let mut vert_borders = GridMap::new(rows, cols + 1);
    let mut pieces = GridMap::new(rows, cols);

    for row in 0..rows {
        for col in 0..cols {
            let coords = BoardCoords::new(row, col);
            let flags = bits.read_bits(3).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;

            if (flags & 1) != 0 {
                let tile = bits.read_bits(3).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;
                let kind = TileKind::from_repr(tile >> 2).unwrap();
                let tint = Tint::from_repr(tile & 3).unwrap();
                tiles.set(coords, Tile::new(kind, tint));
            }

            if (flags & 2) != 0 {
                let piece = bits.read_bits(4).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;
                if piece < 3 {
                    let tint = Tint::from_repr(piece + 1).unwrap();
                    pieces.set(coords, Piece::Particle(Particle::new(tint)));
                } else if piece < 13 {
                    let emitters = Emitters::from_repr(piece - 3).unwrap();
                    pieces.set(coords, Piece::Manipulator(Manipulator::new(emitters)));
                } else {
                    return Err(Pbc1DecodeError::InvalidPiece(piece));
                }
            }

            if (flags & 4) != 0 {
                let borders = bits.read_bits(3).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8 + 1;
                let horz = match borders % 3 {
                    0 => None,
                    1 => Some(Border::Wall),
                    2 => Some(Border::Window),
                    _ => unreachable!(),
                };
                horz_borders.set(coords, horz);
                let vert = match borders / 3 {
                    0 => None,
                    1 => Some(Border::Wall),
                    2 => Some(Border::Window),
                    _ => return Err(Pbc1DecodeError::InvalidBorder(borders)),
                };
                vert_borders.set(coords, vert);
            }
        }
        if bits.read_bit().ok_or(Pbc1DecodeError::UnexpectedEnd)? {
            vert_borders.set((row, cols).into(), Border::Wall);
        }
    }
    for col in 0..cols {
        if bits.read_bit().ok_or(Pbc1DecodeError::UnexpectedEnd)? {
            horz_borders.set((rows, col).into(), Border::Wall);
        }
    }

    let mut board = Board {
        dims,
        tiles,
        horz_borders,
        vert_borders,
        pieces,
    };
    board.retarget_beams();

    Ok(board)
}
