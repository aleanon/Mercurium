use deps::*;

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{debug_info, unwrap_unreachable::UnwrapUnreachable};

/// Stack allocated wrapper around [[[u8]]] that implements `ZeroizeOnDrop`
#[derive(Debug, Clone, ZeroizeOnDrop, Zeroize, PartialEq, Eq)]
pub struct SeedPhrase([[u8; Self::MAX_WORD_LENGTH]; Self::WORD_COUNT]);

impl Default for SeedPhrase {
    fn default() -> Self {
        Self::new()
    }
}

impl SeedPhrase {
    ///24 words with max length of 8 plus whitespaces, including a trailing whitespace
    const MAX_PHRASE_LENGTH: usize = 216;
    const MAX_WORD_LENGTH: usize = 8;
    const WORD_COUNT: usize = 24;

    pub fn new() -> Self {
        Self([[b' '; Self::MAX_WORD_LENGTH]; Self::WORD_COUNT])
    }

    // Intentionally infallible (unlike std::str::FromStr, which returns Result):
    // any input is normalised into a fixed 24x8 buffer, never rejected.
    #[allow(clippy::should_implement_trait)]
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
            // Copy at most MAX_WORD_LENGTH bytes, cut on a UTF-8 char boundary so
            // a multi-byte char can never cause a mid-codepoint slice panic.
            let take = super::floor_char_boundary(input, Self::MAX_WORD_LENGTH);
            self.0[word_index][..take].copy_from_slice(&input.as_bytes()[..take]);
            self.0[word_index].make_ascii_lowercase();
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

impl Default for Phrase {
    fn default() -> Self {
        Self::new()
    }
}

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
        // saturating_sub guards the underflow when the buffer is already full;
        // floor_char_boundary caps the copy at the remaining capacity on a
        // UTF-8 boundary (and is a no-op when the whole string fits).
        let remaining = SeedPhrase::MAX_PHRASE_LENGTH.saturating_sub(self.0.len());
        let take = super::floor_char_boundary(str, remaining);
        self.0.push_str(&str[..take])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_word_lowercases_and_reference_reads_back() {
        let mut phrase = SeedPhrase::new();
        phrase.update_word(0, "Abandon");
        assert_eq!(phrase.reference_word(0), Some("abandon"));
    }

    #[test]
    fn reference_word_out_of_bounds_is_none() {
        let phrase = SeedPhrase::new();
        assert_eq!(phrase.reference_word(SeedPhrase::WORD_COUNT), None);
        assert_eq!(phrase.reference_word(1000), None);
    }

    #[test]
    fn from_str_splits_words() {
        let phrase = SeedPhrase::from_str("abandon ability able");
        assert_eq!(phrase.reference_word(0), Some("abandon"));
        assert_eq!(phrase.reference_word(1), Some("ability"));
        assert_eq!(phrase.reference_word(2), Some("able"));
    }

    #[test]
    fn long_word_is_truncated_to_max_length() {
        let mut phrase = SeedPhrase::new();
        phrase.update_word(0, "abcdefghij"); // 10 chars > MAX_WORD_LENGTH (8)
        assert_eq!(phrase.reference_word(0), Some("abcdefgh"));
    }

    #[test]
    fn word_count_is_24() {
        assert_eq!(SeedPhrase::new().nr_of_words(), 24);
    }

    #[test]
    fn multibyte_word_does_not_panic_on_truncation() {
        let mut phrase = SeedPhrase::new();
        // "€" is 3 bytes; 4 of them = 12 bytes > MAX_WORD_LENGTH (8). A raw
        // byte slice at 8 would land mid-codepoint and panic.
        phrase.update_word(0, "€€€€");
        let w = phrase.reference_word(0).unwrap();
        assert!(w.len() <= SeedPhrase::MAX_WORD_LENGTH);
        assert!(std::str::from_utf8(w.as_bytes()).is_ok());
    }

    #[test]
    fn phrase_push_str_past_capacity_does_not_panic() {
        let mut phrase = Phrase::new();
        let huge = "é".repeat(SeedPhrase::MAX_PHRASE_LENGTH); // 2 bytes each
        phrase.push_str(&huge);
        // Second push with a full buffer must not underflow.
        phrase.push_str("more");
        assert!(phrase.as_str().len() <= SeedPhrase::MAX_PHRASE_LENGTH);
    }
}
