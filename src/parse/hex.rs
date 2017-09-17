pub fn digit_value(c: char) -> Result<u8, ()> {
	match c {
		'0'       => Ok( 0),
		'1'       => Ok( 1),
		'2'       => Ok( 2),
		'3'       => Ok( 3),
		'4'       => Ok( 4),
		'5'       => Ok( 5),
		'6'       => Ok( 6),
		'7'       => Ok( 7),
		'8'       => Ok( 8),
		'9'       => Ok( 9),
		'A' | 'a' => Ok(10),
		'B' | 'b' => Ok(11),
		'C' | 'c' => Ok(12),
		'D' | 'd' => Ok(13),
		'E' | 'e' => Ok(14),
		'F' | 'f' => Ok(15),
		_ => Err(())
	}
}
