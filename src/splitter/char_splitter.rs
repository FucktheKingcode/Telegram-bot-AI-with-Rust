use crate::Result;

pub struct CharSplitter<'a> {
	text: &'a str,
	num: usize,
	position: usize,
}

impl<'a> CharSplitter<'a> {
	pub fn new(text: &'a str, num: u32) -> Self {
		CharSplitter {
			text,
			num: num as usize,
			position: 0,
		}
	}
}

impl<'a> Iterator for CharSplitter<'a> {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		if self.position >= self.text.len() {
			return None;
		}

		let end = self.text[self.position..]
			.char_indices()
			.nth(self.num)
			.map_or(self.text.len(), |(idx, _)| self.position + idx);

		let chunk = self.text[self.position..end].to_string();
		self.position = end;
		Some(chunk)
	}
}

pub fn simple_char_splitter(text: &str, num: u32) -> Result<CharSplitter<'_>> {
	if num == 0 {
		Err("Number of characters must be greater than 0".into())
	} else {
		Ok(CharSplitter::new(text, num))
	}
}