#[derive(Debug)]
pub struct Message<'a> {
    pub message: String,
    pub location: Option<&'a [u8]>,
}

#[derive(Debug)]
pub struct Error<'a> {
    pub message: Message<'a>,
    pub notes: Vec<Message<'a>>,
}

pub fn error<'a>(location: &'a [u8], message: String) -> Error<'a> {
    Error {
        message: Message {
            message,
            location: Some(location),
        },
        notes: vec![],
    }
}
