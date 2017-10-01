#[derive(Debug)]
pub struct Message<'a> {
	pub message: String,
	pub location: Option<&'a str>,
}

#[derive(Debug)]
pub struct Error<'a> {
	pub message: Message<'a>,
	pub notes: Vec<Message<'a>>,
}

pub fn error<'a>(location: &'a str, message: String) -> Error<'a> {
	Error{
		message: Message{
			message: message,
			location: Some(location),
		},
		notes: vec![],
	}
}
