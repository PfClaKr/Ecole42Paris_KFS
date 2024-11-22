use crate::io::hexdump;
use crate::io::keyboard;
use crate::io::vga_buffer::WRITER;
use crate::memory::physicalmemory::BITMAP;
use crate::{print, println};
use spin::Mutex;

pub const INPUT_SIZE: usize = 77;
const TAB_SIZE: usize = 4;

pub static SHELL: Mutex<Shell> = Mutex::new(Shell {
	prompt: "$> ",
	last_input1: [0; INPUT_SIZE],
	last_len1: 0,
	last_input2: [0; INPUT_SIZE],
	last_len2: 0,
	current_shell: 1,
});

pub struct Shell {
	prompt: &'static str,
	last_input1: [u8; INPUT_SIZE],
	last_len1: usize,
	last_input2: [u8; INPUT_SIZE],
	last_len2: usize,
	current_shell: u8,
}

impl Shell {
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
			Ok("keymap") => self.keymap(),
			Ok("help") => self.help(),
			Ok("uptime") => self.uptime(),
			Ok(command) if command.starts_with("interrupt ") => self.interrupt(command),
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

Os management :
   interrupt <0-255>    make system interrupt
   halt                 stop cpu
   reboot               reboot the kernel

User experience : 
   keymap       change keyboard layout
   help         presenting the commands in our kernel
   uptime       show uptime this os
"
		)
	}

	fn uptime(&self) {
		println!(
			"KFS os running while {} seconds.",
			unsafe { crate::include::interrupts::TICKS } / 100
		);
	}

	fn keymap(&self) {
		let keymap = unsafe { keyboard::KEYMAP } as usize;
		println!(
			"Your keyboard layout is {}.",
			if keymap == 0 { "English" } else { "French" }
		);
		println!("If you want to change, press 'y' or 'n' for abort.");
		loop {
			let input = keyboard::read();
			match input {
				Some('y') => {
					if keymap == 0 {
						unsafe { keyboard::KEYMAP = keyboard::Keymap::FR };
					} else {
						unsafe { keyboard::KEYMAP = keyboard::Keymap::EN };
					}
					println!("layout changed.");
					return;
				}
				Some('n') => {
					println!("aborted");
					return;
				}
				_ => continue,
			}
		}
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
		let stack_pointer: usize;
		let base_pointer: usize;

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

		let stack_size = if stack_pointer > base_pointer {
			stack_pointer - base_pointer
		} else {
			base_pointer - stack_pointer
		};

		println!("Stack size: {} kb", stack_size / 1024);
		println!("For hexdump, press 'y' or 'n' for abort.");
		loop {
			let input = keyboard::read();
			match input {
				Some('y') => {
					let (start, size) = if stack_pointer > base_pointer {
						(base_pointer, stack_size)
					} else {
						(stack_pointer, stack_size)
					};

					hexdump::print(start as *const u8, size as i32);
					return;
				}
				Some('n') => {
					println!("aborted");
					return;
				}
				_ => continue,
			}
		}
	}

	pub fn display_prompt(&self) {
		print!("{}", self.prompt);
	}

	pub fn read_input(&mut self, input: &mut [u8; INPUT_SIZE], len: &mut usize) {
		if let Some(c) = keyboard::read() {
			match c {
				'\n' => {
					// let input = self.input;
					print!("\n");
					self.execute_command(&input[..*len]);
					*len = 0;
					self.display_prompt();
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

	fn switch_shell(&mut self, new_shell: u8, input: &mut [u8; INPUT_SIZE], len: &mut usize) {
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

	fn interrupt(&self, input: &str) {
		if let Some(interrupt_number) = input.strip_prefix("interrupt ") {
			match interrupt_number.parse::<u8>() {
				Ok(number) => {
					println!("Triggering interrupt: {:#x}", number);
					match number {
						0 => unsafe { core::arch::asm!("int 0") },
						1 => unsafe { core::arch::asm!("int 1") },
						2 => unsafe { core::arch::asm!("int 2") },
						3 => unsafe { core::arch::asm!("int 3") },
						4 => unsafe { core::arch::asm!("int 4") },
						5 => unsafe { core::arch::asm!("int 5") },
						6 => unsafe { core::arch::asm!("int 6") },
						7 => unsafe { core::arch::asm!("int 7") },
						8 => unsafe { core::arch::asm!("int 8") },
						9 => unsafe { core::arch::asm!("int 9") },
						10 => unsafe { core::arch::asm!("int 10") },
						11 => unsafe { core::arch::asm!("int 11") },
						12 => unsafe { core::arch::asm!("int 12") },
						13 => unsafe { core::arch::asm!("int 13") },
						14 => unsafe { core::arch::asm!("int 14") },
						15 => unsafe { core::arch::asm!("int 15") },
						16 => unsafe { core::arch::asm!("int 16") },
						17 => unsafe { core::arch::asm!("int 17") },
						18 => unsafe { core::arch::asm!("int 18") },
						19 => unsafe { core::arch::asm!("int 19") },
						20 => unsafe { core::arch::asm!("int 20") },
						21 => unsafe { core::arch::asm!("int 21") },
						22 => unsafe { core::arch::asm!("int 22") },
						23 => unsafe { core::arch::asm!("int 23") },
						24 => unsafe { core::arch::asm!("int 24") },
						25 => unsafe { core::arch::asm!("int 25") },
						26 => unsafe { core::arch::asm!("int 26") },
						27 => unsafe { core::arch::asm!("int 27") },
						28 => unsafe { core::arch::asm!("int 28") },
						29 => unsafe { core::arch::asm!("int 29") },
						30 => unsafe { core::arch::asm!("int 30") },
						31 => unsafe { core::arch::asm!("int 31") },
						32 => unsafe { core::arch::asm!("int 32") },
						33 => unsafe { core::arch::asm!("int 33") },
						34 => unsafe { core::arch::asm!("int 34") },
						35 => unsafe { core::arch::asm!("int 35") },
						36 => unsafe { core::arch::asm!("int 36") },
						37 => unsafe { core::arch::asm!("int 37") },
						38 => unsafe { core::arch::asm!("int 38") },
						39 => unsafe { core::arch::asm!("int 39") },
						40 => unsafe { core::arch::asm!("int 40") },
						41 => unsafe { core::arch::asm!("int 41") },
						42 => unsafe { core::arch::asm!("int 42") },
						43 => unsafe { core::arch::asm!("int 43") },
						44 => unsafe { core::arch::asm!("int 44") },
						45 => unsafe { core::arch::asm!("int 45") },
						46 => unsafe { core::arch::asm!("int 46") },
						47 => unsafe { core::arch::asm!("int 47") },
						48 => unsafe { core::arch::asm!("int 48") },
						49 => unsafe { core::arch::asm!("int 49") },
						50 => unsafe { core::arch::asm!("int 50") },
						51 => unsafe { core::arch::asm!("int 51") },
						52 => unsafe { core::arch::asm!("int 52") },
						53 => unsafe { core::arch::asm!("int 53") },
						54 => unsafe { core::arch::asm!("int 54") },
						55 => unsafe { core::arch::asm!("int 55") },
						56 => unsafe { core::arch::asm!("int 56") },
						57 => unsafe { core::arch::asm!("int 57") },
						58 => unsafe { core::arch::asm!("int 58") },
						59 => unsafe { core::arch::asm!("int 59") },
						60 => unsafe { core::arch::asm!("int 60") },
						61 => unsafe { core::arch::asm!("int 61") },
						62 => unsafe { core::arch::asm!("int 62") },
						63 => unsafe { core::arch::asm!("int 63") },
						64 => unsafe { core::arch::asm!("int 64") },
						65 => unsafe { core::arch::asm!("int 65") },
						66 => unsafe { core::arch::asm!("int 66") },
						67 => unsafe { core::arch::asm!("int 67") },
						68 => unsafe { core::arch::asm!("int 68") },
						69 => unsafe { core::arch::asm!("int 69") },
						70 => unsafe { core::arch::asm!("int 70") },
						71 => unsafe { core::arch::asm!("int 71") },
						72 => unsafe { core::arch::asm!("int 72") },
						73 => unsafe { core::arch::asm!("int 73") },
						74 => unsafe { core::arch::asm!("int 74") },
						75 => unsafe { core::arch::asm!("int 75") },
						76 => unsafe { core::arch::asm!("int 76") },
						77 => unsafe { core::arch::asm!("int 77") },
						78 => unsafe { core::arch::asm!("int 78") },
						79 => unsafe { core::arch::asm!("int 79") },
						80 => unsafe { core::arch::asm!("int 80") },
						81 => unsafe { core::arch::asm!("int 81") },
						82 => unsafe { core::arch::asm!("int 82") },
						83 => unsafe { core::arch::asm!("int 83") },
						84 => unsafe { core::arch::asm!("int 84") },
						85 => unsafe { core::arch::asm!("int 85") },
						86 => unsafe { core::arch::asm!("int 86") },
						87 => unsafe { core::arch::asm!("int 87") },
						88 => unsafe { core::arch::asm!("int 88") },
						89 => unsafe { core::arch::asm!("int 89") },
						90 => unsafe { core::arch::asm!("int 90") },
						91 => unsafe { core::arch::asm!("int 91") },
						92 => unsafe { core::arch::asm!("int 92") },
						93 => unsafe { core::arch::asm!("int 93") },
						94 => unsafe { core::arch::asm!("int 94") },
						95 => unsafe { core::arch::asm!("int 95") },
						96 => unsafe { core::arch::asm!("int 96") },
						97 => unsafe { core::arch::asm!("int 97") },
						98 => unsafe { core::arch::asm!("int 98") },
						99 => unsafe { core::arch::asm!("int 99") },
						100 => unsafe { core::arch::asm!("int 100") },
						101 => unsafe { core::arch::asm!("int 101") },
						102 => unsafe { core::arch::asm!("int 102") },
						103 => unsafe { core::arch::asm!("int 103") },
						104 => unsafe { core::arch::asm!("int 104") },
						105 => unsafe { core::arch::asm!("int 105") },
						106 => unsafe { core::arch::asm!("int 106") },
						107 => unsafe { core::arch::asm!("int 107") },
						108 => unsafe { core::arch::asm!("int 108") },
						109 => unsafe { core::arch::asm!("int 109") },
						110 => unsafe { core::arch::asm!("int 110") },
						111 => unsafe { core::arch::asm!("int 111") },
						112 => unsafe { core::arch::asm!("int 112") },
						113 => unsafe { core::arch::asm!("int 113") },
						114 => unsafe { core::arch::asm!("int 114") },
						115 => unsafe { core::arch::asm!("int 115") },
						116 => unsafe { core::arch::asm!("int 116") },
						117 => unsafe { core::arch::asm!("int 117") },
						118 => unsafe { core::arch::asm!("int 118") },
						119 => unsafe { core::arch::asm!("int 119") },
						120 => unsafe { core::arch::asm!("int 120") },
						121 => unsafe { core::arch::asm!("int 121") },
						122 => unsafe { core::arch::asm!("int 122") },
						123 => unsafe { core::arch::asm!("int 123") },
						124 => unsafe { core::arch::asm!("int 124") },
						125 => unsafe { core::arch::asm!("int 125") },
						126 => unsafe { core::arch::asm!("int 126") },
						127 => unsafe { core::arch::asm!("int 127") },
						128 => unsafe { core::arch::asm!("int 128") },
						129 => unsafe { core::arch::asm!("int 129") },
						130 => unsafe { core::arch::asm!("int 130") },
						131 => unsafe { core::arch::asm!("int 131") },
						132 => unsafe { core::arch::asm!("int 132") },
						133 => unsafe { core::arch::asm!("int 133") },
						134 => unsafe { core::arch::asm!("int 134") },
						135 => unsafe { core::arch::asm!("int 135") },
						136 => unsafe { core::arch::asm!("int 136") },
						137 => unsafe { core::arch::asm!("int 137") },
						138 => unsafe { core::arch::asm!("int 138") },
						139 => unsafe { core::arch::asm!("int 139") },
						140 => unsafe { core::arch::asm!("int 140") },
						141 => unsafe { core::arch::asm!("int 141") },
						142 => unsafe { core::arch::asm!("int 142") },
						143 => unsafe { core::arch::asm!("int 143") },
						144 => unsafe { core::arch::asm!("int 144") },
						145 => unsafe { core::arch::asm!("int 145") },
						146 => unsafe { core::arch::asm!("int 146") },
						147 => unsafe { core::arch::asm!("int 147") },
						148 => unsafe { core::arch::asm!("int 148") },
						149 => unsafe { core::arch::asm!("int 149") },
						150 => unsafe { core::arch::asm!("int 150") },
						151 => unsafe { core::arch::asm!("int 151") },
						152 => unsafe { core::arch::asm!("int 152") },
						153 => unsafe { core::arch::asm!("int 153") },
						154 => unsafe { core::arch::asm!("int 154") },
						155 => unsafe { core::arch::asm!("int 155") },
						156 => unsafe { core::arch::asm!("int 156") },
						157 => unsafe { core::arch::asm!("int 157") },
						158 => unsafe { core::arch::asm!("int 158") },
						159 => unsafe { core::arch::asm!("int 159") },
						160 => unsafe { core::arch::asm!("int 160") },
						161 => unsafe { core::arch::asm!("int 161") },
						162 => unsafe { core::arch::asm!("int 162") },
						163 => unsafe { core::arch::asm!("int 163") },
						164 => unsafe { core::arch::asm!("int 164") },
						165 => unsafe { core::arch::asm!("int 165") },
						166 => unsafe { core::arch::asm!("int 166") },
						167 => unsafe { core::arch::asm!("int 167") },
						168 => unsafe { core::arch::asm!("int 168") },
						169 => unsafe { core::arch::asm!("int 169") },
						170 => unsafe { core::arch::asm!("int 170") },
						171 => unsafe { core::arch::asm!("int 171") },
						172 => unsafe { core::arch::asm!("int 172") },
						173 => unsafe { core::arch::asm!("int 173") },
						174 => unsafe { core::arch::asm!("int 174") },
						175 => unsafe { core::arch::asm!("int 175") },
						176 => unsafe { core::arch::asm!("int 176") },
						177 => unsafe { core::arch::asm!("int 177") },
						178 => unsafe { core::arch::asm!("int 178") },
						179 => unsafe { core::arch::asm!("int 179") },
						180 => unsafe { core::arch::asm!("int 180") },
						181 => unsafe { core::arch::asm!("int 181") },
						182 => unsafe { core::arch::asm!("int 182") },
						183 => unsafe { core::arch::asm!("int 183") },
						184 => unsafe { core::arch::asm!("int 184") },
						185 => unsafe { core::arch::asm!("int 185") },
						186 => unsafe { core::arch::asm!("int 186") },
						187 => unsafe { core::arch::asm!("int 187") },
						188 => unsafe { core::arch::asm!("int 188") },
						189 => unsafe { core::arch::asm!("int 189") },
						190 => unsafe { core::arch::asm!("int 190") },
						191 => unsafe { core::arch::asm!("int 191") },
						192 => unsafe { core::arch::asm!("int 192") },
						193 => unsafe { core::arch::asm!("int 193") },
						194 => unsafe { core::arch::asm!("int 194") },
						195 => unsafe { core::arch::asm!("int 195") },
						196 => unsafe { core::arch::asm!("int 196") },
						197 => unsafe { core::arch::asm!("int 197") },
						198 => unsafe { core::arch::asm!("int 198") },
						199 => unsafe { core::arch::asm!("int 199") },
						200 => unsafe { core::arch::asm!("int 200") },
						201 => unsafe { core::arch::asm!("int 201") },
						202 => unsafe { core::arch::asm!("int 202") },
						203 => unsafe { core::arch::asm!("int 203") },
						204 => unsafe { core::arch::asm!("int 204") },
						205 => unsafe { core::arch::asm!("int 205") },
						206 => unsafe { core::arch::asm!("int 206") },
						207 => unsafe { core::arch::asm!("int 207") },
						208 => unsafe { core::arch::asm!("int 208") },
						209 => unsafe { core::arch::asm!("int 209") },
						210 => unsafe { core::arch::asm!("int 210") },
						211 => unsafe { core::arch::asm!("int 211") },
						212 => unsafe { core::arch::asm!("int 212") },
						213 => unsafe { core::arch::asm!("int 213") },
						214 => unsafe { core::arch::asm!("int 214") },
						215 => unsafe { core::arch::asm!("int 215") },
						216 => unsafe { core::arch::asm!("int 216") },
						217 => unsafe { core::arch::asm!("int 217") },
						218 => unsafe { core::arch::asm!("int 218") },
						219 => unsafe { core::arch::asm!("int 219") },
						220 => unsafe { core::arch::asm!("int 220") },
						221 => unsafe { core::arch::asm!("int 221") },
						222 => unsafe { core::arch::asm!("int 222") },
						223 => unsafe { core::arch::asm!("int 223") },
						224 => unsafe { core::arch::asm!("int 224") },
						225 => unsafe { core::arch::asm!("int 225") },
						226 => unsafe { core::arch::asm!("int 226") },
						227 => unsafe { core::arch::asm!("int 227") },
						228 => unsafe { core::arch::asm!("int 228") },
						229 => unsafe { core::arch::asm!("int 229") },
						230 => unsafe { core::arch::asm!("int 230") },
						231 => unsafe { core::arch::asm!("int 231") },
						232 => unsafe { core::arch::asm!("int 232") },
						233 => unsafe { core::arch::asm!("int 233") },
						234 => unsafe { core::arch::asm!("int 234") },
						235 => unsafe { core::arch::asm!("int 235") },
						236 => unsafe { core::arch::asm!("int 236") },
						237 => unsafe { core::arch::asm!("int 237") },
						238 => unsafe { core::arch::asm!("int 238") },
						239 => unsafe { core::arch::asm!("int 239") },
						240 => unsafe { core::arch::asm!("int 240") },
						241 => unsafe { core::arch::asm!("int 241") },
						242 => unsafe { core::arch::asm!("int 242") },
						243 => unsafe { core::arch::asm!("int 243") },
						244 => unsafe { core::arch::asm!("int 244") },
						245 => unsafe { core::arch::asm!("int 245") },
						246 => unsafe { core::arch::asm!("int 246") },
						247 => unsafe { core::arch::asm!("int 247") },
						248 => unsafe { core::arch::asm!("int 248") },
						249 => unsafe { core::arch::asm!("int 249") },
						250 => unsafe { core::arch::asm!("int 250") },
						251 => unsafe { core::arch::asm!("int 251") },
						252 => unsafe { core::arch::asm!("int 252") },
						253 => unsafe { core::arch::asm!("int 253") },
						254 => unsafe { core::arch::asm!("int 254") },
						255 => unsafe { core::arch::asm!("int 255") },
					}
				}
				Err(_) => {
					println!("Invalid interrupt number: {}", interrupt_number);
				}
			}
		} else {
			println!("Invalid input. Please use the format 'interrupt <number>'.");
		}
	}
}
