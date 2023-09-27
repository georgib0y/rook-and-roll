use crate::board::board::{Board, ALL_PIECES, ROOK};
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::{get_xpiece, ALL_SQUARES, NO_SQUARES};
use crate::movegen::movegen_iters::MovegenIterator;
use crate::movegen::moves::{Move, MoveType};

pub struct RookQuietIterator {
    piece: u32,
    rooks: u64,
    occ: u64,
    quiet: u64,
    target: u64,
}

impl RookQuietIterator {
    pub fn new(b: &Board, pinned: u64, target: u64) -> RookQuietIterator {
        let rooks = b.pieces[ROOK + b.ctm] & !pinned;
        let occ = b.util[ALL_PIECES];
        let from = rooks.trailing_zeros() as usize % 64;
        let quiet = MT::rook_moves(occ, from) & !occ & target;

        RookQuietIterator {
            piece: (ROOK + b.ctm) as u32,
            rooks,
            occ,
            quiet,
            target,
        }
    }

    fn next_quiet(&mut self) -> Option<Move> {
        if self.rooks == 0 || (self.quiet == 0 && self.rooks.count_ones() == 1) {
            return None;
        }

        let mut from = self.rooks.trailing_zeros();

        if self.quiet == 0 {
            self.rooks &= self.rooks - 1;
            from = self.rooks.trailing_zeros();
            self.quiet = MT::rook_moves(self.occ, from as usize) & !self.occ & self.target;
            if self.quiet == 0 {
                return None;
            }
        }

        let to = self.quiet.trailing_zeros();
        self.quiet &= self.quiet - 1;

        Some(Move::new(from, to, self.piece, 0, MoveType::Quiet))
    }
}

impl Iterator for RookQuietIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_quiet()
    }
}

pub struct RookAttackIterator<'a> {
    board: &'a Board,
    piece: u32,
    rooks: u64,
    occ: u64,
    attack: u64,
    opp: u64,
}

impl<'a> RookAttackIterator<'a> {
    pub fn new(b: &Board, pinned: u64, target: u64) -> RookAttackIterator {
        let rooks = b.pieces[ROOK + b.ctm] & !pinned;
        let opp = b.util[b.ctm ^ 1] & target;
        let occ = b.util[ALL_PIECES];
        let from = rooks.trailing_zeros() % 64;
        let attack = MT::rook_moves(occ, from as usize) & opp;

        RookAttackIterator {
            board: b,
            piece: (ROOK + b.ctm) as u32,
            rooks,
            occ,
            attack,
            opp,
        }
    }

    fn next_attack(&mut self) -> Option<Move> {
        if self.rooks == 0 || (self.attack == 0 && self.rooks.count_ones() == 1) {
            return None;
        }

        let mut from = self.rooks.trailing_zeros();

        if self.attack == 0 {
            self.rooks &= self.rooks - 1;
            from = self.rooks.trailing_zeros();
            self.attack = MT::rook_moves(self.occ, from as usize) & self.opp;
            if self.attack == 0 {
                return None;
            }
        }

        let to = self.attack.trailing_zeros();
        self.attack &= self.attack - 1;

        let xpiece = get_xpiece(self.board, to).unwrap();

        Some(Move::new(from, to, self.piece, xpiece, MoveType::Cap))
    }
}

impl<'a> Iterator for RookAttackIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_attack()
    }
}

#[test]
fn rook_moves_iter() {
    crate::init();
    let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq -").unwrap();
    let wquiet = RookQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    let moves: Vec<_> = wquiet.collect();
    moves.iter().for_each(|m| println!("{m}"));
    assert_eq!(moves.len(), 4);

    let wattack = RookAttackIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(wattack.count(), 0);

    let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 b kq -").unwrap();

    let bquiet = RookQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(bquiet.count(), 5);

    let battack = RookAttackIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(battack.count(), 1);
}
