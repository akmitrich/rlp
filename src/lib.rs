use parse::{PatternElement, PatternObject};

pub mod nfa;
pub mod parse;

pub struct BackTrack {
    is_backtrackable: bool,
    state: PatternElement,
    consumptions: Vec<usize>,
}

pub fn eval(states: &[PatternElement], subject: &str) -> (bool, usize) {
    fn backtrack(
        backtrack_stack: &mut Vec<BackTrack>,
        st_index: &mut usize,
        subj_index: &mut usize,
    ) -> bool {
        *st_index -= 1;
        let mut could_backtrack = false;
        while let Some(BackTrack {
            is_backtrackable,
            state,
            mut consumptions,
        }) = backtrack_stack.pop()
        {
            if is_backtrackable {
                if consumptions.is_empty() {
                    *st_index -= 1;
                    continue;
                }
                let n = consumptions.pop().unwrap();
                *subj_index -= n;
                backtrack_stack.push(BackTrack {
                    is_backtrackable,
                    state,
                    consumptions,
                });
                could_backtrack = true;
                break;
            }
            *st_index -= 1;
            *subj_index -= consumptions.iter().sum::<usize>();
        }
        if could_backtrack {
            *st_index += 1;
        }
        could_backtrack
    }

    let mut i = 0;
    let mut j = 0;
    // let mut states = states.iter();
    // let mut current_state = states.get(j);
    let mut backtrack_stack = vec![];
    println!("subject={:?}", subject);
    while let Some(state) = states.get(j) {
        println!("{} -> {:?}", i, state);
        match state.quantifier {
            parse::Quantifier::ExactlyOne => {
                let (is_match, consumed) = state_matches_str_at_index(state, subject, i);
                if !is_match {
                    let index_before_backtracking = i;
                    let could_backtrack = backtrack(&mut backtrack_stack, &mut j, &mut i);
                    if !could_backtrack {
                        return (false, index_before_backtracking);
                    }
                    continue;
                }
                backtrack_stack.push(BackTrack {
                    is_backtrackable: false,
                    state: state.to_owned(),
                    consumptions: vec![consumed],
                });
                i += consumed;
                j += 1;
            }
            parse::Quantifier::ZeroOrOne => {
                if i < subject.len() {
                    let (is_match, consumed) = state_matches_str_at_index(state, subject, i);
                    backtrack_stack.push(BackTrack {
                        is_backtrackable: is_match && consumed > 0,
                        state: state.to_owned(),
                        consumptions: vec![consumed],
                    });
                    i += consumed;
                } else {
                    backtrack_stack.push(BackTrack {
                        is_backtrackable: false,
                        state: state.to_owned(),
                        consumptions: vec![0],
                    });
                }
                j += 1;
            }
            parse::Quantifier::ZeroOrMore => {
                let mut backtrack_state = BackTrack {
                    is_backtrackable: true,
                    state: state.to_owned(),
                    consumptions: vec![],
                };
                loop {
                    if i >= subject.len() {
                        if backtrack_state.consumptions.is_empty() {
                            backtrack_state.is_backtrackable = false;
                            backtrack_state.consumptions.push(0);
                        }
                        backtrack_stack.push(backtrack_state);
                        j += 1;
                        break;
                    }
                    let (is_match, consumed) = state_matches_str_at_index(state, subject, i);
                    if !is_match || consumed == 0 {
                        if backtrack_state.consumptions.is_empty() {
                            backtrack_state.is_backtrackable = false;
                            backtrack_state.consumptions.push(0);
                        }
                        backtrack_stack.push(backtrack_state);
                        j += 1;
                        break;
                    }
                    backtrack_state.consumptions.push(consumed);
                    i += consumed;
                }
            }
        }
    }
    (true, i)
}

fn state_matches_str_at_index(
    state: &PatternElement,
    subject: &str,
    index: usize,
) -> (bool, usize) {
    if index >= subject.len() {
        return (false, 0);
    }
    match &state.object {
        PatternObject::Wildcard => (true, 1),
        PatternObject::Literal(c) => {
            let is_match = Some(c) == subject.chars().nth(index).as_ref();
            (is_match, if is_match { 1 } else { 0 })
        }
        PatternObject::Group(group) => eval(group, &subject[index..]),
    }
}

#[cfg(test)]
mod tests {
    use crate::parse::parse;

    use super::*;

    #[test]
    fn it_works() {
        let states = parse::parse(r"a?bc");
        dbg!(eval(&states, "bc"));
    }

    #[test]
    fn one_group() {
        let states = parse::parse(r"a(b.)*cd");
        assert_eq!((true, 7), eval(&states, "ab!b$cd"));
        assert_eq!((true, 5), eval(&states, "ab!cd"));
        assert_eq!((true, 3), eval(&states, "acd"));
        assert_eq!((false, 2), eval(&states, "ac"));
    }

    #[test]
    fn try_backtrack() {
        let states = parse::parse(r"a?(b.*c)+d");
        dbg!(eval(&states, "abcd"));
    }
}
