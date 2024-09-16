use crate::io::keyboard;
use crate::io::vga_buffer::WRITER;
use crate::{print, println};

const INPUT_SIZE: usize = 77;

pub struct Shell {
	prompt: &'static str,
	last_input1: [u8; INPUT_SIZE],
	last_len1: usize,
	last_input2: [u8; INPUT_SIZE],
	last_len2: usize,
	current_shell: u8,
}

impl Shell {
	pub fn new() -> Self {
		Shell {
			prompt: "$> ",
			last_input1: [0; INPUT_SIZE],
			last_len1: 0,
			last_input2: [0; INPUT_SIZE],
			last_len2: 0,
			current_shell: 1,
		}
	}

	pub fn run(&mut self) {
		keyboard::init();
		let mut input = [0u8; INPUT_SIZE];
		let mut len = 0;
		loop {
			self.display_prompt();
			self.read_input(&mut input, &mut len);
			self.execute_command(&input[..len]);
			len = 0;
		}
	}

	fn execute_command(&self, input: &[u8]) {
		use core::str;
		match str::from_utf8(input) {
			Ok("clear") => {
				for _i in 0..25 {
					WRITER.lock().clear_row(_i);
				}
			}
			Ok("reboot") => self.reboot(),
			Ok("stack") => self.print_kernel_stack(),
			Ok("halt") => self.halt(),
			Ok(command) => print!("Command not found: {}\n", command),
			Err(_) => print!("Command not UTF-8 input\n"),
		}
	}

	fn halt(&self) {
		use core::arch::asm;

		println!("System is halting...");
		loop {
			unsafe {
				asm!("hlt");
			}
		}
	}

	fn reboot(&self) {
		use core::arch::asm;
		unsafe {
			asm!(
				"cli",
				"2: in al, 0x64",
				"test al, 0x02",
				"jnz 2b",
				"mov al, 0xFE",
				"out 0x64, al",
				"hlt",
				options(noreturn)
			);
		}
	}

	fn print_kernel_stack(&self) {
		use core::arch::asm;
		let stack_pointer: i32;
		let base_pointer: i32;

		unsafe {
			asm!(
				"mov {0:e}, esp",
				"mov {1:e}, ebp",
				out(reg) stack_pointer,
				out(reg) base_pointer
			);
		}

		println!("Current Stack Pointer (ESP): {:#x}", stack_pointer);
		println!("Current Base Pointer (EBP): {:#x}", base_pointer);

		let stack_size = 64;
		for i in 0..(stack_size / 4) {
			unsafe {
				let stack_value: u64 = *((stack_pointer as *const u64).offset(i as isize));
				println!("Stack[{}]: {:#x}", i, stack_value);
			}
		}
	}

	fn display_prompt(&self) {
		print!("{}", self.prompt);
	}

	fn read_input(&mut self, input: &mut [u8], len: &mut usize) {
		loop {
			if let Some(c) = keyboard::read() {
				match c {
					'\n' => {
						print!("\n");
						break;
					}
					'\x7f' => {
						if *len > 0 {
							*len -= 1;
							input[*len] = b'\0';
							print!("{}", '\x7f');
						}
					}
					'\x01' => {
						self.switch_shell(1, input, len);
					}
					'\x02' => {
						self.switch_shell(2, input, len);
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

	fn switch_shell(&mut self, new_shell: u8, input: &mut [u8], len: &mut usize) {
		if self.current_shell != new_shell {
			if self.current_shell == 1 {
				self.last_input1[..*len].copy_from_slice(&input[..*len]);
				self.last_len1 = *len;
				WRITER.lock().column_position -= *len + self.prompt.len();
			} else {
				self.last_input2[..*len].copy_from_slice(&input[..*len]);
				self.last_len2 = *len;
				WRITER.lock().column_position -= *len + self.prompt.len();
			}

			if new_shell == 1 {
				input[..self.last_len1].copy_from_slice(&self.last_input1[..self.last_len1]);
				*len = self.last_len1;
			} else {
				input[..self.last_len2].copy_from_slice(&self.last_input2[..self.last_len2]);
				*len = self.last_len2;
			}

			self.current_shell = new_shell;

			self.display_prompt();
			for item in input.iter().take(*len) {
				print!("{}", *item as char);
			}
		}
	}
}
