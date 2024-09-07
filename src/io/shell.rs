use crate::io::keyboard;
use crate::print;

const INPUT_SIZE: usize = 256;
pub struct Shell {
	prompt: &'static str,
}

impl Shell {
	pub fn new() -> Self {
		Shell { prompt: "$> " }
	}

	pub fn run(&self) {
		keyboard::init();
		let mut input = [0u8; INPUT_SIZE];
		let mut len = 0;
		loop {
			self.display_prompt();
			self.read_input(&mut input, &mut len);
			// self.execute_command(&input[..len]);
			len = 0;
		}
	}

	fn display_prompt(&self) {
		print!("{}", self.prompt);
	}

	fn read_input(&self, input: &mut [u8], len: &mut usize) {
		loop {
			if let Some(c) = keyboard::read() {
				match c {
					'\n' => {
						print!("\n");
						break;
					}
					'\x7f' => {
						if *len > 0 {
							input[*len] = b'\0';
							*len -= 1;
							print!("\x7f");
						}
					}
					_ => {
						if *len < input.len() {
							input[*len] = c as u8;
							*len += 1;
							print!("{}", c);
						}
					}
				}
			}
		}
	}
}
