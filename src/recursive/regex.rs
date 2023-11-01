use super::{
    lexer::{lex, Lex, Quantifier},
    program::{CharacterClass, Code, Context},
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
        let anchor_start = re.starts_with('^');
        let anchor_end = re.ends_with('$');
        let re = re.strip_prefix('^').unwrap_or(re);
        let re = re.strip_suffix('$').unwrap_or(re);

        for (lex, quantifier) in lex(re) {
            if let Lex::SaveOpen(n) = &lex {
                if *n > captures {
                    captures = *n;
                }
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

        Self {
            program: prog.into_boxed_slice(),
            anchor_start,
            anchor_end,
            captures,
        }
    }

    pub fn match_all<'a>(&self, subj: &'a str) -> Vec<Match<'a>> {
        let mut matches = vec![];
        let input = subj.char_indices().collect::<Vec<_>>();
        println!("Input: {:?}", input.iter().enumerate().collect::<Vec<_>>());
        let mut ctx = Context::new(&self.program, &input);
        while ctx.subj_pointer < input.len() {
            if super::program::exec(&mut ctx) {
                if !self.anchor_end || ctx.subj_pointer >= input.len() {
                    let found = Match {
                        subj,
                        captures: (0..(self.captures + 1))
                            .map(|n| &subj[ctx.captured_range(n)])
                            .collect(),
                    };
                    println!("Found: {:?}", ctx.saved);
                    if found.captures.first().unwrap().is_empty() {
                        ctx.subj_pointer += 1;
                    }
                    matches.push(found);
                }
            } else {
                ctx.subj_pointer += 1;
            }
            if self.anchor_start {
                break;
            }
            ctx.program_counter = 0;
        }
        matches
    }
}

fn code_for_lex(lex: Lex) -> Code {
    match lex {
        Lex::AnyChar => Code::Char(CharacterClass::Any),
        Lex::Literal(c) => Code::Char(CharacterClass::Literal(c)),
        Lex::CharacterClass(c) | Lex::CharacterSet(c) => Code::Char(c),
        Lex::Captured(n) => Code::Captured(n),
        Lex::Border(x, y) => Code::Border(x, y),
        Lex::SaveOpen(n) => Code::Save(2 * n),
        Lex::SaveClose(n) => Code::Save(2 * n + 1),
    }
}

type _ThreadList = std::collections::VecDeque<usize>;

fn _thompsonvm(program: &[Code], input: &str) -> bool {
    let mut clist = _ThreadList::new();
    let mut nlist = _ThreadList::new();
    clist.push_back(0);
    for data in input.chars() {
        while !clist.is_empty() {
            let pc = clist.pop_front().unwrap();
            let inst = program.get(pc).unwrap();
            match inst {
                Code::Char(command) => {
                    if command.is_matched(&data) {
                        nlist.push_back(pc + 1);
                    }
                }
                Code::Captured(_) => todo!(),
                Code::Border(_, _) => todo!(),
                Code::Match => return true,
                Code::Jmp(x) => {
                    clist.push_back(*x);
                }
                Code::Split { x, y } => {
                    clist.push_back(*x);
                    clist.push_back(*y);
                }
                Code::Save(_) => unimplemented!(),
            }
        }
        std::mem::swap(&mut clist, &mut nlist);
        nlist.clear();
    }
    !clist.is_empty()
}
