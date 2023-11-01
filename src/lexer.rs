use itertools::Itertools;

use crate::bytecode::character_class::CharacterClass;

#[derive(Debug, PartialEq)]
pub enum Lex {
    AnyChar,
    Literal(char),
    CharacterClass(CharacterClass),
    CharacterSet(CharacterClass),
    Captured(usize),
    Border(char, char),
    Frontier(CharacterClass),
    SaveOpen(usize),
    SaveClose(usize),
}

#[derive(Debug, PartialEq)]
pub enum Quantifier {
    ExactlyOne,
    ZeroOrOne,
    OneOrMany,
    ZeroOrManyGreedy,
    ZeroOrManyUngreedy,
}

pub fn lex(re: &str) -> impl Iterator<Item = (Lex, Quantifier)> + '_ {
    let mut saves = vec![];
    let mut captures = 0;
    let re = re.chars().peekable();
    re.batching(move |re| {
        let lex = match re.next()? {
            '.' => Lex::AnyChar,
            '[' => Lex::CharacterSet(make_character_set(re)),
            '(' => {
                captures += 1;
                if captures > 9 {
                    panic!("Too many captures.")
                }
                saves.push(captures);
                Lex::SaveOpen(captures)
            }
            ')' => {
                let captured = saves.pop().unwrap();
                Lex::SaveClose(captured)
            }
            '%' => match re.next()? {
                'n' => match re.next()? {
                    d @ ('1'..='9') => Lex::Captured(d.to_digit(10).unwrap() as _),
                    _ => panic!("Inappropriate code %n"),
                },
                'b' => match (re.next()?, re.next()?) {
                    (x, y) if x != y => Lex::Border(x, y),
                    _ => panic!("Border chars must be different"),
                },
                'f' => {
                    if let Some('[') = re.next() {
                        Lex::Frontier(make_character_set(re))
                    } else {
                        panic!("After %f must be '['")
                    }
                }
                c => Lex::CharacterClass(char_to_class(c)),
            },
            c => Lex::Literal(c),
        };
        let quantifier = match re.peek() {
            Some(c) if ['*', '+', '-', '?'].contains(c) => match re.next().unwrap() {
                '*' => Quantifier::ZeroOrManyGreedy,
                '+' => Quantifier::OneOrMany,
                '-' => Quantifier::ZeroOrManyUngreedy,
                '?' => Quantifier::ZeroOrOne,
                _ => unreachable!(),
            },
            _ => Quantifier::ExactlyOne,
        };
        Some((lex, quantifier))
    })
}

fn make_character_set<I>(re: &mut std::iter::Peekable<I>) -> CharacterClass
where
    I: Iterator<Item = char>,
{
    match re.peek() {
        Some(c) if c == &'^' => {
            re.next();
            CharacterClass::Unset(to_character_set(re))
        }
        _ => CharacterClass::Set(to_character_set(re)),
    }
}

fn to_character_set<I>(re: &mut std::iter::Peekable<I>) -> Box<[CharacterClass]>
where
    I: Iterator<Item = char>,
{
    re.batching(|re| {
        Some(match re.next()? {
            '%' => char_to_class(re.next()?),
            ']' => return None,
            c => {
                if let Some('-') = re.peek() {
                    re.next().unwrap();
                    let end = re.next()?;
                    CharacterClass::Range(c..=end)
                } else {
                    CharacterClass::Literal(c)
                }
            }
        })
    })
    .collect()
}

