use crate::board::board::{Board, ALL_PIECES, BISHOP, KING, KNIGHT, QUEEN, ROOK};
use crate::movegen::move_info::{FA, FH};
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::get_xpiece;
use crate::movegen::movegen_iters::MovegenIterator;
use crate::movegen::moves::{Move, MoveType};
use crate::print_bb;

pub struct KingQuietIterator {
    piece: u32,
    from: u32,
    quiet: u64,
}

impl KingQuietIterator {
    pub fn new(b: &Board) -> KingQuietIterator {
        let from = b.pieces[KING + b.ctm].trailing_zeros();
        let quiet = MT::king_moves(from as usize) & !b.util[ALL_PIECES];

        KingQuietIterator {
            piece: (KING + b.ctm) as u32,
            from,
            quiet,
        }
    }

    fn next_quiet(&mut self) -> Option<Move> {
        if self.quiet == 0 {
            return None;
        }

        let to = self.quiet.trailing_zeros();
        self.quiet &= self.quiet - 1;

        Some(Move::new(self.from, to, self.piece, 0, MoveType::Quiet))
    }
}

impl Iterator for KingQuietIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_quiet()
    }
}

pub struct KingAttackIterator<'a> {
    board: &'a Board,
    piece: u32,
    from: u32,
    attack: u64,
}

impl<'a> KingAttackIterator<'a> {
    pub fn new(b: &Board) -> KingAttackIterator {
        let from = b.pieces[KING + b.ctm].trailing_zeros();
        let attack = MT::king_moves(from as usize) & b.util[b.ctm ^ 1];

        KingAttackIterator {
            board: b,
            piece: (KING + b.ctm) as u32,
            from,
            attack,
        }
    }

    fn next_attack(&mut self) -> Option<Move> {
        if self.attack == 0 {
            return None;
        }

        let to = self.attack.trailing_zeros();
        self.attack &= self.attack - 1;

        let xpiece = get_xpiece(self.board, to).unwrap();

        Some(Move::new(self.from, to, self.piece, xpiece, MoveType::Cap))
    }
}

impl<'a> Iterator for KingAttackIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_attack()
    }
}

pub struct KingCastleIterator {
    ctm: usize,
    from: u32,
    kingside: u8,
    queenside: u8,
    occ: u64,
}

impl KingCastleIterator {
    pub fn new(b: &Board) -> KingCastleIterator {
        let rights = b.castle_state >> (2 * (b.ctm ^ 1));
        KingCastleIterator {
            ctm: b.ctm,
            from: b.pieces[KING + b.ctm].trailing_zeros(),
            kingside: rights & 0b10,
            queenside: rights & 1,
            occ: b.util[ALL_PIECES],
        }
    }

    fn next_kingside(&self) -> Option<Move> {
        if (self.occ & (0x60 << (self.ctm * 56))) == 0 {
            let move_type = MoveType::kingside(self.ctm);
            Some(Move::new(
                self.from,
                self.from + 2,
                (KING + self.ctm) as u32,
                0,
                move_type,
            ))
        } else {
            None
        }
    }

    fn next_queenside(&self) -> Option<Move> {
        if self.occ & (0xE << (self.ctm * 56)) == 0 {
            let move_type = MoveType::queenside(self.ctm);
            Some(Move::new(
                self.from,
                self.from - 2,
                (KING + self.ctm) as u32,
                0,
                move_type,
            ))
        } else {
            None
        }
    }

    fn next_castle(&mut self) -> Option<Move> {
        let mut castle = None;

        if self.kingside > 0 {
            castle = self.next_kingside();
            self.kingside = 0;
        } else if self.queenside > 0 {
            castle = self.next_queenside();
            self.queenside = 0;
        }

        castle
    }
}

impl Iterator for KingCastleIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_castle()
    }
}

pub struct KingMovesIterator<'a> {
    board: &'a Board,
    piece: u32,
    from: u32,
    quiet: u64,
    attack: u64,
    // flag for lazily loading legal king moves
    has_loaded: bool,
}

impl<'a> KingMovesIterator<'a> {
    pub fn new(b: &Board) -> KingMovesIterator {
        KingMovesIterator {
            board: b,
            piece: (KING + b.ctm) as u32,
            from: b.pieces[KING + b.ctm].trailing_zeros(),
            quiet: 0,
            attack: 0,
            has_loaded: false,
        }
    }

