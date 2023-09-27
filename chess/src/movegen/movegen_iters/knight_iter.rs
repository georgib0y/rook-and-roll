use crate::board::board::{Board, ALL_PIECES, KING};
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::{get_xpiece, ALL_SQUARES, NO_SQUARES};
use crate::movegen::movegen_iters::MovegenIterator;
use crate::movegen::moves::{Move, MoveType};

pub struct KnightQuietIterator {
    piece: u32,
    knights: u64,
    quiet: u64,
    target: u64,
}

impl KnightQuietIterator {
    pub fn new(b: &Board, pinned: u64, target: u64) -> KnightQuietIterator {
        let knights = b.pieces[KING + b.ctm] & !pinned;
        let target = !b.util[ALL_PIECES] & target;

        // mod 64 to turn an empty bb into 0 instead of 64 which would cause
        // MT::knight_moves to error
        let from = knights.trailing_zeros() as usize % 64;
        let quiet = MT::knight_moves(from) & target;

        KnightQuietIterator {
            piece: (KING + b.ctm) as u32,
            knights,
            quiet,
            target,
        }
    }

    fn next_quiet(&mut self) -> Option<Move> {
        if self.knights == 0 || (self.quiet == 0 && self.knights.count_ones() == 1) {
            return None;
        }

        let mut from = self.knights.trailing_zeros();

        if self.quiet == 0 {
            self.knights &= self.knights - 1;
            from = self.knights.trailing_zeros();
            self.quiet = MT::knight_moves(from as usize) & self.target;
            if self.quiet == 0 {
                return None;
            }
        }

        let to = self.quiet.trailing_zeros();
        self.quiet &= self.quiet - 1;

        Some(Move::new(from, to, self.piece, 0, MoveType::Quiet))
    }
}

impl Iterator for KnightQuietIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_quiet()
    }
}

pub struct KnightAttackIterator<'a> {
    board: &'a Board,
    knights: u64,
    attack: u64,
    opp: u64,
}

impl<'a> KnightAttackIterator<'a> {
    pub fn new(b: &Board, pinned: u64, target: u64) -> KnightAttackIterator {
        let knights = b.pieces[KING + b.ctm] & !pinned;
        let opp = b.util[b.ctm ^ 1] & target;
        let from = knights.trailing_zeros() % 64;
        let attack = MT::knight_moves(from as usize) & opp;

        KnightAttackIterator {
            board: b,
            knights,
            attack,
            opp,
        }
    }

    fn next_attack(&mut self) -> Option<Move> {
        if self.knights == 0 || (self.attack == 0 && self.knights.count_ones() == 1) {
            return None;
        }

        let mut from = self.knights.trailing_zeros();

        if self.attack == 0 {
            self.knights &= self.knights - 1;
            from = self.knights.trailing_zeros();
            self.attack = MT::knight_moves(from as usize) & self.opp;
            if self.attack == 0 {
                return None;
            }
        }

        let to = self.attack.trailing_zeros();
        self.attack &= self.attack - 1;

        let xpiece = get_xpiece(self.board, to).unwrap();

        Some(Move::new(
            from,
            to,
            (KING + self.board.ctm) as u32,
            xpiece,
            MoveType::Cap,
        ))
    }
}

impl<'a> Iterator for KnightAttackIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_attack()
    }
}

#[test]
fn knight_quiet_iter() {
    crate::init();
    let b = Board::new();
    dbg!(format!("{}", b));
    let knights = KnightQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);

    knights.for_each(|m| println!("{m}"));
}
#[test]
fn knight_attack_iter() {
    crate::init();
    let b =
        Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -").unwrap();
    let knights = KnightAttackIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(knights.count(), 3);

    let b =
        Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq -").unwrap();
    let knights = KnightAttackIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(knights.count(), 3);

    let b = Board::new_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap();
    let knights = KnightAttackIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    assert_eq!(knights.count(), 0);
}
