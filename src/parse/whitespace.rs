pub fn skip_whitespace(src: &mut &[u8], skip_newlines: bool) {
    loop {
        let whitespace_end = src
            .iter()
            .position(|&b| !match b {
                b' ' | b'\t' | b'\r' => true,
                b'\n' => skip_newlines,
                _ => false,
            })
            .unwrap_or(src.len());
        *src = &src[whitespace_end..];
        if src.starts_with(b"#") {
            let comment_end = src.iter().position(|&b| b == b'\n').unwrap_or(src.len());
            *src = &src[comment_end..];
        } else {
            return;
        }
    }
}

#[test]
fn test() {
    let mut s = "   \t\n\n\r\nbla\n ".as_bytes();
    skip_whitespace(&mut s, true);
    assert_eq!(s, b"bla\n ");

    let mut s = "   \n  bla\n ".as_bytes();
    skip_whitespace(&mut s, false);
    assert_eq!(s, b"\n  bla\n ");

    let mut s = "   #bla bla bla\n".as_bytes();
    skip_whitespace(&mut s, false);
    assert_eq!(s, b"\n");

    let mut s = "#comment\n#second comment\n  #third\n\n  pizza".as_bytes();
    skip_whitespace(&mut s, true);
    assert_eq!(s, b"pizza");

    let mut s = "  ".as_bytes();
    skip_whitespace(&mut s, true);
    assert_eq!(s, b"");

    let mut s = "a ".as_bytes();
    skip_whitespace(&mut s, true);
    assert_eq!(s, b"a ");

    let mut s = "".as_bytes();
    skip_whitespace(&mut s, true);
    assert_eq!(s, b"");
}