    fn gen_moves(&mut self) {
        let mut possible =
            MT::king_moves(self.board.pieces[KING + self.board.ctm].trailing_zeros() as usize);

        possible &= !self.board.util[self.board.ctm];
        possible &= !opp_pawn_attacks(self.board);
        possible &= !opp_knight_moves(self.board);
        possible &= !opp_rook_queen_moves(self.board);
        possible &= !opp_bishop_queen_moves(self.board);
        possible &= !opp_king_moves(self.board);

        self.quiet = possible & !self.board.util[ALL_PIECES];
        self.attack = possible & self.board.util[self.board.ctm ^ 1];
        self.has_loaded = true;
    }

    fn next_attack(&mut self) -> Option<Move> {
        let to = self.attack.trailing_zeros();
        self.attack &= self.attack - 1;

        let xpiece = get_xpiece(self.board, to).unwrap();

        Some(Move::new(self.from, to, self.piece, xpiece, MoveType::Cap))
    }

    fn next_quiet(&mut self) -> Option<Move> {
        let to = self.quiet.trailing_zeros();
        self.quiet &= self.quiet - 1;

        Some(Move::new(self.from, to, self.piece, 0, MoveType::Quiet))
    }

    fn next_check_move(&mut self) -> Option<Move> {
        if !self.has_loaded {
            self.gen_moves();
        }

        let mut check = None;

        if self.attack > 0 {
            check = self.next_attack()
        } else if self.quiet > 0 {
            check = self.next_quiet()
        }

        check
    }
}

fn opp_pawn_attacks(b: &Board) -> u64 {
    if b.ctm ^ 1 == 0 {
        ((b.pieces[0] & !FA) << 7) | ((b.pieces[0] & !FH) << 9)
    } else {
        ((b.pieces[1] & !FH) >> 7) | ((b.pieces[1] & !FA) >> 9)
    }
}

fn opp_knight_moves(b: &Board) -> u64 {
    let mut knight_moves = 0;
    let mut knights = b.pieces[KING + b.ctm ^ 1];
    while knights > 0 {
        knight_moves |= MT::knight_moves(knights.trailing_zeros() as usize);
        knights &= knights - 1;
    }

    knight_moves
}

fn opp_rook_queen_moves(b: &Board) -> u64 {
    let mut rq_moves = 0;
    let mut rook_queens = b.pieces[ROOK + b.ctm ^ 1] | b.pieces[QUEEN + b.ctm ^ 1];
    let occ = b.util[ALL_PIECES] & !b.pieces[KING + b.ctm];
    while rook_queens > 0 {
        rq_moves |= MT::rook_moves(occ, rook_queens.trailing_zeros() as usize);
        rook_queens &= rook_queens - 1;
    }

    rq_moves
}

fn opp_bishop_queen_moves(b: &Board) -> u64 {
    let mut bq_moves = 0;
    let mut bishop_queens = b.pieces[BISHOP + b.ctm ^ 1] | b.pieces[QUEEN + b.ctm ^ 1];
    let occ = b.util[ALL_PIECES] & !b.pieces[KING + b.ctm];
    while bishop_queens > 0 {
        bq_moves |= MT::bishop_moves(occ, bishop_queens.trailing_zeros() as usize);
        bishop_queens &= bishop_queens - 1;
    }

    bq_moves
}

fn opp_king_moves(b: &Board) -> u64 {
    MT::king_moves(b.pieces[KING + b.ctm ^ 1].trailing_zeros() as usize)
}

impl<'a> Iterator for KingMovesIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_check_move()
    }
}

#[test]
fn king_moves_iter() {
    crate::init();

    let b = Board::new_fen("8/8/1p1p4/KP4Pr/1R3p1k/8/4P3/8 w - -").unwrap();
    let wmoves = KingMovesIterator::new(&b);
    assert_eq!(wmoves.count(), 3);

    let b = Board::new_fen("8/8/1p1p4/KP4Pr/1R3p1k/8/4P3/8 b - -").unwrap();
    let bmoves = KingMovesIterator::new(&b);
    assert_eq!(bmoves.count(), 4)
}

#[test]
fn king_castle_iter() {
    crate::init();

    let b =
        Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap();
    let wcastle = KingCastleIterator::new(&b);
    assert_eq!(wcastle.count(), 2);

    let b =
        Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq -").unwrap();
    let bcastle = KingCastleIterator::new(&b);
    assert_eq!(bcastle.count(), 2);
}
