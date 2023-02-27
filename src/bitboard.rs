use crate::attack::*;
use crate::square::*;
use crate::things::*;

static mut BB_BTWN: &'static mut [u64] = &mut [0; 64 * 64];
static mut BB_RAYS: &'static mut [u64] = &mut [0; 64 * 64];

pub const RANK_BITBOARDS: [u64; 8] = [
    0xff << 0,
    0xff << 8,
    0xff << 16,
    0xff << 24,
    0xff << 32,
    0xff << 40,
    0xff << 48,
    0xff << 56,
];
pub const FILE_BITBOARDS: [u64; 8] = [
    0x101010101010101 << 0,
    0x101010101010101 << 1,
    0x101010101010101 << 2,
    0x101010101010101 << 3,
    0x101010101010101 << 4,
    0x101010101010101 << 5,
    0x101010101010101 << 6,
    0x101010101010101 << 7,
];

pub fn init() {
    for src_sq in 0..64 {
        for dst_sq in 0..64 {
            if src_sq == dst_sq {
                continue;
            }

            let src_sq = src_sq as u8;
            let dst_sq = dst_sq as u8;
            let src_bb = bb_from_sq(src_sq);
            let dst_bb = bb_from_sq(dst_sq);
            let src_rank = rank_of(src_sq);
            let dst_rank = rank_of(dst_sq);
            let src_file = file_of(src_sq);
            let dst_file = file_of(dst_sq);
            let idx = (dst_sq as usize) + ((src_sq as usize) * 64);

            if src_rank == dst_rank || src_file == dst_file {
                // Squares between src & dst are equal to the attack overlap of
                // rooks on both squares.
                let mut between = rook_attacks(src_bb, dst_bb);
                between &= rook_attacks(dst_bb, src_bb);

                let mut ray = src_bb | dst_bb;
                ray |= rook_attacks(src_bb, 0) & rook_attacks(dst_bb, 0);

                unsafe { BB_BTWN[idx] = between };
                unsafe { BB_RAYS[idx] = ray };
            } else {
                let src_attacks = bishop_attacks(src_bb, dst_bb);
                let dst_attacks = bishop_attacks(dst_bb, src_bb);

                if src_bb & dst_attacks != 0 {
                    let mut between = src_attacks;
                    between &= dst_attacks;
                    unsafe { BB_BTWN[idx] = between };
                }

                let src_attacks = bishop_attacks(src_bb, 0) | src_bb;
                let dst_attacks = bishop_attacks(dst_bb, 0) | dst_bb;
                if src_attacks & dst_bb != 0 {
                    let mut ray = src_attacks & dst_attacks;
                    ray |= src_bb | dst_bb;
                    unsafe { BB_RAYS[idx] = ray };
                }
            }
        }
    }
}

pub fn bb_lsb(bb: u64) -> u8 {
    bb.trailing_zeros() as u8
}

pub fn bb_pop(bb: &mut u64) -> u8 {
    let r = bb_lsb(*bb);
    *bb = *bb & (*bb - 1);
    r as u8
}

pub fn bb_north(bb: u64) -> u64 {
    bb << 8
}

pub fn bb_south(bb: u64) -> u64 {
    bb >> 8
}

pub fn bb_east(bb: u64) -> u64 {
    (bb << 1) & !FILE_BITBOARDS[FileA as usize]
}

pub fn bb_west(bb: u64) -> u64 {
    (bb >> 1) & !FILE_BITBOARDS[FileH as usize]
}

pub fn bb_popcnt(bb: u64) -> u32 {
    std::primitive::u64::count_ones(bb)
}

pub fn bb_flip(bb: u64) -> u64 {
    bb.swap_bytes()
}

pub fn bb_from_sq(sq: u8) -> u64 {
    1 << sq
}

pub fn bb_between(sq1: u8, sq2: u8) -> u64 {
    let idx = (sq1 as usize) + ((sq2 as usize) * 64);
    unsafe { BB_BTWN[idx] }
}

pub fn bb_ray(sq1: u8, sq2: u8) -> u64 {
    let idx = (sq1 as usize) + ((sq2 as usize) * 64);
    unsafe { BB_RAYS[idx] }
}

pub fn bb_debug(bb: u64) {
    let mut s = String::from("\n     A   B   C   D   E   F   G   H\n");
    s.push_str("   +---+---+---+---+---+---+---+---+\n");

    for rank in (0..8).rev() {
        s.push_str(" ");
        s.push_str(&(rank + 1).to_string());
        s.push_str(" ");
        for file in 0..8 {
            s.push_str("| ");
            if (bb & (RANK_BITBOARDS[rank] & FILE_BITBOARDS[file])) != 0 {
                s.push_str("X");
            } else {
                s.push_str(" ");
            }
            s.push_str(" ");
        }
        s.push_str("|\n   +---+---+---+---+---+---+---+---+\n");
    }

    s.push_str("     A   B   C   D   E   F   G   H\n");
    println!("{} {:#x}", s, bb);
}

fn white_pawn_advances(pawn: u64, occupancy: u64) -> u64 {
    let unmoved = pawn & RANK_BITBOARDS[Rank2 as usize];
    let mut advances = bb_north(pawn);

    advances &= !occupancy;
    advances |= bb_north(bb_north(unmoved) & !occupancy) & !occupancy;
    advances
}

fn black_pawn_advances(pawn: u64, occupancy: u64) -> u64 {
    let unmoved = pawn & RANK_BITBOARDS[Rank7 as usize];
    let mut advances = bb_south(pawn);

    advances &= !occupancy;
    advances |= bb_south(bb_south(unmoved) & !occupancy) & !occupancy;
    advances
}

pub fn pawn_advances(pawn: u64, occupancy: u64, color: Color) -> u64 {
    match color {
        White => white_pawn_advances(pawn, occupancy),
        Black => black_pawn_advances(pawn, occupancy),
        _ => panic!("bogus color"),
    }
}
