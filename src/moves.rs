use crate::attack::*;
use crate::bitboard::*;
use crate::position::*;
use crate::square::*;
use crate::things::*;

pub const MOVE_NORMAL: u8 = 0;
pub const MOVE_PROMO: u8 = 1;
pub const MOVE_ENPASSANT: u8 = 2;
pub const MOVE_CASTLE: u8 = 3;
pub const PROMOTION_RANKS: [u8; 2] = [Rank8 as u8, Rank1 as u8];

fn mv_create(src: u8, dst: u8, kind: u8, piece: u8) -> u16 {
    let src = src as u16;
    let dst = dst as u16;
    let kind = kind as u16;
    let piece = piece as u16;
    (((piece - 1) << 14) | (kind << 12) | (dst << 6) | src) as u16
}

pub fn mv_create_normal(src: u8, dst: u8) -> u16 {
    mv_create(src, dst, MOVE_NORMAL, Knight as u8)
}

pub fn mv_create_promo(src: u8, dst: u8, piece: u8) -> u16 {
    mv_create(src, dst, MOVE_PROMO, piece)
}

pub fn mv_create_castle(src: u8, dst: u8) -> u16 {
    mv_create(src, dst, MOVE_CASTLE, Knight as u8)
}

pub fn mv_create_ep(src: u8, dst: u8) -> u16 {
    mv_create(src, dst, MOVE_ENPASSANT, Knight as u8)
}

pub fn mv_get_src(mv: u16) -> u8 {
    (mv & 0x3f).try_into().unwrap()
}

pub fn mv_get_dst(mv: u16) -> u8 {
    ((mv >> 6) & 0x3f).try_into().unwrap()
}

pub fn mv_get_kind(mv: u16) -> u8 {
    ((mv >> 12) & 0x3).try_into().unwrap()
}

pub fn mv_get_promo_piece(mv: u16) -> Piece {
    unsafe { std::mem::transmute(1 + ((mv >> 14) & 0x3) as u8) }
}

pub struct Undo {
    pub mv: u16,
    pub captured: Piece,
    pub castle: u8,
    pub half: i32,
    pub ep: u8,
}

pub struct MoveGen {
    position: Position,
    attacked: u64,
    occupancy: u64,
    our_pieces: u64,
    their_pieces: u64,
    our_pinned_pieces: u64,
    their_checkers: u64,
    pub moves: Vec<u16>,
}

impl MoveGen {
    pub fn new(position: Position) -> MoveGen {
        MoveGen {
            position: position,
            attacked: all_attacks(position, position.enemy()),
            occupancy: position.occupancy(),
            our_pieces: position.our_pieces(),
            their_pieces: position.their_pieces(),
            our_pinned_pieces: position.calc_pinned(),
            their_checkers: position.calc_checkers(),
            moves: Vec::with_capacity(256)
        }
    }

    pub fn gen_legal_moves(&mut self) {
        if self.position.our_king() & self.attacked != 0 {
            self.gen_get_out_of_check_moves();
        } else {
            let all: u64 = 0xffffffffffffffff;
            self.gen_pawn_moves(all);
            self.gen_knight_moves(all);
            self.gen_bishop_moves(all);
            self.gen_rook_moves(all);
            self.gen_queen_moves(all);
            self.gen_king_moves(all);
            self.gen_castling_moves();
        }
    }

    fn gen_pawn_advances(&mut self, targets: u64) {
        let mut pawns = self.position.our_pawns();
        while pawns != 0 {
            let src_sq = bb_pop(&mut pawns);
            let pawn = bb_from_sq(src_sq);
            let mut advances = targets & pawn_advances(pawn, self.occupancy, self.position.us());
            while advances != 0 {
                let dst_sq = bb_pop(&mut advances);
                if self.passed_pin_check(src_sq, dst_sq) {
                    if rank_of(dst_sq) == PROMOTION_RANKS[self.position.us() as usize] {
                        let mv_n = mv_create_promo(src_sq, dst_sq, Knight as u8);
                        let mv_b = mv_create_promo(src_sq, dst_sq, Bishop as u8);
                        let mv_r = mv_create_promo(src_sq, dst_sq, Rook as u8);
                        let mv_q = mv_create_promo(src_sq, dst_sq, Queen as u8);
                        self.moves.push(mv_n);
                        self.moves.push(mv_b);
                        self.moves.push(mv_r);
                        self.moves.push(mv_q);
                    } else {
                        let mv = mv_create_normal(src_sq, dst_sq);
                        self.moves.push(mv);
                    }
                }
            }
        }
    }

