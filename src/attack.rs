use super::bitboard::*;
use super::position::Position;
use super::square::*;
use super::things::*;

// TODO: reclaim pawn space
// TODO: make this 2D
static mut EMPTY_BOARD_ATTACKS: &'static mut [u64] = &mut [0; NUM_PIECES * NUM_SQUARES];

// This technique is used to avoid sliding rooks, bishops, and queens around
// during move generation.  Attacks for rooks & bishops are pre-generated based
// their source square and the obstacle(s) their attack ray(s) may encounter.
// This is implemented using a well-known algorithm described at:
// https://www.chessprogramming.org/Magic_Bitboards
static mut BISHOP_MASKS: &'static mut [u64] = &mut [0; 64];
static mut BISHOP_SHIFT: &'static mut [u64] = &mut [0; 64];
static mut BISHOP_ATTACK_TABLE: &'static mut [u64] = &mut [0; 5248];
static mut BISHOP_ATTACK_INDEX: &'static mut [usize] = &mut [0; 64];
static mut ROOK_MASKS: &'static mut [u64] = &mut [0; 64];
static mut ROOK_SHIFT: &'static mut [u64] = &mut [0; 64];
static mut ROOK_ATTACK_TABLE: &'static mut [u64] = &mut [0; 102400];
static mut ROOK_ATTACK_INDEX: &'static mut [usize] = &mut [0; 64];

pub const ROOK_MAGIC: [u64; NUM_SQUARES as usize] = [
    0xd080044000148022,
    0x2440002008401002,
    0x5200104008802200,
    0x200100440082200,
    0x2080040002080080,
    0x100080100040002,
    0x2080008002000100,
    0x100020126804900,
    0x1000800080400022,
    0x804000200080,
    0x682801001802000,
    0x4434800800809000,
    0x2405001100040800,
    0x300800400020080,
    0xc1000200010004,
    0xe702002090440102,
    0x848008400020,
    0x4840008080402000,
    0x1010020004010,
    0x8420010082202,
    0x406020008042010,
    0xc400808002000400,
    0x40001884210,
    0x10002002089004c,
    0x4080a180004000,
    0x810100400020,
    0x822004200201880,
    0x1000100080080080,
    0x18040080080080,
    0x1a1000900040002,
    0x2004040800100,
    0x80c802080004100,
    0x30400020801080,
    0x804000802000,
    0x1460020010100400,
    0x1240801000800800,
    0x41000801001004,
    0x4008004804200,
    0xa090020001010004,
    0x4006102001084,
    0x440410080010029,
    0x2010002000404000,
    0x2000100020008080,
    0x8001000210100,
    0x8a40080011010005,
    0x2114001008020200,
    0x2042000804020001,
    0x8000404100820004,
    0x40088008402880,
    0x2004021009200,
    0x10012004080220,
    0x8840100080080080,
    0x8000400088080,
    0xc200440100801,
    0x4800200010080,
    0x242c0100488200,
    0x20110020408009,
    0x1681004018208202,
    0x7104082000410411,
    0xc081000200501,
    0x802002008100402,
    0x2241000208140003,
    0x22090210804,
    0x2008004021040082,
];

