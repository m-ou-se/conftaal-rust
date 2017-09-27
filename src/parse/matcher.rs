use super::error::{Error, Message};
use super::consume::Consume;

pub enum MatchMode<'a> {
	Nothing,
	EndOfFile,
	Specific(&'a str),
	MatchingBracket(&'a str, &'a str),
	ElementEnd, // , or ; or \n
}

pub use self::MatchMode::*;

pub struct Matcher<'a> {
	pub mode: MatchMode<'a>,
	pub or_before: Option<&'a Matcher<'a>>
}

impl<'a> Matcher<'a> {

	pub fn new(m: MatchMode<'a>) -> Matcher<'a> {
		Matcher{ mode: m, or_before: None }
	}

	pub fn bracket(left: &'a str, right: &'a str) -> Matcher<'a> {
		Matcher{ mode: MatchingBracket(left, right), or_before: None }
	}

	// TODO: Just return a boolean?
	fn try_parse<'b>(
		&self,
		source: &mut &'b str,
		eat_whitespace: bool
	) -> Option<&'b str> {
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

	fn parse<'b: 'a>(
		&self,
		source: &mut &'b str
	) -> Result<&'b str, Error<'a>> {
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
}

fn skip_whitespace(src: &mut &str, skip_newlines: bool) {
	loop {
		*src = src.trim_left_matches(|x|
			x == ' ' ||
			x == '\t' ||
			x == '\r' ||
			(skip_newlines && x == '\n')
		);
		if src.starts_with('#') {
			*src = src.trim_left_matches(|x| x != '\n');
		} else {
			return;
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn whitespace() {
		let mut s = "   \t\n\n\r\nbla\n ";
		skip_whitespace(&mut s, true);
		assert_eq!(s, "bla\n ");

		let mut s = "   \n  bla\n ";
		skip_whitespace(&mut s, false);
		assert_eq!(s, "\n  bla\n ");

		let mut s = "   #bla bla bla\n";
		skip_whitespace(&mut s, false);
		assert_eq!(s, "\n");

		let mut s = "#comment\n#second comment\n  #third\n\n  pizza";
		skip_whitespace(&mut s, true);
		assert_eq!(s, "pizza");

		let mut s = "  ";
		skip_whitespace(&mut s, true);
		assert_eq!(s, "");

		let mut s = "a ";
		skip_whitespace(&mut s, true);
		assert_eq!(s, "a ");

		let mut s = "";
		skip_whitespace(&mut s, true);
		assert_eq!(s, "");
	}

	//TODO: Add tests for Matcher.
}
