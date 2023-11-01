mod bytecode;
mod lexer;
mod recursive;
pub mod regex;

#[derive(Debug)]
pub struct Match<'a> {
    pub subj: &'a str,
    pub captures: Box<[&'a str]>,
}
