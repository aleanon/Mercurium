use deps::*;

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable};

/// Stack allocated wrapper around [[[u8]]] that implements `ZeroizeOnDrop`
#[derive(Debug, Clone, ZeroizeOnDrop, Zeroize, PartialEq, Eq)]
pub struct SeedPhrase([[u8; Self::MAX_WORD_LENGTH]; Self::WORD_COUNT]);

impl SeedPhrase {
    ///24 words with max length of 8 plus whitespaces, including a trailing whitespace
    const MAX_PHRASE_LENGTH: usize = 216;
    const MAX_WORD_LENGTH: usize = 8;
    const WORD_COUNT: usize = 24;

    pub fn new() -> Self {
        Self([[b' '; Self::MAX_WORD_LENGTH]; Self::WORD_COUNT])
    }

    pub fn from_str(phrase: &str) -> Self {
        let mut seed_phrase = Self::new();
        phrase
            .split_whitespace()
            .enumerate()
            .take(Self::WORD_COUNT)
            .for_each(|(index, word)| seed_phrase.update_word(index, word));

        seed_phrase
    }

    pub fn nr_of_words(&self) -> usize {
        Self::WORD_COUNT
    }

    /// Checks if the word index is within bounds and copies a maximum of 8 characters into the buffer.
    /// Index starts at 0
    /// If a word is longer then 8 characters, the first 8 characters are copied.
    /// If the index is out of bounds, no action is performed
    pub fn update_word(&mut self, word_index: usize, input: &str) {
        if word_index < Self::WORD_COUNT {
            self.0[word_index] = [b' '; Self::MAX_WORD_LENGTH];
            if input.len() <= Self::MAX_WORD_LENGTH {
                self.0[word_index][..input.len()].copy_from_slice(input.as_bytes());
                self.0[word_index].make_ascii_lowercase();
            } else {
                self.0[word_index].copy_from_slice(input[..Self::MAX_WORD_LENGTH].as_bytes());
                self.0[word_index].make_ascii_lowercase();
            }
        }
    }

    ///Returns a reference to the word at the given index
    pub fn reference_word(&self, index: usize) -> Option<&str> {
        if index >= Self::WORD_COUNT {
            return None;
        };
        let mut trimmed = self.0[index].as_slice();

        while let [rest @ .., last] = trimmed {
            if last.is_ascii_whitespace() {
                trimmed = rest;
            } else {
                break;
            }
        }

        //The SeedPhrase words can only be created from a &str, it is therefore not possible
        //to have a non-utf8 byte slice, so unwrap is called
        let trimmed_str = std::str::from_utf8(trimmed)
            .unwrap_unreachable(debug_info!("Invalid utf8 in byte slice"));

        Some(trimmed_str)
    }

    ///The byte slices are turned into a `Phrase` instead of a `String` because it should implement `ZeroizeOnDrop`
    pub fn phrase(&self) -> Phrase {
        let mut phrase = String::with_capacity(Self::MAX_PHRASE_LENGTH);

        for slice in self.0.iter() {
            let mut trimmed = slice.as_slice();

            while let [rest @ .., last] = trimmed {
                if last.is_ascii_whitespace() {
                    trimmed = rest;
                } else {
                    break;
                }
            }

            let word = std::str::from_utf8(trimmed)
                .unwrap_unreachable(debug_info!("SeedPhrase contained non utf8 byte"));

            phrase.push_str(word);
            phrase.push(' ');
        }

        phrase.pop();

        Phrase(phrase)
    }
}

#[derive(Debug, ZeroizeOnDrop)]
pub struct Phrase(String);

impl Phrase {
    //The Phrase is created with SeedPhrase max length to avoid possible re-allocations
    //as this can interfere with the ZeroizeOnDrop trait
    pub fn new() -> Self {
        Phrase(String::with_capacity(SeedPhrase::MAX_PHRASE_LENGTH))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn push_str(&mut self, str: &str) {
        self.0
            .push_str(&str[..SeedPhrase::MAX_PHRASE_LENGTH - &self.0.len()])
    }
}

impl From<String> for Phrase {
    fn from(mut value: String) -> Self {
        let mut phrase = Self::new();
        phrase.push_str(value.as_str());
        value.zeroize();

        phrase
    }
}
