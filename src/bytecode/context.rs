use crate::input::Input;

use super::code::Code;
use std::ops::Range;
#[derive(Debug)]
pub(crate) struct Context<'a> {
    pub program: &'a [Code],
    pub input: Input<'a>,
    pub program_counter: usize,
    pub subj_pointer: usize,
    pub saved: Box<[usize]>,
}

impl<'a> Context<'a> {
    pub fn new(program: &'a [Code], input: Input<'a>, captures: usize) -> Self {
        Self {
            program,
            input,
            program_counter: 0,
            subj_pointer: 0,
            saved: vec![0; 2 * captures + 2].into_boxed_slice(),
        }
    }
}

impl Context<'_> {
    pub fn exhausted(&self) -> bool {
        self.subj_pointer >= self.input.len()
    }

    pub fn saved_range(&self, n: usize) -> Range<usize> {
        self.saved[2 * n]..self.saved[2 * n + 1]
    }

    pub fn captured_range(&self, n: usize) -> Range<usize> {
        let begin = self.input.get_byte_index(self.saved[2 * n]).unwrap();
        let end = self
            .input
            .get_byte_index(self.saved[2 * n + 1])
            .unwrap_or(0);
        begin..end
    }

    pub fn captured_ranges(&self) -> Box<[Range<usize>]> {
        (0..self.saved.len() / 2)
            .map(|n| self.captured_range(n))
            .collect()
    }
}