pub const BISHOP_MAGIC: [u64; NUM_SQUARES as usize] = [
    0x710823004208010,
    0x11012124008030,
    0x2810808200485800,
    0x404441280020000,
    0x20040422a0400000,
    0x880540000240,
    0xc009109004a1040a,
    0x900c804108200300,
    0x42410a6004040051,
    0x20942c848008088,
    0x8020420092008810,
    0x71192401043000,
    0x2004020211222014,
    0x9010090000,
    0x4a12114000,
    0x49042250108608b,
    0x4020004083224202,
    0x20205782a0c00,
    0x1010002810404308,
    0x8226104010021,
    0x2894100202020012,
    0x1201004100a008,
    0x800648201242000,
    0x810a002280492880,
    0x29a00ac0240108,
    0x4600002880120,
    0x4004040802080011,
    0x2a0080001004008,
    0x6001001001004010,
    0x5010100820080c4,
    0x8191406190110,
    0xa002003200808820,
    0x46020040a0208,
    0x91082040024480,
    0x8008220121080800,
    0x8002008020020200,
    0x40004012010100,
    0x84004080041000,
    0x9104840842709800,
    0x25108108503e0a00,
    0x5084500410300500,
    0x141042120000508,
    0x4a4120110000103,
    0xa000284200892802,
    0x880100410408,
    0x4040400840a0200,
    0x281080b0800601,
    0x104414c00210101,
    0x6800808820130800,
    0x805402a00142,
    0x1005844100088,
    0x4220100208840b4,
    0x2004001202021103,
    0x100200410009822,
    0x4200444809c0161,
    0x6008884084144088,
    0x4200220820880800,
    0xc0102704109402,
    0x2002110101014112,
    0x80208040208848,
    0x1000011020220,
    0x400806082008c082,
    0x40102121440084,
    0x42810040a4410a1,
];

fn empty_board_index(piece: Piece, sq: u8) -> usize {
    ((piece as usize) * NUM_SQUARES) + sq as usize
}

fn rook_mask(sq: u8) -> u64 {
    let mut mask: u64 = 0;
    for rank in (Rank2 as u8)..(Rank8 as u8) {
        if rank == rank_of(sq) as u8 {
            continue;
        }
        let file = file_of(sq);
        mask |= RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
    }
    for file in (FileB as u8)..(FileH as u8) {
        if file == file_of(sq) as u8 {
            continue;
        }
        let rank = rank_of(sq);
        mask |= RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
    }
    mask
}

fn gen_rook_attack(sq: u8, occupancy: u64) -> u64 {
    let mut attack: u64 = 0;

    // There's a loop to slide in each direction (N/S/E/W) until another piece
    // is encountered.

    let mut rank = rank_of(sq) as i8;
    let file = file_of(sq) as i8;
    loop {
        rank += 1;
        if rank >= NUM_RANKS as i8 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if x & occupancy != 0 {
            break;
        }
    }
    let mut rank = rank_of(sq) as i8;
    let file = file_of(sq) as i8;
    loop {
        rank -= 1;
        if rank < 0 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if x & occupancy != 0 {
            break;
        }
    }
    let rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        file += 1;
        if file >= NUM_FILES as i8 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if x & occupancy != 0 {
            break;
        }
    }
    let rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        file -= 1;
        if file < 0 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if x & occupancy != 0 {
            break;
        }
    }
    attack
}

fn bishop_mask(sq: u8) -> u64 {
    let mut mask: u64 = 0;
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank += 1;
        file += 1;
        if rank >= Rank8 as i8 {
            break;
        }
        if file >= FileH as i8 {
            break;
        }
        mask |= RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
    }
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank += 1;
        file -= 1;
        if rank >= Rank8 as i8 {
            break;
        }
        if file <= FileA as i8 {
            break;
        }
        mask |= RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
    }
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank -= 1;
        file += 1;
        if rank <= Rank1 as i8 {
            break;
        }
        if file >= FileH as i8 {
            break;
        }
        mask |= RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
    }
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank -= 1;
        file -= 1;
        if rank <= Rank1 as i8 {
            break;
        }
        if file <= FileA as i8 {
            break;
        }
        mask |= RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
    }
    mask
}

