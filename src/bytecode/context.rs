use super::code::Code;
use std::ops::Range;

#[derive(Debug, Default)]
pub struct Context<'a> {
    pub program: &'a [Code],
    pub subj: &'a [(usize, char)],
    pub program_counter: usize,
    pub subj_pointer: usize,
    pub saved: [usize; 20],
}

impl<'a> Context<'a> {
    pub fn new(program: &'a [Code], subj: &'a [(usize, char)]) -> Self {
        Self {
            program,
            subj,
            ..Default::default()
        }
    }

    pub fn saved_range(&self, n: usize) -> Range<usize> {
        self.saved[2 * n]..self.saved[2 * n + 1]
    }

    pub fn captured_range(&self, n: usize) -> Range<usize> {
        let begin = self.subj[self.saved[2 * n]].0;
        let end_index = self.saved[2 * n + 1];
        let end = if end_index == self.subj.len() {
            self.subj
                .last()
                .map(|(i, c)| *i + c.len_utf8())
                .unwrap_or(0)
        } else {
            self.subj[end_index].0
        };
        begin..end
    }
}
