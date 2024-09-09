pub fn substring_between(s: &str, start_char: char, end_char: char) -> Option<&str> {
	let start_pos = s.find(start_char)?;
	let end_pos = s[start_pos + 1..].find(end_char)? + start_pos + 1;
	Some(&s[start_pos + 1..end_pos])
}

pub fn atoi(s: &str) -> Result<usize, &'static str> {
	let mut result: usize = 0;
	let chars = s.chars();

	if chars.clone().next() == Some('-') {
		return Err("Negative numbers are not supported for usize");
	}
	for ch in chars {
		if !ch.is_ascii_digit() {
			return Err("Invalid character");
		}
		let digit = (ch as u8 - b'0') as usize;
		result = result * 10 + digit;
	}
	Ok(result)
}

pub fn append(s: &str, buffer: *mut [u8; 14000], len: usize) {
    let s_bytes = s.as_bytes();
    
    if s_bytes.len() > buffer.len() {
        return ;
    }
    buffer[len..s_bytes.len()].copy_from_slice(s_bytes);
}
