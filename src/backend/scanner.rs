use std::char;

use crate::backend::token;

use super::token::Token;
/// Copied from https://depth-first.com/articles/2021/12/16/a-beginners-guide-to-parsing-in-rust/.
/// Will have to rework at some point.


pub struct Scanner {
    cursor: usize,
    characters: Vec<char>,
}

pub enum ScannerError {
    Character(usize),
    EndOfLine,
}

pub enum Action<T> {
    /// If next iteration returns None, return T without advancing
    /// the cursor.
    Request(T),

    /// If the next iteration returns None, return None without advancing
    /// the cursor.
    Require,

    /// Immediately advance the cursor and return T.
    Return(T)
}

impl Scanner {
    fn new (string: &str) -> Self {
        Self {
            cursor: 0,
            characters: string.chars().collect(),
        }
    }

    fn peek(&self) -> Option<&char> {
        self.characters.get(self.cursor)
    }

    fn prev(&self) -> Option<&char> {
        self.characters.get(self.cursor-1)
    }

    fn is_done(&self) -> bool {
        self.cursor == self.characters.len()
    }

    fn pop(&mut self) -> Option<&char> {
        match self.characters.get(self.cursor) {
            Some(character) => {
                self.cursor += 1;

                Some(character)
            }
            None => None,
        }
    }

    /// Returns true if the `target` is found at the current cursor position,
    /// and advances the cursor.
    /// Otherwise, returns false leaving the cursor unchanged.
    pub fn take(&mut self, target: &char) -> bool {
        match self.characters.get(self.cursor) {
            Some(character) => {
                if target == character {
                    self.cursor += 1;

                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }
    
    pub fn scan<T>(&mut self) -> Result<Option<T>, ScannerError> {
        let mut sequence = String::new();

        loop{
            match self.peek() {
                Some(char) => {
                    sequence.push(*char);

                    match cb(&sequence) {

                    }
                }
                None => todo!(),
            }
        }
        
        todo!()
    }

}