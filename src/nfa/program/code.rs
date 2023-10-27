use super::CharacterClass;

#[derive(Debug)]
pub enum Code {
    Char(CharacterClass),
    Captured(usize),
    Jmp(usize),
    Split { x: usize, y: usize },
    Save(usize),
    Match,
}
