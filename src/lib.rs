use std::ops::Range;

mod bytecode;
mod recursive;
pub mod regex;

#[derive(Debug)]
pub struct Match<'a> {
    pub subj: &'a str,
    pub captures: Box<[Range<usize>]>,
}

#[derive(Debug)]
pub enum Capture<'a> {
    Value(&'a str),
    Index(usize),
}

impl Match<'_> {
    pub fn capture(&self, n: usize) -> Option<Capture> {
        self.captures.get(n).map(|r| {
            if r.is_empty() {
                Capture::Index(r.start)
            } else {
                Capture::Value(&self.subj[r.to_owned()])
            }
        })
    }

    pub fn captures(&self) -> Box<[Capture]> {
        self.captures_iter().collect()
    }

    pub fn captures_iter(&self) -> impl Iterator<Item = Capture> {
        (0..self.captures.len())
            .map(|n| self.capture(n))
            .map(Option::unwrap)
    }
}