    fn gen_pawn_captures(&mut self, targets: u64) {
        let mut pawns = self.position.our_pawns();
        while pawns != 0 {
            let src_sq = bb_pop(&mut pawns);
            let pawn = bb_from_sq(src_sq);
            let mut attacks = pawn_attacks(pawn, self.position.us());

            attacks &= targets;
            attacks &= self.their_pieces;

            while attacks != 0 {
                let dst_sq = bb_pop(&mut attacks);
                if self.passed_pin_check(src_sq, dst_sq) {
                    if rank_of(dst_sq) == PROMOTION_RANKS[self.position.us() as usize] {
                        let mv_n = mv_create_promo(src_sq, dst_sq, Knight as u8);
                        let mv_b = mv_create_promo(src_sq, dst_sq, Bishop as u8);
                        let mv_r = mv_create_promo(src_sq, dst_sq, Rook as u8);
                        let mv_q = mv_create_promo(src_sq, dst_sq, Queen as u8);
                        self.moves.push(mv_n);
                        self.moves.push(mv_b);
                        self.moves.push(mv_r);
                        self.moves.push(mv_q);
                    } else {
                        let mv = mv_create_normal(src_sq, dst_sq);
                        self.moves.push(mv);
                    }
                }
            }
        }

        if self.position.ep == NO_SQUARE {
            return;
        }

        // Deal with en passant.
        let ep_sq = self.position.ep;
        let ep_file_bb = FILE_BITBOARDS[file_of(ep_sq) as usize];
        let ep_bb = bb_from_sq(ep_sq);
        let mut pawns = self.position.our_pawns() & (bb_east(ep_file_bb) | bb_west(ep_file_bb));
        while pawns != 0 {
            let src_sq = bb_pop(&mut pawns);
            let dst_sq = ep_sq;
            let pawn = bb_from_sq(src_sq);

            if pawn_attacks(pawn, self.position.us()) & ep_bb != 0 {
                // En Passant may be possible - need to check legality.

                let captured = match self.position.us() {
                    White => bb_south(ep_bb),
                    Black => bb_north(ep_bb),
                    _ => panic!("Bad color")
                };
                let ray = bb_ray(src_sq, dst_sq);
                // A pawn performing an en passant capture can be diagonally
                // pinned against his king, yet still perform the capture since
                // he is moving along the pin ray.  Although the pawn is still
                // "pinned", it's not considered pinned in this context.
                let pawn_is_pinned = (pawn & self.our_pinned_pieces != 0) && (ray & self.position.our_king() == 0);

                // In the board below.. if we put a rook where our king is then
                // remove the two pawns, would that rook attack any enemy rooks
                // or queens?  If so, then the ep move in question is illegal.
                // The diagonal version of this problem does not appear to be
                // possible during legal play, so a check for that is omitted.
                /*    A   B   C   D   E   F   G   H
                    +---+---+---+---+---+---+---+---+
                  8 | k |   |   |   |   |   |   |   |
                    +---+---+---+---+---+---+---+---+
                  7 |   |   |   |   |   |   |   |   |
                    +---+---+---+---+---+---+---+---+
                  6 |   |   |   |   |eps|   |   |   |
                    +---+---+---+---+---+---+---+---+
                  5 | q |   |   | P | p |   |   | K |       d5e6 (captures on e5) is illegal
                    +---+---+---+---+---+---+---+---+
                  4 |   |   |   |   |   |   |   |   |
                    +---+---+---+---+---+---+---+---+
                  3 |   |   |   |   |   |   |   |   |
                    +---+---+---+---+---+---+---+---+
                  2 |   |   |   |   |   |   |   |   |
                    +---+---+---+---+---+---+---+---+
                  1 |   |   |   |   |   |   |   |   |
                    +---+---+---+---+---+---+---+---+ */

                let their_straights = self.position.their_rooks() | self.position.their_queens();
                // occ is board occupancy after ep move is made:
                let occ = (ep_bb | self.occupancy) & !bb_from_sq(src_sq) & !captured;
                // illegal if a rook attack from king's sq reaches an enemy rook or queen
                let illegal = rook_attacks(self.position.our_king(), occ) & their_straights != 0;

                if !illegal && !pawn_is_pinned {
                    let mv = mv_create_ep(src_sq, dst_sq);
                    self.moves.push(mv);
                }
            }
        }
    }

    fn gen_pawn_moves(&mut self, targets: u64) {
        self.gen_pawn_advances(targets);
        self.gen_pawn_captures(targets);
    }