fn gen_bishop_attack(sq: u8, occupancy: u64) -> u64 {
    let mut attack: u64 = 0;
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank += 1;
        file += 1;
        if rank > Rank8 as i8 {
            break;
        }
        if file > FileH as i8 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if occupancy & x != 0 {
            break;
        }
    }
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank += 1;
        file -= 1;
        if rank > Rank8 as i8 {
            break;
        }
        if file < FileA as i8 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if occupancy & x != 0 {
            break;
        }
    }
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank -= 1;
        file += 1;
        if rank < Rank1 as i8 {
            break;
        }
        if file > FileH as i8 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if occupancy & x != 0 {
            break;
        }
    }
    let mut rank = rank_of(sq) as i8;
    let mut file = file_of(sq) as i8;
    loop {
        rank -= 1;
        file -= 1;
        if rank < Rank1 as i8 {
            break;
        }
        if file < FileA as i8 {
            break;
        }
        let x = RANK_BITBOARDS[rank as usize] & FILE_BITBOARDS[file as usize];
        attack |= x;
        if occupancy & x != 0 {
            break;
        }
    }
    attack
}

fn init_rook() {
    let mut table_index: usize = 0;
    for sq in 0..NUM_SQUARES as u8 {
        let magic = ROOK_MAGIC[sq as usize];
        let mask = rook_mask(sq);
        let bits = bb_popcnt(mask) as u64;
        let n: u64 = 1 << bits;
        let shift: u64 = 64 - bits;

        unsafe { ROOK_SHIFT[sq as usize] = shift };
        unsafe { ROOK_MASKS[sq as usize] = mask };
        unsafe { ROOK_ATTACK_INDEX[sq as usize] = table_index };

        let mut variation: u64 = 0;
        for _ in 0..n {
            let attack = gen_rook_attack(sq, variation);
            let idx = table_index + (((mask & variation) * magic) >> shift) as usize;
            unsafe { ROOK_ATTACK_TABLE[idx as usize] = attack };
            variation = (variation - mask) & mask;
        }
        table_index += n as usize;
    }
}

fn init_bishop() {
    let mut table_index: usize = 0;
    for sq in 0..NUM_SQUARES as u8 {
        let magic = BISHOP_MAGIC[sq as usize];
        let mask = bishop_mask(sq);
        let bits = bb_popcnt(mask) as u64;
        let n: u64 = 1 << bits;
        let shift: u64 = 64 - bits;

        unsafe { BISHOP_SHIFT[sq as usize] = shift };
        unsafe { BISHOP_MASKS[sq as usize] = mask };
        unsafe { BISHOP_ATTACK_INDEX[sq as usize] = table_index };

        let mut variation: u64 = 0;
        for _ in 0..n {
            let attack = gen_bishop_attack(sq, variation);
            let idx = table_index + (((mask & variation) * magic) >> shift) as usize;
            unsafe { BISHOP_ATTACK_TABLE[idx as usize] = attack };
            variation = (variation - mask) & mask;
        }
        table_index += n as usize;
    }
}

pub fn init() {
    init_bishop();
    init_rook();

    for sq in 0..NUM_SQUARES {
        let bb: u64 = 1 << sq;
        let index: usize = empty_board_index(Knight, sq as u8);
        unsafe { EMPTY_BOARD_ATTACKS[index] = calc_knight_attacks(bb) };
        let index: usize = empty_board_index(Bishop, sq as u8);
        unsafe { EMPTY_BOARD_ATTACKS[index] = bishop_attacks(bb, 0) };
        let index: usize = empty_board_index(Rook, sq as u8);
        unsafe { EMPTY_BOARD_ATTACKS[index] = rook_attacks(bb, 0) };
        let index: usize = empty_board_index(Queen, sq as u8);
        unsafe { EMPTY_BOARD_ATTACKS[index] = queen_attacks(bb, 0) };
        let index: usize = empty_board_index(King, sq as u8);
        unsafe { EMPTY_BOARD_ATTACKS[index] = calc_king_attacks(bb) };
    }
}

// If there's nothing on the board but this piece, what squares is it attacking?
// Note that this is not useful for pawns.  For knights and kings, this table is
// used during move generation.
pub fn empty_board_attack(piece: Piece, sq: u8) -> u64 {
    assert!(piece != Pawn);
    let index = empty_board_index(piece, sq);
    unsafe { EMPTY_BOARD_ATTACKS[index] }
}

