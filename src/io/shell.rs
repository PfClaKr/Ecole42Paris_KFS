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
			Ok("print_stack") => self.print_kernel_stack(),
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
				"cli",            // 인터럽트 비활성화
				"2: in al, 0x64", // 8042 상태 레지스터 읽기
				"test al, 0x02",  // 입력 버퍼가 비었는지 확인
				"jnz 2b",         // 입력 버퍼가 비어있지 않으면 대기
				"mov al, 0xFE",   // 재부팅 명령(0xFE)을 AL 레지스터에 로드
				"out 0x64, al",   // 명령을 0x64 포트에 전송 (8042 키보드 컨트롤러)
				"hlt",            // CPU 멈춤 (재부팅이 실패할 경우를 대비한 대기)
				options(noreturn)
			);
		}
	}

	fn print_kernel_stack(&self) {
		use core::arch::asm;
		let stack_pointer: i32;
		let base_pointer: i32;

		unsafe {
			// 인라인 어셈블리를 통해 스택 포인터(RSP)와 베이스 포인터(RBP) 읽기
			asm!(
				"mov {0:e}, esp",   // 현재 스택 포인터를 stack_pointer에 저장
				"mov {1:e}, ebp",   // 현재 베이스 포인터를 base_pointer에 저장
				out(reg) stack_pointer,
				out(reg) base_pointer
			);
		}

		// 스택 포인터와 베이스 포인터 값을 출력
		println!("Current Stack Pointer (ESP): {:#x}", stack_pointer);
		println!("Current Base Pointer (EBP): {:#x}", base_pointer);

		// 스택의 특정 범위를 출력하는 부분을 추가할 수 있음
		let stack_size = 64; // 예시로 64바이트 크기
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
