pub mod eval;
pub mod search;
pub mod searchers;
pub mod tt;

#[derive(Clone)]
pub struct HistoryTable {
    history: Box<[[[u32; 64]; 64]; 2]>,
}

impl HistoryTable {
    pub fn new() -> HistoryTable {
        HistoryTable {
            history: Box::new([[[0; 64]; 64]; 2]),
        }
    }

    pub fn insert(&mut self, ctm: usize, from: usize, to: usize, depth: usize) {
        self.history[ctm][from][to] += (depth * depth) as u32
    }

    pub fn get(&self, ctm: usize, from: usize, to: usize) -> u32 {
        self.history[ctm][from][to]
    }
}
