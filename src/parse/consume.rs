pub trait Consume<'a> {
	fn consume_n(&mut self, n: usize) -> &'a str;
	fn consume(&mut self, what: &str) -> Option<&'a str>;
	fn consume_if<F: FnOnce(char)->bool>(&mut self, f: F) -> Option<&'a str>;
	fn consume_while<F: FnMut(char)->bool>(&mut self, f: F) -> &'a str;
}

impl<'a> Consume<'a> for &'a str {
	fn consume_n(&mut self, n: usize) -> &'a str {
		let (left, right) = self.split_at(n);
		*self = right;
		left
	}
	fn consume(&mut self, what: &str) -> Option<&'a str> {
		if self.starts_with(what) {
			Some(self.consume_n(what.len()))
		} else {
			None
		}
	}
	fn consume_if<F: FnOnce(char)->bool>(&mut self, f: F) -> Option<&'a str> {
		let mut c = self.char_indices();
		match c.next() {
			Some((_, ch)) if f(ch) => {
				let n = match c.next() {
					Some((index, _)) => index,
					None => self.len(),
				};
				Some(self.consume_n(n))
			},
			_ => None,
		}
	}
	fn consume_while<F: FnMut(char)->bool>(&mut self, mut f: F) -> &'a str {
		let mut c = self.char_indices();
		while let Some((i, ch)) = c.next() {
			if !f(ch) {
				return self.consume_n(i);
			}
		}
		self.consume_n(self.len())
	}
}

#[cfg(test)]
mod test {
	use super::*;

	static hello: &'static str = "hello";

	#[test]
	fn consume_n() {
		let mut m: &'static str = hello;
		let c: &'static str  = m.consume_n(3);
		assert_eq!(c, "hel");
		assert_eq!(m, "lo");
		assert_eq!(c.as_ptr(), hello.as_ptr());
		assert_eq!(m.as_ptr(), hello[3..].as_ptr());
	}

	#[test]
	fn consume() {
		let mut m: &'static str = hello;
		let c: Option<&'static str>  = m.consume("he");
		assert_eq!(c, Some("he"));
		assert_eq!(c.unwrap().as_ptr(), hello.as_ptr());
		assert_eq!(m.consume("xyz"), None);
		assert_eq!(m, "llo");
		assert_eq!(m.as_ptr(), hello[2..].as_ptr());
	}

	#[test]
	fn consume_if() {
		let mut m: &'static str = hello;
		let h = m.consume_if(|x| x == 'h');
		let x = m.consume_if(|x| x == 'x');
		assert_eq!(m, "ello");
		assert_eq!(h, Some("h"));
		assert_eq!(x, None);
		assert_eq!(m.as_ptr(), hello[1..].as_ptr());
		assert_eq!(h.unwrap().as_ptr(), hello.as_ptr());
	}

	#[test]
	fn wide_char() {
		let mut m = "αβ";
		let x = m.consume_if(|x| x == 'α');
		let y = m.consume_if(|x| x == 'ξ');
		assert_eq!(m, "β");
		assert_eq!(x, Some("α"));
		assert_eq!(y, None);
	}

	#[test]
	fn empty() {
		let mut m = "";
		let p = m.as_ptr();
		assert_eq!(m.consume_if(|_| true), None);
		assert_eq!(m, "");
		assert_eq!(m.as_ptr(), p);
	}

	#[test]
	fn while_() {
		let mut m = "----+++";
		m.consume_while(|x| x == '-');
		assert_eq!(m, "+++");
		m.consume_while(|x| x != '+');
		assert_eq!(m, "+++");
		m.consume_while(|x| x == '+');
		assert_eq!(m, "");
	}

	#[test]
	#[should_panic]
	fn panic() {
		let mut m = "abc";
		m.consume_n(4);
	}
}
