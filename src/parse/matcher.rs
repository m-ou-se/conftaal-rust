use std::fmt::Write;

use super::consume::Consume;
use super::error::{Error, Message, error};
use super::whitespace::skip_whitespace;

#[derive(Clone, Copy)]
pub enum End<'a> {
	EndOfFile,
	Specific(&'static str),
	MatchingBracket(&'a str, &'static str),
	ElementEnd, // , or ; or \n
}

impl<'a> End<'a> {

	fn consume(&self, source: &mut &str) -> bool {
		match self {
			&End::EndOfFile => source.is_empty(),
			&End::Specific(s) | &End::MatchingBracket(_, s) => source.consume(s).is_some(),
			&End::ElementEnd => source.consume_if(|x| x == ',' || x == ';' || x == '\n').is_some(),
		}
	}

	fn matches(&self, mut source: &str) -> bool {
		self.consume(&mut source)
	}

	pub fn description(&self) -> String {
		match self {
			&End::EndOfFile => "end of file".to_string(),
			&End::Specific(s) | &End::MatchingBracket(_, s) => format!("`{}'", s),
			&End::ElementEnd => "newline or `,` or `;'".to_string(),
		}
	}

	fn error(&self, source: &'a str) -> Error<'a> {
		Error{
			message: Message{
				message: format!("expected {}", self.description()),
				location: Some(&source[..0]),
			},
			notes: match self {
				&End::MatchingBracket(b, _) => vec![Message{
					message: format!("... to match this `{}'", b),
					location: Some(b)
				}],
				_ => vec![],
			},
		}
	}

	pub fn parse(&self, source: &mut &'a str) -> Result<bool, Error<'a>> {
		skip_whitespace(source, match self { &End::ElementEnd => false, _ => true });
		// TODO: rewrite
		let matches = self.consume(source);
		if !matches && source.is_empty() {
			Err(self.error(*source))
		} else {
			Ok(matches)
		}
	}

	pub fn or_before(self, or_before: Self) -> OptionalEnd<'a> {
		OptionalEnd{ end: self, or_before: Some(or_before) }
	}

	pub fn as_optional(self) -> OptionalEnd<'a> {
		OptionalEnd{ end: self, or_before: None }
	}
}

pub struct OptionalEnd<'a> {
	pub end: End<'a>,
	pub or_before: Option<End<'a>>,
}

impl<'a> OptionalEnd<'a> {

	pub fn parse(&self, source: &mut &'a str) -> Result<bool, Error<'a>> {
		skip_whitespace(source, match self.end { End::ElementEnd => false, _ => true });
		let matches = self.end.consume(source) || self.or_before.map(|e| e.matches(*source)).unwrap_or(false);
		if !matches && source.is_empty() {
			Err(self.error(*source))
		} else {
			Ok(matches)
		}
	}

	pub fn description(&self) -> String {
		let mut desc = self.end.description();
		if let Some(e) = self.or_before {
			write!(&mut desc, " or {}", e.description()).unwrap();
		}
		desc
	}

	fn error(&self, source: &'a str) -> Error<'a> {
		match self.or_before {
			None => self.end.error(source),
			Some(_) => error(&source[..0], format!("expected {}", self.description())),
		}
	}
}

//TODO: Add tests.
