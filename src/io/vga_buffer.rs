#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
	Black = 0,
	Blue = 1,
	Green = 2,
	Cyan = 3,
	Red = 4,
	Magenta = 5,
	Brown = 6,
	LightGray = 7,
	DarkGray = 8,
	LightBlue = 9,
	LightGreen = 10,
	LightCyan = 11,
	LightRed = 12,
	Pink = 13,
	Yellow = 14,
	White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorMap(u8);

impl ColorMap {
	pub fn from_str(color: usize) -> Option<Color> {
		match color {
			0 => Some(Color::Black),
			1 => Some(Color::Blue),
			2 => Some(Color::Green),
			3 => Some(Color::Cyan),
			4 => Some(Color::Red),
			5 => Some(Color::Magenta),
			6 => Some(Color::Brown),
			7 => Some(Color::LightGray),
			8 => Some(Color::DarkGray),
			9 => Some(Color::LightBlue),
			10 => Some(Color::LightGreen),
			11 => Some(Color::LightCyan),
			12 => Some(Color::LightRed),
			13 => Some(Color::Pink),
			14 => Some(Color::Yellow),
			15 => Some(Color::White),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
	fn new(foreground: Color, background: Color) -> ColorCode {
		ColorCode((background as u8) << 4 | (foreground as u8))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
	ascii_character: u8,
	color_code: ColorCode,
}

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

use volatile::Volatile;

#[repr(transparent)]
struct Buffer {
	chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
	pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
		column_position: 0,
		color_code: ColorCode::new(Color::White, Color::Black),
		buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
		skip: 0,
	});
}

pub struct Writer {
	pub column_position: usize,
	color_code: ColorCode,
	buffer: &'static mut Buffer,
	skip: usize,
}

impl Writer {
	pub fn write_byte(&mut self, byte: u8) {
		match byte {
			b'\n' => self.new_line(),
			0x7f => {
				if self.column_position > 0 {
					self.column_position -= 1;
					set_cursor(self.column_position);
					self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(ScreenChar {
						ascii_character: b' ',
						color_code: self.color_code,
					});
				}
			}
			0x7f => {
				if self.column_position > 0 {
					self.column_position -= 1;
					self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(ScreenChar {
						ascii_character: b' ',
						color_code: self.color_code,
					});
				}
			}
			byte => {
				if self.column_position >= BUFFER_WIDTH {
					self.new_line();
				}
				let row = BUFFER_HEIGHT - 1;
				let col = self.column_position;

				let color_code = self.color_code;
				self.buffer.chars[row][col].write(ScreenChar {
					ascii_character: byte,
					color_code,
				});
				self.column_position += 1;
				set_cursor(self.column_position);
			}
		}
	}

	#[inline(never)] // debug
	pub fn write_string(&mut self, s: &str) {
		let mut index;
		index = 0;
		for byte in s.bytes() {
			if self.skip > 0 {
				self.skip -= 1;
				index += 1;
				continue;
			}
			match byte {
				b'\x1b' => {
					if !self.sgr(&s[index..]) {
						self.write_byte(0xfe);
					}
				}
				0x20..=0x7e | b'\n' => {
					self.write_byte(byte);
				}
				0x7f => self.write_byte(0x7f),
				_ => {
					self.write_byte(0xfe);
				}
			}
			index += 1;
		}
	}

	fn new_line(&mut self) {
		for row in 1..BUFFER_HEIGHT {
			for col in 0..BUFFER_WIDTH {
				let character = self.buffer.chars[row][col].read();
				self.buffer.chars[row - 1][col].write(character);
			}
		}
		self.clear_row(BUFFER_HEIGHT - 1);
		self.column_position = 0;
		set_cursor(self.column_position);
	}

	fn clear_row(&mut self, row: usize) {
		let blank = ScreenChar {
			ascii_character: b' ',
			color_code: self.color_code,
		};
		for col in 0..BUFFER_WIDTH {
			self.buffer.chars[row][col].write(blank);
		}
	}

