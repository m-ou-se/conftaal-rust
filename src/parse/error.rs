pub struct Message<'a> {
	pub message: String,
	pub location: Option<&'a str>,
}

pub struct Error<'a> {
	pub message: Message<'a>,
	pub notes: Vec<Message<'a>>,
}
