use super::consume::Consume;
use super::error::{Error, Message, error};
use super::whitespace::skip_whitespace;

pub enum MatchMode<'a> {
	Nothing,
	EndOfFile,
	Specific(&'static str),
	MatchingBracket(&'a str, &'static str),
	ElementEnd, // , or ; or \n
}

pub use self::MatchMode::*;

pub struct Matcher<'a: 'b, 'b> {
	pub mode: MatchMode<'a>,
	pub or_before: Option<&'b Matcher<'a, 'b>>
}

impl<'a, 'b> Matcher<'a, 'b> {

	pub fn new(m: MatchMode<'a>) -> Self {
		Matcher{ mode: m, or_before: None }
	}

	pub fn specific(s: &'static str) -> Self {
		Matcher{ mode: Specific(s), or_before: None }
	}

	pub fn bracket(left: &'a str, right: &'static str) -> Self {
		Matcher{ mode: MatchingBracket(left, right), or_before: None }
	}

	// TODO: Just return a boolean?
	fn try_parse<'c>(
		&self,
		source: &mut &'c str,
		eat_whitespace: bool
	) -> Option<&'c str> {
		if eat_whitespace {
			let skip_newlines = match self.mode {
				ElementEnd => false,
				_          => true,
			};
			skip_whitespace(source, skip_newlines);
		}
		match self.mode {
			Nothing => (),
			EndOfFile => {
				if source.is_empty() {
					return Some(&source[..]);
				}
			},
			Specific(s) | MatchingBracket(_, s) => {
				if let Some(m) = source.consume(s) {
					return Some(m);
				}
			},
			ElementEnd => {
				if let Some(m) = source.consume_if(|x| x == ',' || x == ';' || x == '\n') {
					return Some(m);
				}
			}
		}
		if let Some(b) = self.or_before {
			let mut s: &str = *source;
			if let Some(m) = b.try_parse(&mut s, false) {
				return Some(&m[..0]);
			}
		}
		None
	}

	fn parse<'c: 'a>(
		&self,
		source: &mut &'c str
	) -> Result<&'c str, Error<'a>> {
		self.try_parse(source, true).ok_or_else(|| self.error(source))
	}

	pub fn parse_end(
		&self,
		source: &mut &'a str
	) -> Result<bool, Error<'a>> {
		match self.try_parse(source, true) {
			None if source.is_empty() => Err(self.error(*source)),
			None => Ok(false),
			Some(_) => Ok(true),
		}
	}

	pub fn description(&self) -> String {
		let mut desc = match self.mode {
			Nothing => String::new(),
			EndOfFile => "end of file".to_string(),
			Specific(s) | MatchingBracket(_, s) => format!("`{}'", s),
			ElementEnd => "newline or `,` or `;'".to_string(),
		};
		if let Some(b) = self.or_before {
			if desc.is_empty() {
				desc = b.description();
			} else {
				desc.push_str(" or ");
				desc.push_str(&b.description());
			}
		}
		desc
	}

	pub fn error(&self, source: &'a str) -> Error<'a> {
		Error{
			message: Message{
				message: self.description(),
				location: Some(&source[..0]),
			},
			notes: match (&self.mode, self.or_before) {
				(&MatchingBracket(b, _), None) =>
					vec![Message{
						message: format!("... to match this `{}'", b),
						location: Some(b)
					}],
				_ => vec![],
			},
		}
	}

	pub fn or_before(mut self, or_before: &'b Matcher<'a, 'b>) -> Self {
		assert!(self.or_before.is_none());
		self.or_before = Some(or_before);
		self
	}
}

//TODO: Add tests for Matcher.
