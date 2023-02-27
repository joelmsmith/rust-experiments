use crate::attack::*;
use crate::bitboard::*;
use crate::moves::*;
use crate::square::*;
use crate::things::*;

const EP_OFFSETS: [i16; NUM_COLORS] = [8, -8];
const CASTLE_RIGHTS: [u8; NUM_SQUARES] = [
    13, 15, 15, 15, 12, 15, 15, 14, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 7, 15, 15, 15, 3, 15, 15, 11,
];

#[derive(Clone, Copy, PartialEq)]
pub struct Position {
    bb_piece: [u64; NUM_PIECES],    // bitboards indexed by piece
    bb_color: [u64; NUM_COLORS],    // bitboards indexed by color
    piece_sq: [Piece; NUM_SQUARES], // pieces indexed by square
    color_sq: [Color; NUM_SQUARES], // colors indexed by square

    pub side: Color, // side to move
    pub castle: u8,  // castle rights
    pub ep: u8,      // ep square
    half: i32,       // halfmove clock
    full: i32,       // fullmove clock
}

impl Position {
    pub fn new() -> Position {
        Position {
            bb_piece: [0; NUM_PIECES],
            bb_color: [0; NUM_COLORS],
            piece_sq: [NoPiece; NUM_SQUARES],
            color_sq: [NoColor; NUM_SQUARES],
            side: White,
            ep: NO_SQUARE,
            castle: 0,
            half: 0,
            full: 0,
        }
    }

    pub fn enemy(&self) -> Color {
        match self.side {
            White => Black,
            Black => White,
            _ => panic!("bad side state"),
        }
    }

    pub fn us(&self) -> Color {
        self.side
    }

    pub fn occupancy(&self) -> u64 {
        self.bb_color[White as usize] | self.bb_color[Black as usize]
    }

    pub fn our_pawns(&self) -> u64 {
        self.pawns(self.side)
    }

    pub fn our_knights(&self) -> u64 {
        self.knights(self.side)
    }

    pub fn our_bishops(&self) -> u64 {
        self.bishops(self.side)
    }

    pub fn our_rooks(&self) -> u64 {
        self.rooks(self.side)
    }

    pub fn our_queens(&self) -> u64 {
        self.queens(self.side)
    }

    pub fn our_king(&self) -> u64 {
        self.king(self.side)
    }

    pub fn our_pieces(&self) -> u64 {
        self.bb_color[self.side as usize]
    }

    pub fn their_pawns(&self) -> u64 {
        self.pawns(self.enemy())
    }

    pub fn their_knights(&self) -> u64 {
        self.knights(self.enemy())
    }

    pub fn their_bishops(&self) -> u64 {
        self.bishops(self.enemy())
    }

    pub fn their_rooks(&self) -> u64 {
        self.rooks(self.enemy())
    }

    pub fn their_queens(&self) -> u64 {
        self.queens(self.enemy())
    }

    pub fn their_king(&self) -> u64 {
        self.king(self.enemy())
    }

    pub fn their_pieces(&self) -> u64 {
        self.bb_color[self.enemy() as usize]
    }

    pub fn pawns(&self, color: Color) -> u64 {
        self.bb_piece[Pawn as usize] & self.bb_color[color as usize]
    }

    pub fn knights(&self, color: Color) -> u64 {
        self.bb_piece[Knight as usize] & self.bb_color[color as usize]
    }

    pub fn bishops(&self, color: Color) -> u64 {
        self.bb_piece[Bishop as usize] & self.bb_color[color as usize]
    }

    pub fn rooks(&self, color: Color) -> u64 {
        self.bb_piece[Rook as usize] & self.bb_color[color as usize]
    }

    pub fn queens(&self, color: Color) -> u64 {
        self.bb_piece[Queen as usize] & self.bb_color[color as usize]
    }

    pub fn king(&self, color: Color) -> u64 {
        self.bb_piece[King as usize] & self.bb_color[color as usize]
    }

    pub fn piece_on(&self, sq: u8) -> Piece {
        self.piece_sq[sq as usize]
    }

    pub fn color_on(&self, sq: u8) -> Color {
        self.color_sq[sq as usize]
    }

