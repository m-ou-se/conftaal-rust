use std::fmt::Write;

use super::consume::Consume;
use super::error::{Error, Message, error};
use super::whitespace::skip_whitespace;

/// Determines until what point should be parsed.
/// An value of this type is given to the `parse_*` functions.
#[derive(Clone, Copy)]
pub enum End<'a> {

	/// Don't stop till the end of the file is earched.
	EndOfFile,

	/// Only stop when this specific string is found.
	Specific(&'static str),

	/// Only stop when these brackets are matched.
	///
	/// `MatchingBracket("(", ")")` looks for a `")"` to match the `"("`.
	MatchingBracket(&'a str, &'static str),

	/// Stop at either a comma, semicolon, or newline.
	///
	/// Values in conftaal objects use this mode.
	ElementEnd, // , or ; or \n
}

impl<'a> End<'a> {

	fn consume(&self, source: &mut &[u8]) -> bool {
		match self {
			&End::EndOfFile => source.is_empty(),
			&End::Specific(s) | &End::MatchingBracket(_, s) => source.consume(s).is_some(),
			&End::ElementEnd => source.consume_one_of(",;\n").is_some(),
		}
	}

	fn matches(&self, mut source: &[u8]) -> bool {
		self.consume(&mut source)
	}

	/// An human-readable description of what this `End` matches.
	/// Useful for in error messages.
	pub fn description(&self) -> String {
		match self {
			&End::EndOfFile => "end of file".to_string(),
			&End::Specific(s) | &End::MatchingBracket(_, s) => format!("`{}'", s),
			&End::ElementEnd => "newline or `,` or `;'".to_string(),
		}
	}

	fn error(&self, source: &'a [u8]) -> Error<'a> {
		let mut e = error(&source[..0], format!("expected {}", self.description()));
		if let &End::MatchingBracket(b, _) = self {
			e.notes = vec![Message{
				message: format!("... to match this `{}'", b),
				location: Some(b.as_bytes())
			}];
		}
		e
	}

	/// Check if we reached this `End` yet.
	///
	/// First eats whitespace, and then tries to match.
	///
	/// If the end of the file is reached, and this is not a `End::EndOfFile`,
	/// an error is given.
	pub fn parse(&self, source: &mut &'a [u8]) -> Result<bool, Error<'a>> {
		skip_whitespace(source, match self { &End::ElementEnd => false, _ => true });
		let did_match = self.consume(source);
		if !did_match && source.is_empty() {
			Err(self.error(*source))
		} else {
			Ok(did_match)
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

	pub fn parse(&self, source: &mut &'a [u8]) -> Result<bool, Error<'a>> {
		skip_whitespace(source, match self.end { End::ElementEnd => false, _ => true });
		let did_match = self.end.consume(source) || self.or_before.map(|e| e.matches(*source)).unwrap_or(false);
		if !did_match && source.is_empty() {
			Err(self.error(*source))
		} else {
			Ok(did_match)
		}
	}

	pub fn description(&self) -> String {
		let mut desc = self.end.description();
		if let Some(e) = self.or_before {
			write!(&mut desc, " or {}", e.description()).unwrap();
		}
		desc
	}

	fn error(&self, source: &'a [u8]) -> Error<'a> {
		match self.or_before {
			None => self.end.error(source),
			Some(_) => error(&source[..0], format!("expected {}", self.description())),
		}
	}
}

//TODO: Add tests.
