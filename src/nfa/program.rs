use std::ops::Range;

pub(super) fn exec(ctx: &mut Context) -> bool {
    loop {
        match &ctx.program[ctx.program_counter] {
            Code::Char(c) => {
                let other = ctx.subj.get(ctx.subj_pointer);
                if other.is_some() && c.is_matched(other.unwrap()) {
                    ctx.program_counter += 1;
                    ctx.subj_pointer += 1;
                } else {
                    return false;
                }
            }
            Code::Captured(n) => {
                let old = ctx.subj_pointer;
                for i in ctx.captured_range(*n) {
                    if ctx.subj[ctx.subj_pointer] == ctx.subj[i] {
                        ctx.subj_pointer += 1;
                    } else {
                        ctx.subj_pointer = old;
                        return false;
                    }
                }
                ctx.program_counter += 1;
            }
            Code::Jmp(x) => ctx.program_counter = *x,
            Code::Split { x, y } => {
                ctx.program_counter = *x;
                if exec(ctx) {
                    return true;
                } else {
                    ctx.program_counter = *y;
                }
            }
            Code::Save(x) => {
                let slot = *x;
                let old = ctx.saved[slot];
                ctx.saved[slot] = ctx.subj_pointer;
                ctx.program_counter += 1;
                if exec(ctx) {
                    return true;
                } else {
                    ctx.saved[slot] = old;
                    return false;
                }
            }
            Code::Match => return true,
        }
    }
}

#[derive(Debug)]
pub(super) enum Code {
    Char(CharacterClass),
    Captured(usize),
    Jmp(usize),
    Split { x: usize, y: usize },
    Save(usize),
    Match,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum CharacterClass {
    Literal(char),
    Any,
    AlphaNumeric(bool),
    Letter(bool),
    ControlChar(bool),
    Digit(bool),
    Printable(bool),
    Lowercase(bool),
    Punctuation(bool),
    WhiteSpace(bool),
    Uppercase(bool),
    Hexadecimal(bool),
    Set(Vec<CharacterClass>),
    Unset(Vec<CharacterClass>),
}

#[derive(Debug, Default)]
pub(super) struct Context<'a> {
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

impl CharacterClass {
    pub(super) fn is_matched(&self, other: &char) -> bool {
        match self {
            CharacterClass::Literal(c) => c == other,
            CharacterClass::Any => true,
            CharacterClass::AlphaNumeric(is_in) => {
                if *is_in {
                    other.is_alphanumeric()
                } else {
                    !other.is_alphanumeric()
                }
            }
            CharacterClass::Letter(is_in) => {
                if *is_in {
                    other.is_alphabetic()
                } else {
                    !other.is_alphabetic()
                }
            }
            CharacterClass::ControlChar(is_in) => {
                if *is_in {
                    other.is_ascii_control()
                } else {
                    !other.is_ascii_control()
                }
            }
            CharacterClass::Digit(is_in) => {
                if *is_in {
                    other.is_numeric()
                } else {
                    !other.is_numeric()
                }
            }
            CharacterClass::Printable(is_in) => {
                if *is_in {
                    other.is_ascii_graphic() && other != &' '
                } else {
                    !other.is_ascii_graphic() || other == &' '
                }
            }
            CharacterClass::Lowercase(is_in) => {
                if *is_in {
                    other.to_lowercase().next() == Some(*other)
                } else {
                    other.to_lowercase().next() != Some(*other)
                }
            }
            CharacterClass::Punctuation(is_in) => {
                if *is_in {
                    other.is_ascii_punctuation()
                } else {
                    !other.is_ascii_punctuation()
                }
            }
            CharacterClass::WhiteSpace(is_in) => {
                if *is_in {
                    other.is_whitespace()
                } else {
                    !other.is_whitespace()
                }
            }
            CharacterClass::Uppercase(is_in) => {
                if *is_in {
                    other.to_uppercase().next() == Some(*other)
                } else {
                    other.to_uppercase().next() != Some(*other)
                }
            }
            CharacterClass::Hexadecimal(is_in) => {
                if *is_in {
                    other.is_ascii_hexdigit()
                } else {
                    !other.is_ascii_hexdigit()
                }
            }
            CharacterClass::Set(s) => s.iter().any(|x| x.is_matched(other)),
            CharacterClass::Unset(s) => s.iter().all(|x| !x.is_matched(other)),
        }
    }
}