pub fn knight_attacks_from(sq: u8) -> u64 {
    empty_board_attack(Knight, sq)
}

pub fn king_attacks_from(sq: u8) -> u64 {
    empty_board_attack(King, sq)
}

pub fn pawn_attacks_white(pawns: u64) -> u64 {
    bb_north(bb_west(pawns)) | bb_north(bb_east(pawns))
}

pub fn pawn_attacks_black(pawns: u64) -> u64 {
    bb_south(bb_west(pawns)) | bb_south(bb_east(pawns))
}

pub fn pawn_attacks(pawns: u64, color: Color) -> u64 {
    match color {
        White => pawn_attacks_white(pawns),
        Black => pawn_attacks_black(pawns),
        _ => panic!("bogus color"),
    }
}

fn calc_knight_attacks(knights: u64) -> u64 {
    let l1 = (knights >> 1) & 0x7f7f7f7f7f7f7f7f;
    let l2 = (knights >> 2) & 0x3f3f3f3f3f3f3f3f;
    let r1 = (knights << 1) & 0xfefefefefefefefe;
    let r2 = (knights << 2) & 0xfcfcfcfcfcfcfcfc;
    let h1 = l1 | r1;
    let h2 = l2 | r2;
    (h1 << 16) | (h1 >> 16) | (h2 << 8) | (h2 >> 8)
}

fn calc_king_attacks(king: u64) -> u64 {
    let mut attack = king;
    attack |= bb_north(attack);
    attack |= bb_south(attack);
    attack |= bb_east(attack);
    attack |= bb_west(attack);
    attack &= !king;
    attack
}

pub fn knight_attacks(knight: u64) -> u64 {
    assert!(bb_popcnt(knight) == 1);
    let sq = bb_lsb(knight);
    empty_board_attack(Knight, sq)
}

pub fn bishop_attacks(bishops: u64, occupancy: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut bishops = bishops;
    while bishops != 0 {
        let sq = bb_pop(&mut bishops);
        let mask = unsafe { BISHOP_MASKS[sq as usize] };
        let magic = BISHOP_MAGIC[sq as usize];
        let shift = unsafe { BISHOP_SHIFT[sq as usize] };
        let index = ((mask & occupancy) * magic) >> shift;
        let attack =
            unsafe { BISHOP_ATTACK_TABLE[BISHOP_ATTACK_INDEX[sq as usize] + index as usize] };
        attacks |= attack;
    }
    attacks
}

pub fn rook_attacks(rooks: u64, occupancy: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut rooks = rooks;
    while rooks != 0 {
        let sq = bb_pop(&mut rooks);
        let mask = unsafe { ROOK_MASKS[sq as usize] };
        let magic = ROOK_MAGIC[sq as usize];
        let shift = unsafe { ROOK_SHIFT[sq as usize] };
        let index = ((mask & occupancy) * magic) >> shift;
        let attack = unsafe { ROOK_ATTACK_TABLE[ROOK_ATTACK_INDEX[sq as usize] + index as usize] };
        attacks |= attack;
    }
    attacks
}

pub fn queen_attacks(queens: u64, occupancy: u64) -> u64 {
    rook_attacks(queens, occupancy) | bishop_attacks(queens, occupancy)
}

pub fn king_attacks(king: u64) -> u64 {
    let sq = bb_lsb(king) as u8;
    empty_board_attack(King, sq)
}

pub fn all_attacks(position: Position, attacker: Color) -> u64 {
    let color = attacker;
    let occupancy = position.occupancy();
    let mut attacks: u64 = 0;

    attacks |= pawn_attacks(position.pawns(color), color);
    attacks |= calc_knight_attacks(position.knights(color));
    attacks |= bishop_attacks(position.bishops(color), occupancy);
    attacks |= rook_attacks(position.rooks(color), occupancy);
    attacks |= queen_attacks(position.queens(color), occupancy);
    attacks |= king_attacks(position.king(color));

    attacks
}
