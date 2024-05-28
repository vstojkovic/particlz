use base64::Engine;
use bitter::{BitReader, LittleEndianReader};
use thiserror::Error;

use super::{Board, Border, Emitters, Manipulator, Particle, Piece, TileKind, Tint};

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

    let num_tiles = rows * cols;
    let mut tiles = Vec::with_capacity(num_tiles);

    let num_horz_borders = (rows + 1) * cols;
    let mut horz_borders = Vec::with_capacity(num_horz_borders);

    let num_vert_borders = rows * (cols + 1);
    let mut vert_borders = Vec::with_capacity(num_vert_borders);

    let num_pieces = num_tiles;
    let mut pieces = Vec::with_capacity(num_pieces);

    for _ in 0..rows {
        for _ in 0..cols {
            let flags = bits.read_bits(3).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;

            if (flags & 1) != 0 {
                let tile = bits.read_bits(3).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;
                let kind = TileKind::from_repr(tile >> 2).unwrap();
                let tint = Tint::from_repr(tile & 3).unwrap();
                tiles.push(Some(super::Tile::new(kind, tint)));
            } else {
                tiles.push(None);
            }

            if (flags & 2) != 0 {
                let piece = bits.read_bits(4).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8;
                if piece < 3 {
                    let tint = Tint::from_repr(piece + 1).unwrap();
                    pieces.push(Some(Piece::Particle(Particle { tint })));
                } else if piece < 13 {
                    let emitters = Emitters::from_repr(piece - 3).unwrap();
                    pieces.push(Some(Piece::Manipulator(Manipulator { emitters })));
                } else {
                    return Err(Pbc1DecodeError::InvalidPiece(piece));
                }
            } else {
                pieces.push(None);
            }

            if (flags & 4) != 0 {
                let borders = bits.read_bits(3).ok_or(Pbc1DecodeError::UnexpectedEnd)? as u8 + 1;
                let horz = match borders % 3 {
                    0 => None,
                    1 => Some(Border::Wall),
                    2 => Some(Border::Window),
                    _ => unreachable!(),
                };
                horz_borders.push(horz);
                let vert = match borders / 3 {
                    0 => None,
                    1 => Some(Border::Wall),
                    2 => Some(Border::Window),
                    _ => return Err(Pbc1DecodeError::InvalidBorder(borders)),
                };
                vert_borders.push(vert);
            } else {
                horz_borders.push(None);
                vert_borders.push(None);
            }
        }
        if bits.read_bit().ok_or(Pbc1DecodeError::UnexpectedEnd)? {
            vert_borders.push(Some(Border::Wall));
        } else {
            vert_borders.push(None);
        }
    }
    for _ in 0..cols {
        if bits.read_bit().ok_or(Pbc1DecodeError::UnexpectedEnd)? {
            horz_borders.push(Some(Border::Wall));
        } else {
            horz_borders.push(None);
        }
    }

    Ok(Board {
        rows,
        cols,
        tiles,
        horz_borders,
        vert_borders,
        pieces,
    })
}