    fn put_piece(&mut self, sq: u8, piece: Piece, color: Color) {
        let s_idx = sq as usize;
        let p_idx = piece as usize;
        let c_idx = color as usize;

        debug_assert!(self.piece_sq[s_idx] == NoPiece);
        debug_assert!(self.color_sq[s_idx] == NoColor);

        self.piece_sq[s_idx] = piece;
        self.color_sq[s_idx] = color;
        self.bb_piece[p_idx] |= bb_from_sq(sq);
        self.bb_color[c_idx] |= bb_from_sq(sq);
    }

    fn clear_sq(&mut self, sq: u8) {
        let s_idx = sq as usize;
        let p_idx = self.piece_on(sq) as usize;
        let c_idx = self.color_on(sq) as usize;
        self.piece_sq[s_idx] = NoPiece;
        self.color_sq[s_idx] = NoColor;
        self.bb_piece[p_idx] &= !bb_from_sq(sq);
        self.bb_color[c_idx] &= !bb_from_sq(sq);
    }

    fn clear(&mut self) {
        self.bb_piece = [0; NUM_PIECES];
        self.bb_color = [0; NUM_COLORS];
        self.piece_sq = [NoPiece; NUM_SQUARES];
        self.color_sq = [NoColor; NUM_SQUARES];
    }

    pub fn from_fen(&mut self, fen: &str) {
        let mut sq = Squares::A8 as u8;
        let mut rank = Rank8 as u8;
        let split: Vec<&str> = fen.split(" ").collect();
        let board = split[0].chars();
        let side = split[1].as_bytes()[0].to_ascii_lowercase() as char; // ?!
        let castling = split[2].chars();
        let ep = split[3].as_bytes();
        let half = split[4].parse::<i32>().unwrap(); // ?!
        let full = split[5].parse::<i32>().unwrap();

        self.clear();

        for c in board {
            if c == ' ' {
                break;
            }
            if c.is_ascii_digit() {
                match c.to_digit(10) {
                    Some(x) => sq += x as u8,
                    None => panic!("bogus input"),
                }
            } else if c == '/' {
                rank = rank - 1;
                sq = make_sq(rank as u8, FileA as u8);
            } else {
                let piece = piece_from_char(c);
                let color = color_from_char(c);
                self.put_piece(sq, piece, color);
                sq = sq + 1;
            }
        }

        self.castle = 0;
        for c in castling {
            match c {
                'K' => self.castle |= WHITE_OO,
                'Q' => self.castle |= WHITE_OOO,
                'k' => self.castle |= BLACK_OO,
                'q' => self.castle |= BLACK_OOO,
                _ => break,
            }
        }

        match side {
            'w' => self.side = White,
            'b' => self.side = Black,
            _ => panic!("weird side"),
        }

        if ep[0] as char != '-' {
            let ep_file = ep[0] - ('a' as u8);
            let ep_rank = ep[1] - ('1' as u8);
            self.ep = make_sq(ep_rank, ep_file);
        }

        self.half = half;
        self.full = full;
    }

    pub fn to_fen(&self) -> String {
        let mut s = String::new();
        for rank in (0..(NUM_RANKS as u8)).rev() {
            let mut empty: u8 = 0;
            for file in 0..(NUM_FILES as u8) {
                let sq = make_sq(rank, file);
                let piece = self.piece_on(sq);
                let color = self.color_on(sq);
                let c = piece_to_str(piece, color).to_string().chars().next();

                if c.unwrap() == ' ' {
                    empty += 1;
                } else {
                    if empty > 0 {
                        s.push_str(&empty.to_string());
                        empty = 0;
                    }
                    s.push(c.unwrap());
                }
            }
            if empty > 0 {
                s.push_str(&empty.to_string());
            }
            if rank != 0 {
                s.push('/');
            }
        }
        match self.side {
            White => s.push_str(" w "),
            Black => s.push_str(" b "),
            _ => panic!("Unknown side"),
        }

        // can I use match here?
        if self.castle == 0 {
            s.push('-');
        } else {
            if self.castle & WHITE_OO != 0 {
                s.push('K');
            }
            if self.castle & WHITE_OOO != 0 {
                s.push('Q');
            }
            if self.castle & BLACK_OO != 0 {
                s.push('k');
            }
            if self.castle & BLACK_OOO != 0 {
                s.push('q');
            }
        }
        s.push(' ');
        s.push_str(&sq_to_str(self.ep));
        s.push(' ');
        s.push_str(&self.half.to_string());
        s.push(' ');
        s.push_str(&self.full.to_string());
        s
    }