fn char_to_class(c: char) -> CharacterClass {
    let is_in = c.is_ascii_lowercase();
    match c {
        'w' | 'W' => CharacterClass::AlphaNumeric(is_in),
        'a' | 'A' => CharacterClass::Letter(is_in),
        'c' | 'C' => CharacterClass::ControlChar(is_in),
        'd' | 'D' => CharacterClass::Digit(is_in),
        'g' | 'G' => CharacterClass::Printable(is_in),
        'l' | 'L' => CharacterClass::Lowercase(is_in),
        'p' | 'P' => CharacterClass::Punctuation(is_in),
        's' | 'S' => CharacterClass::WhiteSpace(is_in),
        'u' | 'U' => CharacterClass::Uppercase(is_in),
        'x' | 'X' => CharacterClass::Hexadecimal(is_in),
        c if !c.is_alphanumeric() => CharacterClass::Literal(c),
        _ => panic!("Illegal char in escaping."),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn run(re: &str, answer: &[(Lex, Quantifier)], comment: &str) {
        assert_eq!(
            answer,
            lex(re).collect::<Vec<_>>().as_slice(),
            "FAIL: {}",
            comment
        )
    }

    #[test]
    fn it_works() {
        let abcd = lex("abcd").collect::<Vec<_>>();
        assert_eq!(
            &[
                (Lex::Literal('a'), Quantifier::ExactlyOne),
                (Lex::Literal('b'), Quantifier::ExactlyOne),
                (Lex::Literal('c'), Quantifier::ExactlyOne),
                (Lex::Literal('d'), Quantifier::ExactlyOne)
            ],
            abcd.as_slice()
        )
    }

    #[test]
    fn character_classes() {
        let cases = [
            (
                r"%a",
                [(
                    Lex::CharacterClass(CharacterClass::Letter(true)),
                    Quantifier::ExactlyOne,
                )],
            ),
            (
                r"%d+",
                [(
                    Lex::CharacterClass(CharacterClass::Digit(true)),
                    Quantifier::OneOrMany,
                )],
            ),
            (
                r"%D?",
                [(
                    Lex::CharacterClass(CharacterClass::Digit(false)),
                    Quantifier::ZeroOrOne,
                )],
            ),
            (r"%n8", [(Lex::Captured(8), Quantifier::ExactlyOne)]),
        ];
        for (re, answer) in cases {
            run(re, &answer, "One lex.");
        }
    }

    #[test]
    fn two_lexes() {
        let cases = [
            (
                r"%a%l",
                [
                    (
                        Lex::CharacterClass(CharacterClass::Letter(true)),
                        Quantifier::ExactlyOne,
                    ),
                    (
                        Lex::CharacterClass(CharacterClass::Lowercase(true)),
                        Quantifier::ExactlyOne,
                    ),
                ],
            ),
            (
                r"%d+%D*",
                [
                    (
                        Lex::CharacterClass(CharacterClass::Digit(true)),
                        Quantifier::OneOrMany,
                    ),
                    (
                        Lex::CharacterClass(CharacterClass::Digit(false)),
                        Quantifier::ZeroOrManyGreedy,
                    ),
                ],
            ),
            (
                r"%D?%w-",
                [
                    (
                        Lex::CharacterClass(CharacterClass::Digit(false)),
                        Quantifier::ZeroOrOne,
                    ),
                    (
                        Lex::CharacterClass(CharacterClass::AlphaNumeric(true)),
                        Quantifier::ZeroOrManyUngreedy,
                    ),
                ],
            ),
            (
                r"[%D?%w%-]-%U",
                [
                    (
                        Lex::CharacterSet(CharacterClass::Set(
                            [
                                CharacterClass::Digit(false),
                                CharacterClass::Literal('?'),
                                CharacterClass::AlphaNumeric(true),
                                CharacterClass::Literal('-'),
                            ]
                            .into(),
                        )),
                        Quantifier::ZeroOrManyUngreedy,
                    ),
                    (
                        Lex::CharacterClass(CharacterClass::Uppercase(false)),
                        Quantifier::ExactlyOne,
                    ),
                ],
            ),
            (
                r"%a-%b78",
                [
                    (
                        Lex::CharacterClass(CharacterClass::Letter(true)),
                        Quantifier::ZeroOrManyUngreedy,
                    ),
                    (Lex::Border('7', '8'), Quantifier::ExactlyOne),
                ],
            ),
        ];
        for (re, answer) in cases {
            run(re, &answer, "Two lexes");
        }
    }
}
