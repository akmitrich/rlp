#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    ExactlyOne,
    ZeroOrOne,
    ZeroOrMore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternElement {
    pub quantifier: Quantifier,
    pub object: PatternObject,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternObject {
    Wildcard,
    Literal(char),
    Group(Vec<PatternElement>),
}

pub fn parse(re: &str) -> Vec<PatternElement> {
    let mut stack = vec![vec![]];
    let mut re = re.chars();
    loop {
        let next = if let Some(c) = re.next() {
            c
        } else {
            break;
        };
        match next {
            '.' => {
                stack.last_mut().unwrap().push(PatternElement::new(
                    Quantifier::ExactlyOne,
                    PatternObject::Wildcard,
                ));
            }
            '?' => make_sure_exactly_one_and_update(&mut stack, Quantifier::ZeroOrOne).unwrap(),
            '*' => make_sure_exactly_one_and_update(&mut stack, Quantifier::ZeroOrMore).unwrap(),
            '+' => make_one_or_more(&mut stack).unwrap(),
            '(' => stack.push(vec![]),
            ')' => make_group(&mut stack).unwrap(),
            '\\' => {
                if let Some(c) = re.next() {
                    push_literal(stack.last_mut().unwrap(), c);
                } else {
                    panic!("Bad escape character.")
                }
            }
            c => push_literal(stack.last_mut().unwrap(), c),
        }
    }
    if stack.len() != 1 {
        panic!("Unmatched groups in regular expression");
    }
    stack.first().unwrap().to_owned()
}

fn make_sure_exactly_one_and_update(
    stack: &mut [Vec<PatternElement>],
    quantifier: Quantifier,
) -> Result<(), String> {
    if let Some(last_element) = stack.last_mut().unwrap().last_mut() {
        if last_element.quantifier != Quantifier::ExactlyOne {
            return Err("Too many quantifiers in the row".to_owned());
        }
        last_element.quantifier = quantifier;
        Ok(())
    } else {
        Err("No element for quantifier".to_owned())
    }
}

fn make_one_or_more(stack: &mut [Vec<PatternElement>]) -> Result<(), String> {
    let states = stack.last_mut().unwrap();
    if let Some(last_element) = states.last_mut() {
        if last_element.quantifier != Quantifier::ExactlyOne {
            return Err("Too many quantifiers in the row".to_owned());
        }
        let mut zero_or_more_copy = last_element.to_owned();
        zero_or_more_copy.quantifier = Quantifier::ZeroOrMore;
        states.push(zero_or_more_copy);
        Ok(())
    } else {
        Err("No element fo quantifier '+'".to_owned())
    }
}

fn make_group(stack: &mut Vec<Vec<PatternElement>>) -> Result<(), String> {
    if stack.len() <= 1 {
        Err("Unexpected ')' token.".to_owned())
    } else {
        let states = stack.pop().unwrap();
        let group = PatternElement::new(Quantifier::ExactlyOne, PatternObject::Group(states));
        stack.last_mut().unwrap().push(group);
        Ok(())
    }
}

fn push_literal(states: &mut Vec<PatternElement>, c: char) {
    states.push(PatternElement::new(
        Quantifier::ExactlyOne,
        PatternObject::Literal(c),
    ));
}

impl PatternElement {
    pub fn new(quantifier: Quantifier, object: PatternObject) -> Self {
        Self { quantifier, object }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let regex = r"a?(b.*c)+d";
        let result = parse(regex);
        assert_eq!(4, result.len());
        assert_eq!(
            &PatternElement {
                quantifier: Quantifier::ZeroOrOne,
                object: PatternObject::Literal('a')
            },
            result.first().unwrap()
        );
        assert_eq!(
            &PatternElement {
                quantifier: Quantifier::ExactlyOne,
                object: PatternObject::Literal('d')
            },
            result.last().unwrap()
        );
    }
}
