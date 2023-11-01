use super::character_class::CharacterClass;

#[derive(Debug)]
pub enum Code {
    Char(CharacterClass),
    Captured(usize),
    Border(char, char),
    Frontier(CharacterClass),
    Jmp(usize),
    Split { x: usize, y: usize },
    Save(usize),
    Match,
}
