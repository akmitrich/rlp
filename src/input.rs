#[derive(Debug, PartialEq)]
pub(crate) struct Input<'a> {
    pub subj: &'a str,
    chars: Box<[(usize, char)]>,
}

impl<'a> Input<'a> {
    pub fn new(subj: &'a str) -> Self {
        Self {
            subj,
            chars: subj.char_indices().collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn get_char(&self, index: usize) -> Option<char> {
        self.chars.get(index).map(|(_, c)| *c)
    }

    pub fn get_byte_index(&self, char_index: usize) -> Option<usize> {
        if char_index == self.len() {
            self.chars.last().map(|(i, c)| *i + c.len_utf8())
        } else {
            self.chars.get(char_index).map(|(i, _)| *i)
        }
    }
}
