use crate::io::hexdump;
use crate::io::keyboard;
use crate::io::vga_buffer::WRITER;
use crate::memory::physicalmemory::BITMAP;
use crate::{print, println};

const INPUT_SIZE: usize = 77;
const TAB_SIZE: usize = 4;

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

	fn execute_command(&mut self, input: &[u8]) {
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
			Ok("bitmap") => self.bitmap(false),
			Ok("bitmap --all") => self.bitmap(true),
			Ok("help") => self.help(),
			Ok(command) => print!("Command not found: {}\n", command),
			Err(_) => print!("Command not UTF-8 input\n"),
		}
	}

	fn help(&self) {
		println!(
			"Usage: [COMMAND]
For print some information of kernel :
   stack        visualy see stack status with hex and char
   bitmap       visualy see allocated physical frame
      bitmap --all   visualy see all physical frame

Power management :
   halt         stop cpu before intrruepting (not implement)
   reboot       reboot the kernel

User experience : 
   help         presenting the commands in our kernel
"
		)
	}

	fn bitmap(&mut self, all_flag: bool) {
		let mut line_count = 0;

		for (i, entry) in BITMAP.lock().bitmap.iter().enumerate() {
			if all_flag {
				println!("Entry {}: {:032b}", i, entry);
				line_count += 1;
			}

			for j in 0..32 {
				let bit_status = (entry >> (31 - j)) & 1;
				let frame_index = i * 32 + j;
				let frame_address = frame_index * 0x1000;
				if bit_status == 1 || all_flag {
					println!(
						"Frame {} (0x{:08x}): {}",
						frame_index,
						frame_address,
						if bit_status == 1 { "Allocated" } else { "Free" }
					);
					line_count += 1;
				}

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
			}
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

		let stack_size = base_pointer - stack_pointer;
		println!("Stack size: {} bytes", stack_size);

		if stack_size > 8192 {
			println!("Stack size too large, Limiting to 8KB.");
			return;
		}

		hexdump::print(stack_pointer as *const u8, stack_size);
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
					'\x09' => {
						print!("{}", " ".repeat(TAB_SIZE));
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
