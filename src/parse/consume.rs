use std::str::from_utf8_unchecked;
use std::ascii::AsciiExt;

pub trait Consume<'a> {
	fn consume_n(&mut self, n: usize) -> &'a [u8];
	unsafe fn consume_str_n(&mut self, n: usize) -> &'a str;
	fn consume(&mut self, what: &str) -> Option<&'a str>;
	fn consume_if<F: FnOnce(char)->bool>(&mut self, f: F) -> Option<&'a str>;
	fn consume_while<F: FnMut(char)->bool>(&mut self, f: F) -> &'a str;
}

impl<'a> Consume<'a> for &'a [u8] {
	fn consume_n(&mut self, n: usize) -> &'a [u8] {
		let (left, right) = self.split_at(n);
		*self = right;
		left
	}
	unsafe fn consume_str_n(&mut self, n: usize) -> &'a str {
		from_utf8_unchecked(self.consume_n(n))
	}
	fn consume(&mut self, what: &str) -> Option<&'a str> {
		if self.starts_with(what.as_bytes()) {
			Some(unsafe { self.consume_str_n(what.len()) })
		} else {
			None
		}
	}
	fn consume_if<F: FnOnce(char)->bool>(&mut self, f: F) -> Option<&'a str> {
		match self.first() {
			Some(&b) if (b as char).is_ascii() && f(b as char) => {
				Some(unsafe { self.consume_str_n(1) })
			},
			_ => None,
		}
	}
	fn consume_while<F: FnMut(char)->bool>(&mut self, mut f: F) -> &'a str {
		let n = self.iter().position(
			|&b| !(b as char).is_ascii() || !f(b as char)
		).unwrap_or(self.len());
		unsafe { self.consume_str_n(n) }
	}
}

#[cfg(test)]
mod test {
	use super::*;

	static HELLO: &'static [u8] = b"hello";

	#[test]
	fn consume_n() {
		let mut m: &'static [u8] = HELLO;
		let c: &'static [u8]  = m.consume_n(3);
		assert_eq!(c, b"hel");
		assert_eq!(m, b"lo");
		assert_eq!(c.as_ptr(), HELLO.as_ptr());
		assert_eq!(m.as_ptr(), HELLO[3..].as_ptr());
	}

	#[test]
	fn consume() {
		let mut m: &'static [u8] = HELLO;
		let c: Option<&'static str>  = m.consume("he");
		assert_eq!(c, Some("he"));
		assert_eq!(c.unwrap().as_ptr(), HELLO.as_ptr());
		assert_eq!(m.consume("xyz"), None);
		assert_eq!(m, b"llo");
		assert_eq!(m.as_ptr(), HELLO[2..].as_ptr());
	}

	#[test]
	fn consume_if() {
		let mut m: &'static [u8] = HELLO;
		let h = m.consume_if(|x| x == 'h');
		let x = m.consume_if(|x| x == 'x');
		assert_eq!(m, b"ello");
		assert_eq!(h, Some("h"));
		assert_eq!(x, None);
		assert_eq!(m.as_ptr(), HELLO[1..].as_ptr());
		assert_eq!(h.unwrap().as_ptr(), HELLO.as_ptr());
	}

	// consume_if now only works on ascii.
	// This test verifies that it just stops at the first non-ascii character.
	#[test]
	fn wide_char() {
		let mut m = "αβ".as_bytes();
		let x = m.consume_if(|x| x == 'α');
		let y = m.consume_if(|x| x == 'ξ');
		assert_eq!(x, None);
		assert_eq!(y, None);
	}

	#[test]
	fn empty() {
		let mut m = b"" as &[u8];
		let p = m.as_ptr();
		assert_eq!(m.consume_if(|_| true), None);
		assert_eq!(m, b"");
		assert_eq!(m.as_ptr(), p);
	}

	#[test]
	fn while_() {
		let mut m = b"----+++" as &[u8];
		assert_eq!(m.consume_while(|x| x == '-'), "----");
		assert_eq!(m, b"+++");
		assert_eq!(m.consume_while(|x| x != '+'), "");
		assert_eq!(m, b"+++");
		assert_eq!(m.consume_while(|x| x == '+'), "+++");
		assert_eq!(m, b"");
	}

	#[test]
	#[should_panic]
	fn panic() {
		let mut m = b"abc" as &[u8];
		m.consume_n(4);
	}
}
