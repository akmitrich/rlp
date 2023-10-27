use super::{
    program::{CharacterClass, Code, Context},
    Match,
};

pub fn compile(re: &str) -> Regex {
    Regex::new(re)
}

pub fn regex_match<'a>(re: &'a Regex, subj: &'a str) -> Vec<Match<'a>> {
    let mut matches = vec![];
    let input = subj.chars().collect::<Vec<_>>();
    let mut ctx = Context::new(&re.program, &input);
    while ctx.subj_pointer < input.len() {
        if super::program::exec(&mut ctx) {
            if !re.anchor_end || ctx.subj_pointer >= input.len() {
                let found = Match {
                    subj,
                    captures: (0..(re.captures + 1))
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
        if re.anchor_start {
            break;
        }
        ctx.program_counter = 0;
    }
    matches
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
        let mut captures = 1;
        let anchor_start = re.starts_with('^');
        let anchor_end = re.ends_with('$');

        let re = re.strip_prefix('^').unwrap_or(re);
        let re = re.strip_suffix('$').unwrap_or(re);
        let mut re = re.chars().peekable();

        while let Some(c) = re.next() {
            let pc = program.len();
            match c {
                '?' => program.insert(pc - 1, Code::Split { x: pc, y: pc + 1 }),
                '+' => program.push(Code::Split {
                    x: pc - 1,
                    y: pc + 1,
                }),
                '*' => {
                    program.insert(pc - 1, Code::Split { x: pc, y: pc + 2 });
                    program.push(Code::Jmp(pc - 1));
                }
                '-' => {
                    program.insert(pc - 1, Code::Split { x: pc + 2, y: pc });
                    program.push(Code::Jmp(pc - 1));
                }
                '(' => program.push(Code::Save(2 * captures)),
                ')' => {
                    program.push(Code::Save(2 * captures + 1));
                    captures += 1;
                    if captures > 9 {
                        panic!("Too many captures.")
                    }
                }
                '[' => {
                    let un = if re.peek() == Some('^').as_ref() {
                        re.next();
                        true
                    } else {
                        false
                    };
                    let mut set = vec![];
                    while let Some(c) = re.next() {
                        match c {
                            ']' => break,
                            '%' => set.push(take_escaped_class(&mut re)),
                            _ => {
                                set.push(CharacterClass::Literal(c));
                            }
                        }
                    }
                    program.push(Code::Char(if un {
                        CharacterClass::Unset(set)
                    } else {
                        CharacterClass::Set(set)
                    }));
                }
                '.' => program.push(Code::Char(CharacterClass::Any)),
                '%' => program.push(take_escaped_code(&mut re)),
                c => program.push(Code::Char(CharacterClass::Literal(c))),
            }
        }

        program.push(Code::Save(1));
        program.push(Code::Match);
        captures -= 1; // 0th capture is the matched pattern if it's empty the subject is not matched

        Self {
            program,
            anchor_start,
            anchor_end,
            captures,
        }
    }
}

fn take_escaped_code<I>(re: &mut I) -> Code
where
    I: Iterator<Item = char>,
{
    if let Some(c) = re.next() {
        match c {
            '1'..='9' => Code::Captured(c.to_digit(10).unwrap() as _),
            _ => Code::Char(char_to_class(c)),
        }
    } else {
        panic!("Inappropriate escaping.")
    }
}

fn take_escaped_class<I>(re: &mut I) -> CharacterClass
where
    I: Iterator<Item = char>,
{
    if let Some(c) = re.next() {
        char_to_class(c)
    } else {
        panic!("Inappropriate escaping.")
    }
}

fn char_to_class(c: char) -> CharacterClass {
    let is_in = c.is_ascii_lowercase();
    match c.to_ascii_lowercase() {
        'w' => CharacterClass::AlphaNumeric(is_in),
        'a' => CharacterClass::Letter(is_in),
        'c' => CharacterClass::ControlChar(is_in),
        'd' => CharacterClass::Digit(is_in),
        'g' => CharacterClass::Printable(is_in),
        'l' => CharacterClass::Lowercase(is_in),
        'p' => CharacterClass::Punctuation(is_in),
        's' => CharacterClass::WhiteSpace(is_in),
        'u' => CharacterClass::Uppercase(is_in),
        'x' => CharacterClass::Hexadecimal(is_in),
        c @ ('%' | '\\' | '.' | '*' | '+' | '-' | '?') => CharacterClass::Literal(c),
        _ => panic!("Illegal char in escaping."),
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
