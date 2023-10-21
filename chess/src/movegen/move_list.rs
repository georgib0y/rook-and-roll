use crate::board::board::Board;
use crate::movegen::move_scoring::score_move;
use crate::movegen::moves::Move;
use crate::search::search::Searcher;

pub const MAX_MOVES: usize = 214;

pub trait MoveList: IntoIterator<Item = Move> {
    fn add_move(&mut self, m: Move);
    fn len(&self) -> usize;
}

pub struct StackMoveList<const N: usize = MAX_MOVES> {
    moves: [Move; N],
    count: usize,
    length: usize,
}

impl<const N: usize> StackMoveList<N> {
    pub fn new() -> StackMoveList<N> {
        StackMoveList {
            moves: [Move::empty(); N],
            count: 0,
            length: 0,
        }
    }
}

impl Default for StackMoveList {
    fn default() -> Self {
        StackMoveList::<MAX_MOVES>::new()
    }
}

impl<const N: usize> Iterator for StackMoveList<N> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.length {
            return None;
        }

        let m = self.moves[self.count];
        self.count += 1;

        Some(m)
    }
}

impl<const N: usize> MoveList for StackMoveList<N> {
    fn add_move(&mut self, m: Move) {
        self.moves[self.length] = m;
        self.length += 1;
    }

    fn len(&self) -> usize {
        self.length
    }
}

pub struct ScoredMoveList<'a, S: Searcher, const N: usize = MAX_MOVES> {
    moves: [(Move, i32); N],
    count: usize,
    length: usize,
    board: &'a Board,
    searcher: &'a S,
    depth: usize,
}

impl<'a, S: Searcher, const N: usize> ScoredMoveList<'a, S, N> {
    pub fn new(board: &'a Board, searcher: &'a S, depth: usize) -> ScoredMoveList<'a, S, N> {
        ScoredMoveList {
            moves: [(Move::empty(), 0); N],
            count: 0,
            length: 0,
            board,
            searcher,
            depth,
        }
    }
}

impl<'a, S: Searcher, const N: usize> IntoIterator for ScoredMoveList<'a, S, N> {
    type Item = Move;
    type IntoIter = ScoreMoveListIter<N>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.moves[0..self.length].sort_unstable_by(|m1, m2| m2.1.cmp(&m1.1));
        ScoreMoveListIter::new(self.moves, self.length)
    }
}

impl<'a, S: Searcher, const N: usize> MoveList for ScoredMoveList<'a, S, N> {
    fn add_move(&mut self, m: Move) {
        let score = score_move(self.board, self.searcher, self.depth, m);

        self.moves[self.length] = (m, score);
        self.length += 1;
    }

    fn len(&self) -> usize {
        self.length
    }
}

pub struct ScoreMoveListIter<const N: usize> {
    moves: [(Move, i32); N],
    length: usize,
    count: usize,
}

impl<const N: usize> ScoreMoveListIter<N> {
    fn new(moves: [(Move, i32); N], length: usize) -> ScoreMoveListIter<N> {
        ScoreMoveListIter {
            moves,
            length,
            count: 0,
        }
    }
}

impl<const N: usize> Iterator for ScoreMoveListIter<N> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.length {
            return None;
        }

        let m = self.moves[self.count].0;
        self.count += 1;
        Some(m)
    }
}
