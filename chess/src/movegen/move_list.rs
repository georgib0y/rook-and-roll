use crate::movegen::moves::Move;
use std::array::IntoIter;

const MAX_MOVES: usize = 214;

pub trait MoveList: Iterator<Item = Move> {
    fn add_move(&mut self, m: Move);
    fn sort(&mut self);
}

pub struct StackMoveList<const N: usize = MAX_MOVES> {
    move_list: [Move; N],
    length: usize,
    count: usize,
}

impl<const N: usize> StackMoveList<N> {
    pub fn new() -> StackMoveList<N> {
        StackMoveList {
            move_list: [Move::empty(); N],
            length: 0,
            count: 0,
        }
    }
}

impl Iterator for StackMoveList {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.length {
            return None;
        }

        let m = self.move_list[self.count];
        self.count += 1;

        Some(m)
    }
}

impl MoveList for StackMoveList {
    fn add_move(&mut self, m: Move) {
        self.move_list[self.length] = m;
        self.length += 1;
    }

    fn sort(&mut self) {
        todo!();
    }
}
