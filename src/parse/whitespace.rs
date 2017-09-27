pub fn skip_whitespace(src: &mut &str, skip_newlines: bool) {
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

#[test]
fn test() {
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