	#[inline(never)] // debug
	fn syntax_check(&mut self, s: &str) -> bool {
		use crate::include::string;
		if !s.contains('\x1b') || !s.contains('m') {
			return false;
		}
		if let Some(substr) = string::substring_between(s, '\x1b', 'm') {
			if substr.contains(' ') {
				return false;
			}
			let mut ss_chars = substr.chars();

			if ss_chars.next() != Some('[') {
				return false;
			}

			let color_code = ss_chars.next();
			if color_code.is_none() || !color_code.unwrap().is_ascii_digit() {
				return false;
			}

			let tbd = ss_chars.next();
			match tbd {
				Some(c) if c.is_ascii_digit() => {
					if ss_chars.next() == Some(';') {
						return ss_chars.next().is_none();
					}
				}
				Some(';') => {
					return ss_chars.next().is_none();
				}
				_ => return false,
			}
		}
		false
	}

	#[inline(never)] // debug
	fn sgr(&mut self, s: &str) -> bool {
		// Up Down Right Left : A B C D
		// SGR : m
		if !self.syntax_check(s) {
			return false;
		}
		let slice;
		if let Some(end) = s.find('m') {
			slice = &s[1..end]; // from ESC character (excluded) to 'm' (with 'm' excluded)
		} else {
			return false; // wrong syntax: missing m
		};
		if slice.len() < 3 {
			// minimal count of sgr syntax: \x1b[0;m
			return false;
		}

		use crate::include::string;
		if slice.starts_with('[') && slice.ends_with(';') {
			if let Some(substr) = string::substring_between(slice, '[', ';') {
				if let Ok(color_code) = string::atoi(substr) {
					if let Some(color) = ColorMap::from_str(color_code) {
						self.color_code = ColorCode::new(color, Color::Black);
					} else {
						self.color_code = ColorCode::new(Color::White, Color::Black); // default color
					}
					self.skip = slice.len() + 1; // ESC and 'm'
					return true;
				}
			}
		}
		false
	}
}

use core::fmt;

impl fmt::Write for Writer {
	fn write_str(&mut self, s: &str) -> fmt::Result {
		self.write_string(s);
		Ok(())
	}
}

static mut VGA1_BUFFER: [[u8; BUFFER_WIDTH]; BUFFER_HEIGHT] = [[0; BUFFER_WIDTH]; BUFFER_HEIGHT];
static mut VGA2_BUFFER: [[u8; BUFFER_WIDTH]; BUFFER_HEIGHT] = [[0; BUFFER_WIDTH]; BUFFER_HEIGHT];
static mut CURRENT_VGA: u8 = 1;

pub fn switch(new_vga: u8) {
	unsafe {
		if CURRENT_VGA != new_vga {
			if CURRENT_VGA == 1 {
				save_vga(&raw mut VGA1_BUFFER);
			} else {
				save_vga(&raw mut VGA2_BUFFER);
			}

			if new_vga == 1 {
				load_vga(&raw const VGA1_BUFFER);
			} else {
				load_vga(&raw const VGA2_BUFFER);
			}

			CURRENT_VGA = new_vga;
		}
	}
}

fn read_char_at(x: usize, y: usize) -> u8 {
	unsafe {
		let vga_buffer = 0xb8000 as *const Buffer;
		(*vga_buffer).chars[y][x].read().ascii_character
	}
}

fn write_char_at(character: u8, x: usize, y: usize, color: u8) {
	unsafe {
		let vga_buffer = 0xb8000 as *mut Buffer;
		(*vga_buffer).chars[y][x].write(ScreenChar {
			ascii_character: character,
			color_code: ColorCode(color),
		});
	}
}

fn save_vga(buffer_ptr: *mut [[u8; BUFFER_WIDTH]; BUFFER_HEIGHT]) {
	unsafe {
		for y in 0..BUFFER_HEIGHT {
			for x in 0..BUFFER_WIDTH {
				(*buffer_ptr)[y][x] = read_char_at(x, y);
			}
		}
	}
}

fn load_vga(buffer_ptr: *const [[u8; BUFFER_WIDTH]; BUFFER_HEIGHT]) {
	unsafe {
		for y in 0..BUFFER_HEIGHT {
			for x in 0..BUFFER_WIDTH {
				write_char_at((*buffer_ptr)[y][x], x, y, 15);
			}
		}
	}
}

use crate::include::asm_utile;

fn set_cursor(x: usize) {
	let position = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH + x;
	unsafe {
		asm_utile::outb(0x3D4, 14);
		asm_utile::outb(0x3D5, (position >> 8) as u8);
		asm_utile::outb(0x3D4, 15);
		asm_utile::outb(0x3D5, (position & 0xFF) as u8);
	}
}
