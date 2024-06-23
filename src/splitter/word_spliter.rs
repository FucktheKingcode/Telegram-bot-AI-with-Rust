use crate::Result;
use std::str::SplitWhitespace;

pub struct WordSplitter<'a> {
	text: &'a str,
	num_of_words: usize,
	position: usize,
}

impl<'a> WordSplitter<'a> {
	pub fn new(text: &'a str, num_of_words: u32) -> Self {
		WordSplitter {
			text,
			num_of_words: num_of_words as usize,
			position: 0,
		}
	}
}

impl<'a> Iterator for WordSplitter<'a> {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		if self.position >= self.text.len() {
			return None;
		}

		let mut word_count = 0;
		let mut end_position = self.position;
		let mut last_space_position = self.position;

		for (idx, ch) in self.text[self.position..].char_indices() {
			if ch.is_whitespace() {
				word_count += 1;
				last_space_position = self.position + idx;
				if word_count == self.num_of_words {
					end_position = self.position + idx;
					break;
				}
			}
		}

		// If we didn't reach the number of words, set end_position to the end of the text
		if word_count < self.num_of_words {
			end_position = self.text.len();
		} else {
			// Include the last word
			end_position = last_space_position;
		}

		let chunk = &self.text[self.position..end_position];
		self.position = end_position + 1; // Move position past the space
		Some(chunk.to_string())
	}
}

pub fn simple_word_splitter(
	text: &str,
	num_of_words: u32,
) -> Result<WordSplitter<'_>> {
	if num_of_words == 0 {
		Err("Number of words must be greater than 0".into())
	} else {
		Ok(WordSplitter::new(text, num_of_words))
	}
}