    pub fn debug(&self) {
        let mut s = String::new();
        s.push_str("\n     A   B   C   D   E   F   G   H\n");
        s.push_str("   +---+---+---+---+---+---+---+---+\n");

        for rank in (0..NUM_RANKS).rev() {
            let rank = rank as u8; // clumsy
            s.push_str(" ");
            s.push_str(&(rank + 1).to_string());
            s.push_str(" ");
            for file in 0..NUM_FILES {
                let file = file as u8;
                let sq = make_sq(rank, file);
                let piece = self.piece_on(sq);
                let color = self.color_on(sq);
                s.push_str("| ");
                s.push_str(&piece_to_str(piece, color));
                s.push_str(" ");
            }
            s.push_str("|\n   +---+---+---+---+---+---+---+---+\n");
        }
        s.push_str(&self.to_fen());
        println!("{}", s);
    }

    pub fn make_move(&mut self, mv: u16) -> Undo {
        let src = mv_get_src(mv);
        let dst = mv_get_dst(mv);
        let kind = mv_get_kind(mv);
        let moved_piece = self.piece_on(src);
        let moved_color = self.color_on(src);
        let captured_piece = self.piece_on(dst);
        let is_capture = captured_piece != NoPiece;
        let is_pawn_mv = moved_piece == Pawn;
        let enemy = self.enemy();
        let undo = Undo {
            mv: mv,
            captured: captured_piece,
            castle: self.castle,
            half: self.half,
            ep: self.ep,
        };

        debug_assert!(moved_color == self.side);

        if is_capture || is_pawn_mv {
            self.half = 0;
        } else {
            self.half += 1;
        }
        self.full += self.side as i32;

        if is_capture {
            self.clear_sq(dst);
        }

        if self.ep != NO_SQUARE {
            self.ep = NO_SQUARE;
        }

        self.clear_sq(src);
        match kind {
            MOVE_ENPASSANT => {
                let capture_sq = (dst as i16 - EP_OFFSETS[self.side as usize]) as u8; // clumsy
                self.clear_sq(capture_sq);
                self.put_piece(dst, Pawn, self.side);
            }
            MOVE_CASTLE => {
                // The move's src and dst squares are for the king; if king
                // moves to the right then it's OO, else OOO.  From this, the
                // rook location is determined.
                let src_file = file_of(src);
                let dst_file = file_of(dst);
                let rook_src_file = if src_file < dst_file { FileH } else { FileA } as u8;
                let rook_dst_file = if rook_src_file == FileH as u8 {
                    FileF
                } else {
                    FileD
                } as u8;
                let rank = if self.side == White { Rank1 } else { Rank8 } as u8;
                let rook_src_sq = make_sq(rank, rook_src_file);
                let rook_dst_sq = make_sq(rank, rook_dst_file);

                debug_assert!(moved_piece == King);
                debug_assert!(file_of(src) == FileE as u8);
                debug_assert!(self.piece_on(rook_src_sq) == Rook);
                debug_assert!(self.color_on(rook_src_sq) == self.side);
                debug_assert!(self.piece_on(rook_dst_sq) == NoPiece);

                self.clear_sq(rook_src_sq);
                self.put_piece(dst, King, self.side);
                self.put_piece(rook_dst_sq, Rook, self.side);
            }
            MOVE_PROMO => {
                let promoted_piece = mv_get_promo_piece(mv);
                self.put_piece(dst, promoted_piece, self.side);
            }
            MOVE_NORMAL => {
                self.put_piece(dst, moved_piece, self.side);
                if moved_piece == Pawn && (dst.abs_diff(src) == 16) {
                    // Set EP square, but only if our pawn move draws up alongside an enemy pawn.
                    let enemy_pawns = self.bb_piece[Pawn as usize] & self.bb_color[enemy as usize];
                    let pawn = bb_from_sq(dst);
                    if bb_east(pawn) & enemy_pawns != 0 || bb_west(pawn) & enemy_pawns != 0 {
                        self.ep = (dst as i16 - EP_OFFSETS[self.side as usize]) as u8;
                    }
                }
            }
            _ => panic!("Unknown move kind"),
        }

        self.castle &= CASTLE_RIGHTS[src as usize];
        self.castle &= CASTLE_RIGHTS[dst as usize];
        self.side = enemy;
        undo
    }

