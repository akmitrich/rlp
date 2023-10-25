#[derive(Debug)]
pub struct Regex {
    program: Vec<Code>,
    anchor_start: bool,
    anchor_end: bool,
    captures: usize,
}

#[derive(Debug)]
pub struct Match<'a> {
    pub subj: &'a str,
    pub captures: Vec<&'a str>,
}

#[derive(Debug)]
pub enum Code {
    Char(CharacterClass),
    Jmp(usize),
    Split { x: usize, y: usize },
    Save(usize),
    Match,
}

#[derive(Debug)]
pub enum CharacterClass {
    Literal(char),
    WordChar,
}

impl CharacterClass {
    pub fn is_matched(&self, other: &char) -> bool {
        match self {
            CharacterClass::Literal(c) => c == other,
            CharacterClass::WordChar => other.is_alphanumeric() || other == &'_',
        }
    }
}

pub fn parse(re: &str) -> Regex {
    let mut regex = Regex {
        program: vec![Code::Save(0)],
        captures: 0,
        anchor_start: false,
        anchor_end: false,
    };
    if re.starts_with('^') {
        regex.anchor_start = true
    }
    let re = re.strip_prefix('^').unwrap_or(re);
    let mut re = re.chars();
    let mut k = 1;
    while let Some(c) = re.next() {
        let pc = regex.program.len();
        match c {
            '?' => regex
                .program
                .insert(pc - 1, Code::Split { x: pc, y: pc + 1 }),
            '+' => regex.program.push(Code::Split {
                x: pc - 1,
                y: pc + 1,
            }),
            '-' => regex.program.push(Code::Split {
                x: pc + 1,
                y: pc - 1,
            }),
            '*' => {
                regex
                    .program
                    .insert(pc - 1, Code::Split { x: pc, y: pc + 2 });
                regex.program.push(Code::Jmp(pc - 1));
            }
            '(' => regex.program.push(Code::Save(2 * k)),
            ')' => {
                regex.program.push(Code::Save(2 * k + 1));
                k += 1;
                if k > 9 {
                    panic!("Too many captures.")
                }
            }
            '%' => {
                if let Some(c) = re.next() {
                    match c {
                        'w' => regex.program.push(Code::Char(CharacterClass::WordChar)),
                        c @ ('%' | '*') => {
                            regex.program.push(Code::Char(CharacterClass::Literal(c)))
                        }
                        _ => panic!("Illegal char in escaping."),
                    }
                } else {
                    panic!("Inappropriate escaping.")
                }
            }
            c => regex.program.push(Code::Char(CharacterClass::Literal(c))),
        }
    }
    regex.program.push(Code::Save(1));
    regex.program.push(Code::Match);
    regex.captures = k - 1;
    regex
}

pub fn regex_match<'a>(re: &'a Regex, subj: &'a str) -> Vec<Match<'a>> {
    let mut matches = vec![];
    let mut sp = 0;
    let mut saved = [0; 20];
    let input = subj.chars().collect::<Vec<_>>();
    while sp < subj.len() {
        if let Some(parsed) = exec(&re.program, input.as_slice(), 0, sp, &mut saved) {
            matches.push(Match {
                subj,
                captures: (0..(re.captures + 1))
                    .map(|n| &subj[saved[2 * n]..saved[2 * n + 1]])
                    .collect(),
            });
            sp = parsed;
        } else {
            sp += 1;
        }
        if re.anchor_start {
            break;
        }
    }
    if re.anchor_end && sp < subj.len() {
        vec![]
    } else {
        matches
    }
}

type ThreadList = std::collections::VecDeque<usize>;

fn exec(
    program: &[Code],
    subj: &[char],
    mut pc: usize,
    mut sp: usize,
    saved: &mut [usize],
) -> Option<usize> {
    loop {
        match &program[pc] {
            Code::Char(c) => {
                let other = subj.get(sp)?;
                if c.is_matched(other) {
                    pc += 1;
                    sp += 1;
                } else {
                    return None;
                }
            }
            Code::Jmp(x) => pc = *x,
            Code::Split { x, y } => {
                if let Some(sp) = exec(program, subj, *x, sp, saved) {
                    return Some(sp);
                } else {
                    pc = *y;
                }
            }
            Code::Save(x) => {
                let slot = *x;
                let old = saved[slot];
                saved[slot] = sp;
                if let Some(sp) = exec(program, subj, pc + 1, sp, saved) {
                    return Some(sp);
                } else {
                    saved[slot] = old;
                    return None;
                }
            }
            Code::Match => return Some(sp),
        }
    }
}

pub fn thompsonvm(program: &[Code], input: &str) -> bool {
    let mut clist = ThreadList::new();
    let mut nlist = ThreadList::new();
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

#[cfg(test)]
mod test {
    use pcre2::bytes::RegexBuilder;

    use super::*;

    #[test]
    fn it_works() {
        let regex = parse("%w+&?");
        let subj = "bab__&&&ghi";
        let m = regex_match(&regex, subj);
        let pcre = RegexBuilder::new()
            .ucp(true)
            .utf(true)
            .build(r"\w+&?")
            .unwrap();
        let pcre_m = pcre.find_iter(b"bab__&&&ghi").collect::<Vec<_>>();
        assert_eq!(pcre_m.len(), m.len());
        for (m, pcre_m) in m.iter().zip(pcre_m.iter().map(|m| m.as_ref().unwrap())) {
            let s1 = *m.captures.first().unwrap();
            let s2 = &subj[pcre_m.start()..pcre_m.end()];
            assert_eq!(s1, s2);
        }
    }
}
