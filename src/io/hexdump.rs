use crate::io::keyboard;
use crate::{print, println};

pub fn print(address: *const u8, size: i32) {
	const BYTES_PER_LINE: usize = 16;
	if size <= 0 {
		println!("Invalid size: {}", size);
		return;
	}

	let size = size as usize;
	let mut line_count = 0;

	for i in (0..size).step_by(BYTES_PER_LINE) {
		if line_count == 24 {
			print!("Press Enter to continue or press x to quit ...");
			loop {
				let input = keyboard::read();
				match input {
					Some('\n') => break,
					Some('x') => {
						println!("");
						return;
					}
					_ => continue,
				}
			}
			line_count = 0;
			println!("");
		}
		let mut is_all_zero = true;

		// 16바이트 검사
		for j in 0..BYTES_PER_LINE {
			if i + j < size {
				unsafe {
					let byte = *address.add(i + j);
					if byte != 0 {
						is_all_zero = false;
						break;
					}
				}
			}
		}

		if is_all_zero {
			continue;
		}

		print!("{:#08x}:  ", (address as usize) + i);

		for j in 0..BYTES_PER_LINE {
			if i + j < size {
				unsafe {
					let byte = *address.add(i + j);
					print!("{:02x} ", byte);
				}
			} else {
				print!("   ");
			}
		}

		print!(" |");
		for j in 0..BYTES_PER_LINE {
			if i + j < size {
				unsafe {
					let byte = *address.add(i + j);
					let ascii = if byte.is_ascii_graphic() || byte == b' ' {
						byte as char
					} else {
						'.'
					};
					print!("{}", ascii);
				}
			}
		}
		println!("|");
		line_count += 1;
	}
}
