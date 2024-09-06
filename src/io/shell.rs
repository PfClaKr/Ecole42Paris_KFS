use crate::io::keyboard;

fn print_to_terminal(c: char) {
	let vga_buffer = 0xb8000 as *mut u8;

	unsafe {
		*vga_buffer.offset(0) = c as u8;
		*vga_buffer.offset(1) = 0x07;
	}
}

pub fn new() {
	keyboard::init();
	loop {
		if let Some(c) = keyboard::read() {
			print_to_terminal(c);
		}
	}
}