    fn gen_knight_moves(&mut self, targets: u64) {
        let mut knights = self.position.our_knights();
        let mut targets = targets;

        targets &= !self.our_pieces;

        // Knights can't move along a pin ray, so any pinned knight has no moves
        knights &= !self.our_pinned_pieces;

        while knights != 0 {
            let src_sq = bb_pop(&mut knights);

            let mut attacks = knight_attacks_from(src_sq);
            attacks &= targets;

            while attacks != 0 {
                let dst_sq = bb_pop(&mut attacks);
                let mv = mv_create_normal(src_sq, dst_sq);
                self.moves.push(mv);
            }
        }
    }

    fn gen_bishop_moves(&mut self, targets: u64) {
        let mut bishops = self.position.our_bishops();
        let mut targets = targets;

        targets &= !self.our_pieces;

        while bishops != 0 {
            let src_sq = bb_pop(&mut bishops);
            let bishop = bb_from_sq(src_sq);
            let mut attacks = bishop_attacks(bishop, self.occupancy);
            attacks &= targets;
            while attacks != 0 {
                let dst_sq = bb_pop(&mut attacks);
                if self.passed_pin_check(src_sq, dst_sq) {
                    let mv = mv_create_normal(src_sq, dst_sq);
                    self.moves.push(mv);
                }
            }
        }
    }

    fn gen_rook_moves(&mut self, targets: u64) {
        let mut rooks = self.position.our_rooks();
        let mut targets = targets;

        targets &= !self.our_pieces;

        while rooks != 0 {
            let src_sq = bb_pop(&mut rooks);
            let rook = bb_from_sq(src_sq);
            let mut attacks = rook_attacks(rook, self.occupancy);
            attacks &= targets;
            while attacks != 0 {
                let dst_sq = bb_pop(&mut attacks);
                if self.passed_pin_check(src_sq, dst_sq) {
                    let mv = mv_create_normal(src_sq, dst_sq);
                    self.moves.push(mv);
                }
            }
        }
    }

    fn gen_queen_moves(&mut self, targets: u64) {
        let mut queens = self.position.our_queens();
        let mut targets = targets;

        targets &= !self.our_pieces;

        while queens != 0 {
            let src_sq = bb_pop(&mut queens);
            let queen = bb_from_sq(src_sq);
            let mut attacks = queen_attacks(queen, self.occupancy);
            attacks &= targets;
            while attacks != 0 {
                let dst_sq = bb_pop(&mut attacks);
                if self.passed_pin_check(src_sq, dst_sq) {
                    let mv = mv_create_normal(src_sq, dst_sq);
                    self.moves.push(mv);
                }
            }
        }
    }

    fn gen_king_moves(&mut self, targets: u64) {
        let king = self.position.our_king();
        let mut attacks = king_attacks(king);

        attacks &= targets;
        attacks &= !self.our_pieces;
        attacks &= !self.attacked;

        while attacks != 0 {
            let src_sq = bb_lsb(king);
            let dst_sq = bb_pop(&mut attacks);
            let mv = mv_create_normal(src_sq, dst_sq);
            self.moves.push(mv);
        }
    }

    fn gen_get_out_of_check_moves(&mut self) {
        let checkers = self.their_checkers;
        let king_sq = bb_lsb(self.position.our_king());

        // Handle the following situation:
        // Although c1 is not attacked by black, Kc1 is illegal.
        /*  A   B   C   D   E   F   G   H
          +---+---+---+---+---+---+---+---+
        8 | k |   | q | b |   |   |   |   |
          +---+---+---+---+---+---+---+---+
        7 |   |   |   |   |   |   |   |   |
          +---+---+---+---+---+---+---+---+
        6 |   |   |   |   |   |   |   |   |
          +---+---+---+---+---+---+---+---+
        5 |   |   |   |   |   |   |   |   |
          +---+---+---+---+---+---+---+---+
        4 |   |   |   |   |   |   |   |   |
          +---+---+---+---+---+---+---+---+
        3 |   |   |   |   |   |   |   |   |
          +---+---+---+---+---+---+---+---+
        2 |   |   | K | N |   |   |   |   |
          +---+---+---+---+---+---+---+---+
        1 |   |   |   |   |   |   |   |   |
          +---+---+---+---+---+---+---+---+
            A   B   C   D   E   F   G   H   */

        let mut slider_attack_rays: u64 = 0;
        let mut slider_checkers = checkers
            & (self.position.their_bishops()
                | self.position.their_queens()
                | self.position.their_rooks());
        // It's possible to have double check with two slider checkers, e.g. a
        // Rook unblocks an check from a Bishop while delivering a check itself.
        while slider_checkers != 0 {
            let sq = bb_pop(&mut slider_checkers);
            slider_attack_rays |= bb_ray(king_sq, sq) ^ bb_from_sq(sq);
        }

        let king_targets = !(self.attacked | slider_attack_rays);
        self.gen_king_moves(king_targets);

        if bb_popcnt(checkers) == 1 {
            // Only one checker - can capture it or intercept its attack ray.
            let attack_ray = bb_between(bb_lsb(checkers), king_sq);
            let targets = checkers | attack_ray;
            self.gen_pawn_moves(targets);
            self.gen_knight_moves(targets);
            self.gen_bishop_moves(targets);
            self.gen_rook_moves(targets);
            self.gen_queen_moves(targets);
        } else {
            // Double check: must move king
        }
    }

