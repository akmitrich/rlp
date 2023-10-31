use std::ops::RangeInclusive;

#[derive(Debug, PartialEq, Eq)]
pub enum CharacterClass {
    Literal(char),
    Any,
    AlphaNumeric(bool),
    Letter(bool),
    ControlChar(bool),
    Digit(bool),
    Printable(bool),
    Lowercase(bool),
    Punctuation(bool),
    WhiteSpace(bool),
    Uppercase(bool),
    Hexadecimal(bool),
    Range(RangeInclusive<char>),
    Set(Vec<CharacterClass>),
    Unset(Vec<CharacterClass>),
}

impl CharacterClass {
    pub fn is_matched(&self, other: &char) -> bool {
        match self {
            CharacterClass::Literal(c) => c == other,
            CharacterClass::Any => true,
            CharacterClass::AlphaNumeric(is_in) => {
                if *is_in {
                    other.is_alphanumeric()
                } else {
                    !other.is_alphanumeric()
                }
            }
            CharacterClass::Letter(is_in) => {
                if *is_in {
                    other.is_alphabetic()
                } else {
                    !other.is_alphabetic()
                }
            }
            CharacterClass::ControlChar(is_in) => {
                if *is_in {
                    other.is_ascii_control()
                } else {
                    !other.is_ascii_control()
                }
            }
            CharacterClass::Digit(is_in) => {
                if *is_in {
                    other.is_numeric()
                } else {
                    !other.is_numeric()
                }
            }
            CharacterClass::Printable(is_in) => {
                if *is_in {
                    other.is_ascii_graphic() && other != &' '
                } else {
                    !other.is_ascii_graphic() || other == &' '
                }
            }
            CharacterClass::Lowercase(is_in) => {
                if *is_in {
                    other.to_lowercase().next() == Some(*other)
                } else {
                    other.to_lowercase().next() != Some(*other)
                }
            }
            CharacterClass::Punctuation(is_in) => {
                if *is_in {
                    other.is_ascii_punctuation()
                } else {
                    !other.is_ascii_punctuation()
                }
            }
            CharacterClass::WhiteSpace(is_in) => {
                if *is_in {
                    other.is_whitespace()
                } else {
                    !other.is_whitespace()
                }
            }
            CharacterClass::Uppercase(is_in) => {
                if *is_in {
                    other.to_uppercase().next() == Some(*other)
                } else {
                    other.to_uppercase().next() != Some(*other)
                }
            }
            CharacterClass::Hexadecimal(is_in) => {
                if *is_in {
                    other.is_ascii_hexdigit()
                } else {
                    !other.is_ascii_hexdigit()
                }
            }
            CharacterClass::Range(r) => r.contains(other),
            CharacterClass::Set(s) => s.iter().any(|x| x.is_matched(other)),
            CharacterClass::Unset(s) => s.iter().all(|x| !x.is_matched(other)),
        }
    }
}
