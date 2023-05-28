#![allow(unused)]

use std::sync::atomic::AtomicU32;
use crate::tt::ORDER;

pub trait HTable {
    fn get(&self, colour_to_move: usize, from: usize, to: usize) -> u32;
}

pub struct HistoryTable {
    history: Vec<[[u32; 64]; 64]>
}

impl HistoryTable {
    pub fn new() -> HistoryTable {
        let mut history = Vec::with_capacity(2 * 64 * 64);
        history.push([[0; 64]; 64]);
        history.push([[0; 64]; 64]);
        HistoryTable { history }
    }

    pub fn insert(&mut self, colour_to_move: usize, from: usize, to: usize, depth: usize) {
        self.history[colour_to_move][from][to] += (depth * depth) as u32
    }
}

impl HTable for HistoryTable {
    fn get(&self, colour_to_move: usize, from: usize, to: usize) -> u32 {
        self.history[colour_to_move][from][to]
    }
}

pub struct AtomicHistoryTable {
    history: Vec<AtomicU32>
}

impl AtomicHistoryTable {
    pub fn new() -> AtomicHistoryTable {
        let mut history = Vec::with_capacity(2 * 64 * 64);
        (0..2 * 64 * 64).for_each(|_| history.push(AtomicU32::new(0)));
        AtomicHistoryTable { history }
    }

    pub fn insert(&self, colour_to_move: usize, from: usize, to: usize, depth: usize) {
        self.history[colour_to_move*64*64 + from*64 + to]
            .store((depth*depth) as u32, ORDER)
    }
}

impl HTable for AtomicHistoryTable {
    fn get(&self, colour_to_move: usize, from: usize, to: usize) -> u32 {
        self.history[colour_to_move*64*64 + from*64 + to].load(ORDER)
    }
}
