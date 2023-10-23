#[derive(Debug)]
pub struct Regex {
    program: Vec<Code>,
    anchor_start: bool,
    anchor_end: bool,
    captures: usize,
}

#[derive(Debug)]
pub struct Match<'a> {
    pub re: &'a Regex,
    pub subj: &'a str,
    pub is_matched: bool,
    pub matches: Vec<&'a str>,
}

#[derive(Debug)]
pub enum Code {
    Char(char),
    Jmp(usize),
    Split { x: usize, y: usize },
    Save(usize),
    Match,
}

pub fn parse(re: &str) -> Regex {
    let mut regex = Regex {
        program: vec![],
        captures: 0,
        anchor_start: false,
        anchor_end: false,
    };
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
            c => regex.program.push(Code::Char(c)),
        }
    }
    regex.program.push(Code::Match);
    regex.captures = k - 1;
    regex
}

pub fn regex_match<'a>(re: &'a Regex, subj: &'a str) -> Vec<Match<'a>> {
    (0..if re.anchor_start { 1 } else { subj.len() })
        .filter_map(|n| {
            let mut saved = [0; 20];
            let subsubj = &subj[n..];
            let is_matched = dbg!(exec(
                &re.program,
                subsubj.chars().collect::<Vec<_>>().as_slice(),
                0,
                0,
                &mut saved,
            ));
            if is_matched {
                Some(Match {
                    re,
                    subj,
                    is_matched,
                    matches: (0..(re.captures + 1))
                        .map(|n| &subsubj[saved[2 * n]..saved[2 * n + 1]])
                        .collect(),
                })
            } else {
                None
            }
        })
        .collect()
}

type ThreadList = std::collections::VecDeque<usize>;

fn exec(
    program: &[Code],
    subj: &[char],
    mut pc: usize,
    mut sp: usize,
    saved: &mut [usize],
) -> bool {
    loop {
        match program[pc] {
            Code::Char(c) => {
                if subj.get(sp) != Some(&c) {
                    return false;
                } else {
                    pc += 1;
                    sp += 1;
                }
            }
            Code::Jmp(x) => pc = x,
            Code::Split { x, y } => {
                if exec(program, subj, x, sp, saved) {
                    return true;
                } else {
                    pc = y;
                }
            }
            Code::Save(slot) => {
                let old = saved[slot];
                saved[slot] = sp;
                if exec(program, subj, pc + 1, sp, saved) {
                    return true;
                } else {
                    saved[slot] = old;
                    return false;
                }
            }
            Code::Match => return true,
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
                    if &data == command {
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
    use super::*;

    #[test]
    fn it_works() {
        let regex = parse("(a+)(b*)");
        let subj = "bab";
        println!("{:?} -> {:?}", regex, regex_match(&regex, subj));
    }
}
