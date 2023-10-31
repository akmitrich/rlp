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
    program: Vec<Code>,
    anchor_start: bool,
    anchor_end: bool,
    captures: usize,
}

impl Regex {
    pub fn new(re: &str) -> Self {
        let mut program = vec![Code::Save(0)];
        let mut saves = vec![];
        let mut captures = 0;
        let anchor_start = re.starts_with('^');
        let anchor_end = re.ends_with('$');

        let re = re.strip_prefix('^').unwrap_or(re);
        let re = re.strip_suffix('$').unwrap_or(re);
        let re = lex(re);

        for (lex, quantifier) in re {
            match lex {
                Lex::AnyChar => program.push(Code::Char(CharacterClass::Any)),
                Lex::Literal(c) => program.push(Code::Char(CharacterClass::Literal(c))),
                Lex::CharacterClass(c) => program.push(Code::Char(c)),
                Lex::CharacterSet(s) => program.push(Code::Char(s)),
                Lex::Captured(n) => {
                    assert_eq!(Quantifier::ExactlyOne, quantifier);
                    program.push(Code::Captured(n))
                }
                Lex::Border(x, y) => {
                    assert_eq!(Quantifier::ExactlyOne, quantifier);
                    program.push(Code::Border(x, y))
                }
                Lex::SaveOpen => {
                    assert_eq!(Quantifier::ExactlyOne, quantifier);
                    captures += 1;
                    if captures > 9 {
                        panic!("Too many captures.")
                    }
                    saves.push(captures);
                    program.push(Code::Save(2 * captures));
                }
                Lex::SaveClose => {
                    assert_eq!(Quantifier::ExactlyOne, quantifier);
                    let captured = saves.pop().unwrap();
                    program.push(Code::Save(2 * captured + 1));
                }
            }
            let pc = program.len();
            match quantifier {
                Quantifier::ExactlyOne => {}
                Quantifier::ZeroOrOne => program.insert(pc - 1, Code::Split { x: pc, y: pc + 1 }),
                Quantifier::OneOrMany => program.push(Code::Split {
                    x: pc - 1,
                    y: pc + 1,
                }),
                Quantifier::ZeroOrManyGreedy => {
                    program.insert(pc - 1, Code::Split { x: pc, y: pc + 2 });
                    program.push(Code::Jmp(pc - 1));
                }
                Quantifier::ZeroOrManyUngreedy => {
                    program.insert(pc - 1, Code::Split { x: pc + 2, y: pc });
                    program.push(Code::Jmp(pc - 1));
                }
            }
        }

        program.push(Code::Save(1));
        program.push(Code::Match);
        assert!(saves.is_empty());

        Self {
            program,
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
