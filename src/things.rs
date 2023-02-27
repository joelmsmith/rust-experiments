#[repr(u8)]
#[derive(Clone,Copy,PartialEq)]
pub enum Rank { Rank1, Rank2, Rank3, Rank4, Rank5, Rank6, Rank7, Rank8 }
pub const NUM_RANKS: usize = 8;

#[repr(u8)]
#[derive(Clone,Copy,PartialEq)]
pub enum File { FileA, FileB, FileC, FileD, FileE, FileF, FileG, FileH }
pub const NUM_FILES: usize = 8;

#[repr(u8)]
#[derive(Clone,Copy,PartialEq)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
    NoPiece = 255
}
pub const NUM_PIECES: usize = 6;

#[repr(u8)]
#[derive(Clone,Copy,PartialEq)]
pub enum Color {
    White,
    Black,
    NoColor = 255
}
pub const NUM_COLORS: usize = 2;

pub use Piece::*;
pub use Color::*;
pub use File::*;
pub use Rank::*;

pub fn piece_from_char(c: char) -> Piece {
    match c.to_ascii_lowercase() {
        'p' => Pawn,
        'n' => Knight,
        'b' => Bishop,
        'r' => Rook,
        'q' => Queen,
        'k' => King,
        _ => panic!("bogus piece"),
    }
}

pub fn piece_to_str(piece: Piece, color: Color) -> String {
    let p = match piece {
        Pawn => 'P',
        Knight => 'N',
        Bishop => 'B',
        Rook => 'R',
        Queen => 'Q',
        King => 'K',
        _ => ' ',
    };
    match color {
        Black => p.to_lowercase().to_string(),
        _ => p.to_string(),
    }
}

pub fn color_from_char(c: char) -> Color {
    match c.is_ascii_uppercase() {
        true => White,
        false => Black,
    }
}

pub const WHITE_OO: u8 = 1;
pub const WHITE_OOO: u8 = 2;
pub const BLACK_OO: u8 = 4;
pub const BLACK_OOO: u8 = 8;