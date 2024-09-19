use crate::{print, println};

pub fn print(address: *const u8, size: i32) {
	const BYTES_PER_LINE: usize = 16;
	if size <= 0 {
		println!("Invalid stack size: {}", size);
		return;
	}

	let size = size as usize;

	for i in (0..size).step_by(BYTES_PER_LINE) {
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
	}
}
