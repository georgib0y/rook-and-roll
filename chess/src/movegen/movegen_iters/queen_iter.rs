use crate::board::board::{Board, ALL_PIECES, BISHOP, QUEEN, ROOK};
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::{get_xpiece, ALL_SQUARES, NO_SQUARES};
use crate::movegen::movegen_iters::MovegenIterator;
use crate::movegen::moves::{Move, MoveType};
use crate::print_bb;

pub struct QueenQuietIterator {
    piece: u32,
    queens: u64,
    occ: u64,
    quiet: u64,
    target: u64,
}

impl QueenQuietIterator {
    pub fn new(b: &Board, pinned: u64, target: u64) -> QueenQuietIterator {
        let queens = b.pieces[QUEEN + b.ctm] & !pinned;
        let occ = b.util[ALL_PIECES];
        let from = queens.trailing_zeros() as usize % 64;
        let quiet = (MT::bishop_moves(occ, from) | MT::rook_moves(occ, from)) & !occ & target;

        QueenQuietIterator {
            piece: (QUEEN + b.ctm) as u32,
            queens,
            occ,
            quiet,
            target,
        }
    }

    fn next_quiet(&mut self) -> Option<Move> {
        if self.queens == 0 || (self.quiet == 0 && self.queens.count_ones() == 1) {
            return None;
        }

        let mut from = self.queens.trailing_zeros();

        if self.quiet == 0 {
            self.queens &= self.queens - 1;
            from = self.queens.trailing_zeros();
            self.quiet = (MT::bishop_moves(self.occ, from as usize)
                | MT::rook_moves(self.occ, from as usize))
                & !self.occ
                & self.target;
            if self.quiet == 0 {
                return None;
            }
        }

        let to = self.quiet.trailing_zeros();
        self.quiet &= self.quiet - 1;

        Some(Move::new(from, to, self.piece, 0, MoveType::Quiet))
    }
}

impl Iterator for QueenQuietIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_quiet()
    }
}

pub struct QueenAttackIterator<'a> {
    board: &'a Board,
    piece: u32,
    queens: u64,
    occ: u64,
    attack: u64,
    opp: u64,
}

impl<'a> QueenAttackIterator<'a> {
    pub fn new(b: &Board, pinned: u64, target: u64) -> QueenAttackIterator {
        let queens = b.pieces[QUEEN + b.ctm] & !pinned;
        let opp = b.util[b.ctm ^ 1] & target;
        let occ = b.util[ALL_PIECES];
        let from = queens.trailing_zeros() % 64;
        let attack =
            (MT::bishop_moves(occ, from as usize) | MT::rook_moves(occ, from as usize)) & opp;

        QueenAttackIterator {
            board: b,
            piece: (BISHOP + b.ctm) as u32,
            queens,
            occ,
            attack,
            opp,
        }
    }

    fn next_attack(&mut self) -> Option<Move> {
        if self.queens == 0 || (self.attack == 0 && self.queens.count_ones() == 1) {
            return None;
        }

        let mut from = self.queens.trailing_zeros();

        if self.attack == 0 {
            self.queens &= self.queens - 1;
            from = self.queens.trailing_zeros();
            self.attack = (MT::bishop_moves(self.occ, from as usize)
                | MT::rook_moves(self.occ, from as usize))
                & self.opp;
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

impl<'a> Iterator for QueenAttackIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_attack()
    }
}

#[test]
fn bishop_moves_iter() {
    crate::init();
    let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq -").unwrap();
    let wquiet = QueenQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    let moves: Vec<_> = wquiet.collect();
    moves.iter().for_each(|m| println!("{m}"));
    assert_eq!(moves.len(), 6);

    let wattack = QueenAttackIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(wattack.count(), 0);

    let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 b kq -").unwrap();

    let bquiet = QueenQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(bquiet.count(), 4);

    let battack = QueenAttackIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(battack.count(), 4);
}
