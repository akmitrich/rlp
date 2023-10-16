use parse::{PatternElement, PatternObject};

pub mod parse;

pub fn eval(states: &[PatternElement], subject: &str) -> (bool, usize) {
    let mut i = 0;
    let mut states = states.iter();
    let mut current_state = states.next();
    println!("subject={:?}", subject);
    while let Some(state) = current_state {
        println!("{} -> {:?}", i, current_state);
        match state.quantifier {
            parse::Quantifier::ExactlyOne => {
                let (is_match, consumed) = state_matches_str_at_index(state, subject, i);
                if !is_match {
                    return (false, i);
                }
                i += consumed;
                current_state = states.next();
            }
            parse::Quantifier::ZeroOrOne => {
                if i < subject.len() {
                    let (_, consumed) = state_matches_str_at_index(state, subject, i);
                    i += consumed;
                    current_state = states.next();
                }
            }
            parse::Quantifier::ZeroOrMore => loop {
                if i >= subject.len() {
                    current_state = states.next();
                    break;
                }
                let (is_match, consumed) = state_matches_str_at_index(state, subject, i);
                if !is_match || consumed == 0 {
                    current_state = states.next();
                    break;
                }
                i += consumed;
            },
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
        dbg!(eval(&states, "abc"));
    }

    #[test]
    fn one_group() {
        let states = parse::parse(r"a(b.)*cd");
        assert_eq!((true, 7), eval(&states, "ab!b$cd"));
        assert_eq!((true, 5), eval(&states, "ab!cd"));
        assert_eq!((true, 3), eval(&states, "acd"));
        assert_eq!((false, 2), eval(&states, "ac"));
    }
}
