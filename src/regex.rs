use std::ops::Range;

use crate::{
    bytecode::{
        character_class::CharacterClass,
        code::Code,
        context::Context,
        lexer::{lex, PatternElement, Quantifier},
    },
    input::Input,
    Match,
};

pub fn compile(re: &str) -> Regex {
    Regex::new(re)
}

#[derive(Debug)]
pub struct Regex {
    program: Box<[Code]>,
    anchor_start: bool,
    anchor_end: bool,
    captures: usize,
}

impl Regex {
    pub fn new(re: &str) -> Self {
        let mut prog = vec![Code::Save(0)];
        let mut captures = 0;
        let mut saves = 0;
        let anchor_start = re.starts_with('^');
        let anchor_end = re.ends_with('$');
        let re = re.strip_prefix('^').unwrap_or(re);
        let re = re.strip_suffix('$').unwrap_or(re);

        for (lex, quantifier) in lex(re) {
            if let PatternElement::SaveOpen(n) = &lex {
                saves += 1;
                if *n > captures {
                    captures = *n;
                }
            }
            if let PatternElement::SaveClose(_) = &lex {
                saves -= 1;
            }
            let code = code_for_lex(lex);
            let pc = prog.len();
            match quantifier {
                Quantifier::ExactlyOne => {
                    prog.push(code);
                }
                Quantifier::ZeroOrOne => {
                    prog.push(Code::Split {
                        x: pc + 1,
                        y: pc + 2,
                    });
                    prog.push(code)
                }
                Quantifier::OneOrMany => {
                    prog.push(code);
                    prog.push(Code::Split { x: pc, y: pc + 2 });
                }
                Quantifier::ZeroOrManyGreedy => {
                    prog.push(Code::Split {
                        x: pc + 1,
                        y: pc + 3,
                    });
                    prog.push(code);
                    prog.push(Code::Jmp(pc));
                }
                Quantifier::ZeroOrManyUngreedy => {
                    prog.push(Code::Split {
                        x: pc + 3,
                        y: pc + 1,
                    });
                    prog.push(code);
                    prog.push(Code::Jmp(pc));
                }
            }
        }
        prog.push(Code::Save(1));
        prog.push(Code::Match);

        assert_eq!(0, saves);
        Self {
            program: prog.into_boxed_slice(),
            anchor_start,
            anchor_end,
            captures,
        }
    }

    pub fn match_one<'a>(&self, subj: &'a str) -> Option<Match<'a>> {
        let mut ctx = Context::new(&self.program, Input::new(subj));
        self.find_match(&mut ctx)
            .map(|captures| Match { subj, captures })
    }

    pub fn match_all<'a>(&self, subj: &'a str) -> Box<[Match<'a>]> {
        let mut matches = vec![];
        let mut ctx = Context::new(&self.program, Input::new(subj));
        while !ctx.exhausted() {
            if let Some(captures) = self.find_match(&mut ctx) {
                matches.push(Match { subj, captures });
            }
            if self.anchor_start {
                break;
            }
        }
        matches.into()
    }
}

impl Regex {
    fn find_match(&self, ctx: &mut Context) -> Option<Box<[Range<usize>]>> {
        let mut found = None;
        while found.is_none() && !ctx.exhausted() {
            if crate::recursive::exec(ctx) {
                if !self.anchor_end || ctx.exhausted() {
                    let matches = self.captured_ranges(ctx);
                    println!("Found: {:?}", ctx.saved);
                    if matches.first().unwrap().is_empty() {
                        ctx.subj_pointer += 1;
                    }
                    found = Some(matches);
                }
            } else {
                ctx.subj_pointer += 1;
            }
            ctx.program_counter = 0;
            if self.anchor_start {
                break;
            }
        }
        found
    }

    fn captured_ranges(&self, ctx: &Context) -> Box<[Range<usize>]> {
        (0..(self.captures + 1))
            .map(|n| ctx.captured_range(n))
            .collect()
    }
}

fn code_for_lex(lex: PatternElement) -> Code {
    match lex {
        PatternElement::AnyChar => Code::Char(CharacterClass::Any),
        PatternElement::Literal(c) => Code::Char(CharacterClass::Literal(c)),
        PatternElement::CharacterClass(c) | PatternElement::CharacterSet(c) => Code::Char(c),
        PatternElement::Captured(n) => Code::Captured(n),
        PatternElement::Border(x, y) => Code::Border(x, y),
        PatternElement::SaveOpen(n) => Code::Save(2 * n),
        PatternElement::SaveClose(n) => Code::Save(2 * n + 1),
        PatternElement::Frontier(s) => Code::Frontier(s),
    }
}

#[cfg(test)]
mod test {
    use super::{compile, CharacterClass::*, Code::*};

    #[test]
    fn it_works() {
        assert_eq!(
            [
                Save(0),
                Char(Literal('a')),
                Char(Literal('b')),
                Char(Literal('c')),
                Save(1),
                Match
            ]
            .as_slice(),
            compile("abc").program.as_ref()
        )
    }

    #[test]
    fn char_classes_with_quantifiers() {
        assert_eq!(
            [
                Save(0),
                Char(Digit(true)),
                Split { x: 1, y: 3 },
                Split { x: 4, y: 6 },
                Char(Letter(true)),
                Jmp(3),
                Split { x: 9, y: 7 },
                Char(Hexadecimal(false)),
                Jmp(6),
                Split { x: 10, y: 11 },
                Char(Unset(Box::new([
                    AlphaNumeric(true),
                    Literal('_'),
                    Literal('.')
                ]))),
                Save(1),
                Match
            ]
            .as_slice(),
            compile("%d+%a*%X-[^%w_%.]?").program.as_ref()
        )
    }
}