    // TODO: replace rank/file lookups with bb squares OR consolidate this code
    // to eliminate white vs black.
    fn gen_castling_moves(&mut self) {
        let rights = self.position.castle;
        let us = self.position.us();

        if (us == White) && (rights & WHITE_OO != 0) {
            let src_sq = bb_lsb(self.position.our_king());
            let dst_sq = Squares::G1 as u8;
            let need_empty = RANK_BITBOARDS[Rank1 as usize]
                & (FILE_BITBOARDS[FileF as usize] | FILE_BITBOARDS[FileG as usize]);
            let need_unattacked = self.position.our_king() | need_empty;

            if (self.occupancy & need_empty == 0) && (need_unattacked & self.attacked == 0) {
                let mv = mv_create_castle(src_sq, dst_sq);
                self.moves.push(mv);
            }
        }
        if (us == White) && (rights & WHITE_OOO != 0) {
            let src_sq = bb_lsb(self.position.our_king());
            let dst_sq = Squares::C1 as u8;
            let need_empty = RANK_BITBOARDS[Rank1 as usize]
                & (FILE_BITBOARDS[FileB as usize]
                    | FILE_BITBOARDS[FileC as usize]
                    | FILE_BITBOARDS[FileD as usize]);
            let need_unattacked = self.position.our_king()
                | (RANK_BITBOARDS[Rank1 as usize]
                    & (FILE_BITBOARDS[FileC as usize] | FILE_BITBOARDS[FileD as usize]));

            if (self.occupancy & need_empty == 0) && (need_unattacked & self.attacked == 0) {
                let mv = mv_create_castle(src_sq, dst_sq);
                self.moves.push(mv);
            }
        }
        if (us == Black) && (rights & BLACK_OO != 0) {
            let src_sq = bb_lsb(self.position.our_king());
            let dst_sq = Squares::G8 as u8;
            let need_empty = RANK_BITBOARDS[Rank8 as usize]
                & (FILE_BITBOARDS[FileF as usize] | FILE_BITBOARDS[FileG as usize]);
            let need_unattacked = self.position.our_king() | need_empty;

            if (self.occupancy & need_empty == 0) && (need_unattacked & self.attacked == 0) {
                let mv = mv_create_castle(src_sq, dst_sq);
                self.moves.push(mv);
            }
        }
        if (us == Black) && (rights & BLACK_OOO != 0) {
            let src_sq = bb_lsb(self.position.our_king());
            let dst_sq = Squares::C8 as u8;
            let need_empty = RANK_BITBOARDS[Rank8 as usize]
                & (FILE_BITBOARDS[FileB as usize]
                    | FILE_BITBOARDS[FileC as usize]
                    | FILE_BITBOARDS[FileD as usize]);
            let need_unattacked = self.position.our_king()
                | (RANK_BITBOARDS[Rank8 as usize]
                    & (FILE_BITBOARDS[FileC as usize] | FILE_BITBOARDS[FileD as usize]));

            if (self.occupancy & need_empty == 0) && (need_unattacked & self.attacked == 0) {
                let mv = mv_create_castle(src_sq, dst_sq);
                self.moves.push(mv);
            }
        }
    }

    fn passed_pin_check(&self, src_sq: u8, dst_sq: u8) -> bool {
        let pinned = bb_from_sq(src_sq) & self.our_pinned_pieces != 0;
        if pinned {
            // Pinned piece is only allowed to move along the pin ray.
            return bb_ray(src_sq, dst_sq) & self.position.our_king() != 0;
        }
        true // not pinned
    }
}