    pub fn unmake_move(&mut self, undo: Undo) {
        // src and dst are from the perspective of the player before the move
        // was made
        let src = mv_get_src(undo.mv);
        let dst = mv_get_dst(undo.mv);
        let kind = mv_get_kind(undo.mv);
        let piece = self.piece_on(dst);
        let captured = undo.captured;
        let enemy = self.side;

        // self.side ^= 1;
        self.side = if self.side == White { Black } else { White }; // CLUMSY

        // self.full -= self.side;
        self.full -= if self.side == Black { 1 } else { 0 }; // CLUMSY

        self.clear_sq(dst);
        self.put_piece(src, piece, self.side);
        if captured != NoPiece {
            // self.put_piece(dst, captured, self.side ^ 1);
            self.put_piece(dst, captured, enemy);
        }

        match kind {
            MOVE_ENPASSANT => {
                // Restore the captured pawn.
                let sq = (undo.ep as i16 - EP_OFFSETS[self.side as usize]) as u8; // CLUMSY
                self.put_piece(sq, Pawn, self.enemy());
            }
            MOVE_CASTLE => {
                // Put the rook back.  The king is back already.
                let rank = if self.side == White { Rank1 } else { Rank8 } as u8;
                let rook_src_file = if file_of(dst) > file_of(src) {
                    FileH
                } else {
                    FileA
                } as u8;
                let rook_dst_file = if rook_src_file == FileH as u8 {
                    FileF
                } else {
                    FileD
                } as u8;
                let rook_dst_sq = make_sq(rank, rook_dst_file);
                let rook_src_sq = make_sq(rank, rook_src_file);
                self.clear_sq(rook_dst_sq);
                self.put_piece(rook_src_sq, Rook, self.side);
            }
            MOVE_PROMO => {
                // Transform the promoted piece back to a pawn.
                self.clear_sq(src);
                self.put_piece(src, Pawn, self.side);
            }
            MOVE_NORMAL => {}
            _ => panic!("Unknown move kind"),
        }
        self.castle = undo.castle;
        self.half = undo.half;
        self.ep = undo.ep;
    }

    pub fn calc_checkers(&self) -> u64 {
        let king = self.our_king();
        let king_sq = bb_lsb(king);
        let their_rooks = self.their_rooks();
        let their_bishops = self.their_bishops();
        let their_queens = self.their_queens();
        let their_diagonals = their_queens | their_bishops;
        let their_straights = their_queens | their_rooks;
        let mut attackers: u64 = 0;

        attackers |= pawn_attacks(king, self.us()) & self.their_pawns();
        attackers |= knight_attacks_from(king_sq) & self.their_knights();
        attackers |= bishop_attacks(king, self.occupancy()) & their_diagonals;
        attackers |= rook_attacks(king, self.occupancy()) & their_straights;
        attackers
    }

    pub fn calc_pinned(&self) -> u64 {
        let mut pinned: u64 = 0;
        let their_queens = self.their_queens();
        let their_rooks = self.their_rooks();
        let their_bishops = self.their_bishops();
        let their_diagonals = their_queens | their_bishops;
        let their_straights = their_queens | their_rooks;

        if their_diagonals | their_straights != 0 {
            // Enemy has slider pieces on the board.   Need to check to see if
            // any of them are pinning any of our pieces to our king.
            let our_king = self.our_king();
            let king_sq = bb_lsb(our_king) as u8;
            debug_assert!(bb_popcnt(our_king) == 1);

            // Conceptually, put a queen where our king is on an empty board.
            // What squares are attacked?  Discard enemy slider pieces not on
            // those attacked squares.
            let mut maybe_pinners: u64 = 0;
            maybe_pinners |= their_diagonals & bishop_attacks(our_king, 0);
            maybe_pinners |= their_straights & rook_attacks(our_king, 0);

            // Examine each potential pinner.
            while maybe_pinners != 0 {
                let sq = bb_pop(&mut maybe_pinners);
                let between = bb_between(king_sq, sq);

                // Any enemy piece(s) between our king and enemy slider?
                if between & self.their_pieces() != 0 {
                    // Yes -> that enemy piece is not a pinner.
                    continue;
                }

                // Any friendly piece(s) between our king and enemy slider?
                let maybe_pinned = self.our_pieces() & between;
                if bb_popcnt(maybe_pinned) == 1 {
                    // Only one friendly piece between: that piece is pinned.
                    pinned |= maybe_pinned;
                } else {
                    // > 1 piece between: no pin.
                }
            }
        }
        pinned
    }
}
