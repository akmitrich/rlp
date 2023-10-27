use std::ops::Range;

use super::Code;

#[derive(Debug, Default)]
pub struct Context<'a> {
    pub program: &'a [Code],
    pub subj: &'a [char],
    pub program_counter: usize,
    pub subj_pointer: usize,
    pub saved: [usize; 20],
}

impl<'a> Context<'a> {
    pub fn new(program: &'a [Code], subj: &'a [char]) -> Self {
        Self {
            program,
            subj,
            ..Default::default()
        }
    }

    pub fn captured_range(&self, n: usize) -> Range<usize> {
        self.saved[2 * n]..self.saved[2 * n + 1]
    }
}
