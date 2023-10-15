#[derive(Debug, Clone, PartialEq)]
pub enum Quantifier {
    ExactlyOne,
    ZeroOrOne,
    ZeroOrMore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternElement {
    quantifier: Quantifier,
    object: PatternObject,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternObject {
    Wildcard,
    Literal(char),
    Group(Vec<PatternElement>),
}

pub fn parse(re: &str) -> Vec<PatternElement> {
    let mut i = 0;
    let mut stack = vec![vec![]];
    let re = re.chars().collect::<Vec<_>>();
    while i < re.len() {
        let next = dbg!(re[i]);
        match next {
            '.' => {
                stack.last_mut().unwrap().push(PatternElement {
                    quantifier: Quantifier::ExactlyOne,
                    object: PatternObject::Wildcard,
                });
                i += 1;
            }
            '?' => {
                if let Some(last_element) = stack.last_mut().unwrap().last_mut() {
                    if last_element.quantifier != Quantifier::ExactlyOne {
                        panic!("Too many quantifiers in the row");
                    }
                    last_element.quantifier = Quantifier::ZeroOrOne;
                    i += 1;
                }
            }
            '*' => {
                if let Some(last_element) = stack.last_mut().unwrap().last_mut() {
                    if last_element.quantifier != Quantifier::ExactlyOne {
                        panic!("Too many quantifiers in the row");
                    }
                    last_element.quantifier = Quantifier::ZeroOrMore;
                    i += 1;
                }
            }
            '+' => {
                if let Some(last_element) = stack.last_mut().unwrap().last_mut() {
                    if last_element.quantifier != Quantifier::ExactlyOne {
                        panic!("Too many quantifiers in the row");
                    }
                    let mut zero_or_more_copy = last_element.to_owned();
                    zero_or_more_copy.quantifier = Quantifier::ZeroOrMore;
                    stack.last_mut().unwrap().push(zero_or_more_copy);
                    i += 1;
                }
            }
            '(' => {
                stack.push(vec![]);
                i += 1;
            }
            ')' => {
                if stack.len() <= 1 {
                    panic!("Unexpected ')' token.");
                }
                let states = stack.pop().unwrap();
                let group = PatternElement {
                    quantifier: Quantifier::ExactlyOne,
                    object: PatternObject::Group(states),
                };
                stack.last_mut().unwrap().push(group);
                i += 1;
            }
            '\\' => {
                if i + 1 >= re.len() {
                    panic!("Bad escape character.")
                }
                stack.last_mut().unwrap().push(PatternElement {
                    quantifier: Quantifier::ExactlyOne,
                    object: PatternObject::Literal(re[i + 1]),
                });
                i += 2;
            }
            c => {
                stack.last_mut().unwrap().push(PatternElement {
                    quantifier: Quantifier::ExactlyOne,
                    object: PatternObject::Literal(c),
                });
                i += 1;
            }
        }
    }
    if stack.len() != 1 {
        panic!("Unmatched groups in regular expression");
    }
    stack.first().unwrap().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let regex = r"a?(b.*c)+d";
        let result = parse(regex);
        dbg!(&result);
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
