
pub enum Squares {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

pub const NUM_SQUARES: usize = 64;
pub const NO_SQUARE: u8 = 255;

pub fn rank_of(sq: u8) -> u8 {
    sq >> 3
}

pub fn file_of(sq: u8) -> u8 {
    sq & 7
}

pub fn make_sq(rank: u8, file: u8) -> u8 {
    (rank * 8) + file
}

pub fn sq_to_str(sq: u8) -> String {
    let mut s = String::new();
    if sq == NO_SQUARE {
        return "-".to_string();
    }
    let rank = rank_of(sq);
    let file = file_of(sq);
    s.push((file + 'a' as u8) as char);
    s.push_str(&(rank+1).to_string());
    s
}