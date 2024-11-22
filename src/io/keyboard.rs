use crate::include::asm_utile;
use crate::io::vga_buffer;
use spin::Mutex;

const KEYBOARD_DATA_PORT: u16 = 0x60;
const SHIFT_LEFT: u8 = 0x2A;
const SHIFT_RIGHT: u8 = 0x36;
const SHIFT_LEFT_RELEASE: u8 = 0x2A + 0x80;
const SHIFT_RIGHT_RELEASE: u8 = 0x36 + 0x80;

static SHIFT_PRESSED: Mutex<bool> = Mutex::new(false);
static mut LAST_SCANCODE: u8 = 0;
pub static mut KEYMAP: Keymap = Keymap::EN;

#[derive(PartialEq, Copy, Clone)]
pub enum Keymap {
	EN,
	FR,
}

pub fn read() -> Option<char> {
	let scancode: u8;
	let key: Option<char>;
	let mut shift: spin::MutexGuard<'_, bool> = SHIFT_PRESSED.lock();

	unsafe {
		scancode = asm_utile::inb(KEYBOARD_DATA_PORT);
	}

	unsafe {
		if scancode == LAST_SCANCODE {
			return None;
		}
		LAST_SCANCODE = scancode;
	}

	match scancode {
		SHIFT_LEFT | SHIFT_RIGHT => {
			*shift = true;
			return None;
		}
		SHIFT_LEFT_RELEASE | SHIFT_RIGHT_RELEASE => {
			*shift = false;
			return None;
		}
		0x1C => {
			return Some('\n');
		}
		0x0E => {
			return Some('\x7f');
		}
		0x3B | 0x3C => {
			if scancode == 0x3B {
				vga_buffer::switch(1);
				return Some('\x01');
			} else {
				vga_buffer::switch(2);
				return Some('\x02');
			}
		}
		_ => {
			if scancode & 0x80 == 0 {
				if unsafe { KEYMAP == Keymap::EN } {
					key = if *shift {
						TO_SHIFT_ASCII_EN[scancode as usize]
					} else {
						TO_ASCII_EN[scancode as usize]
					};
				} else {
					key = if *shift {
						TO_SHIFT_ASCII_FR[scancode as usize]
					} else {
						TO_ASCII_FR[scancode as usize]
					}
				}
				return key;
			}
		}
	}
	None
}

static TO_ASCII_EN: [Option<char>; 256] = {
	let mut table = [None; 256];

	table[0x02] = Some('1');
	table[0x03] = Some('2');
	table[0x04] = Some('3');
	table[0x05] = Some('4');
	table[0x06] = Some('5');
	table[0x07] = Some('6');
	table[0x08] = Some('7');
	table[0x09] = Some('8');
	table[0x0A] = Some('9');
	table[0x0B] = Some('0');
	table[0x1E] = Some('a');
	table[0x30] = Some('b');
	table[0x2E] = Some('c');
	table[0x20] = Some('d');
	table[0x12] = Some('e');
	table[0x21] = Some('f');
	table[0x22] = Some('g');
	table[0x23] = Some('h');
	table[0x17] = Some('i');
	table[0x24] = Some('j');
	table[0x25] = Some('k');
	table[0x26] = Some('l');
	table[0x32] = Some('m');
	table[0x31] = Some('n');
	table[0x18] = Some('o');
	table[0x19] = Some('p');
	table[0x10] = Some('q');
	table[0x13] = Some('r');
	table[0x1F] = Some('s');
	table[0x14] = Some('t');
	table[0x16] = Some('u');
	table[0x2F] = Some('v');
	table[0x11] = Some('w');
	table[0x2D] = Some('x');
	table[0x15] = Some('y');
	table[0x2C] = Some('z');
	table[0x39] = Some(' ');

	table[0x0C] = Some('-');
	table[0x0D] = Some('=');
	table[0x1A] = Some('[');
	table[0x1B] = Some(']');
	table[0x2B] = Some('\\');
	table[0x27] = Some(';');
	table[0x28] = Some('\'');
	table[0x33] = Some(',');
	table[0x34] = Some('.');
	table[0x35] = Some('/');

	table
};

static TO_SHIFT_ASCII_EN: [Option<char>; 256] = {
	let mut table = [None; 256];

	table[0x02] = Some('!');
	table[0x03] = Some('@');
	table[0x04] = Some('#');
	table[0x05] = Some('$');
	table[0x06] = Some('%');
	table[0x07] = Some('^');
	table[0x08] = Some('&');
	table[0x09] = Some('*');
	table[0x0A] = Some('(');
	table[0x0B] = Some(')');
	table[0x1E] = Some('A');
	table[0x30] = Some('B');
	table[0x2E] = Some('C');
	table[0x20] = Some('D');
	table[0x12] = Some('E');
	table[0x21] = Some('F');
	table[0x22] = Some('G');
	table[0x23] = Some('H');
	table[0x17] = Some('I');
	table[0x24] = Some('J');
	table[0x25] = Some('K');
	table[0x26] = Some('L');
	table[0x32] = Some('M');
	table[0x31] = Some('N');
	table[0x18] = Some('O');
	table[0x19] = Some('P');
	table[0x10] = Some('Q');
	table[0x13] = Some('R');
	table[0x1F] = Some('S');
	table[0x14] = Some('T');
	table[0x16] = Some('U');
	table[0x2F] = Some('V');
	table[0x11] = Some('W');
	table[0x2D] = Some('X');
	table[0x15] = Some('Y');
	table[0x2C] = Some('Z');
	table[0x39] = Some(' ');

	table[0x0C] = Some('_');
	table[0x0D] = Some('+');
	table[0x1A] = Some('{');
	table[0x1B] = Some('}');
	table[0x2B] = Some('|');
	table[0x27] = Some(':');
	table[0x28] = Some('"');
	table[0x33] = Some('<');
	table[0x34] = Some('>');
	table[0x35] = Some('?');

	table
};

static TO_ASCII_FR: [Option<char>; 256] = {
	let mut table = [None; 256];

	table[0x02] = Some('&');
	table[0x03] = Some('é');
	table[0x04] = Some('"');
	table[0x05] = Some('\'');
	table[0x06] = Some('(');
	table[0x07] = Some('-');
	table[0x08] = Some('è');
	table[0x09] = Some('_');
	table[0x0A] = Some('ç');
	table[0x0B] = Some('à');

	table[0x10] = Some('a');
	table[0x11] = Some('z');
	table[0x12] = Some('e');
	table[0x13] = Some('r');
	table[0x14] = Some('t');
	table[0x15] = Some('y');
	table[0x16] = Some('u');
	table[0x17] = Some('i');
	table[0x18] = Some('o');
	table[0x19] = Some('p');
	table[0x1E] = Some('q');
	table[0x1F] = Some('s');
	table[0x20] = Some('d');
	table[0x21] = Some('f');
	table[0x22] = Some('g');
	table[0x23] = Some('h');
	table[0x24] = Some('j');
	table[0x25] = Some('k');
	table[0x26] = Some('l');
	table[0x2C] = Some('w');
	table[0x2D] = Some('x');
	table[0x2E] = Some('c');
	table[0x2F] = Some('v');
	table[0x30] = Some('b');
	table[0x31] = Some('n');
	table[0x32] = Some('m');

	table[0x33] = Some(',');
	table[0x34] = Some(';');
	table[0x35] = Some(':');
	table[0x39] = Some(' ');
	table[0x0C] = Some(')');
	table[0x0D] = Some('=');
	table[0x1A] = Some('^');
	table[0x1B] = Some('$');
	table[0x2B] = Some('\\');

	table
};

static TO_SHIFT_ASCII_FR: [Option<char>; 256] = {
	let mut table = [None; 256];

	table[0x02] = Some('1');
	table[0x03] = Some('2');
	table[0x04] = Some('3');
	table[0x05] = Some('4');
	table[0x06] = Some('5');
	table[0x07] = Some('6');
	table[0x08] = Some('7');
	table[0x09] = Some('8');
	table[0x0A] = Some('9');
	table[0x0B] = Some('0');

	table[0x10] = Some('A');
	table[0x11] = Some('Z');
	table[0x12] = Some('E');
	table[0x13] = Some('R');
	table[0x14] = Some('T');
	table[0x15] = Some('Y');
	table[0x16] = Some('U');
	table[0x17] = Some('I');
	table[0x18] = Some('O');
	table[0x19] = Some('P');
	table[0x1E] = Some('Q');
	table[0x1F] = Some('S');
	table[0x20] = Some('D');
	table[0x21] = Some('F');
	table[0x22] = Some('G');
	table[0x23] = Some('H');
	table[0x24] = Some('J');
	table[0x25] = Some('K');
	table[0x26] = Some('L');
	table[0x2C] = Some('W');
	table[0x2D] = Some('X');
	table[0x2E] = Some('C');
	table[0x2F] = Some('V');
	table[0x30] = Some('B');
	table[0x31] = Some('N');
	table[0x32] = Some('M');

	table[0x33] = Some('?');
	table[0x34] = Some('.');
	table[0x35] = Some('/');
	table[0x0C] = Some('°');
	table[0x0D] = Some('+');
	table[0x1A] = Some('~');
	table[0x1B] = Some('£');
	table[0x2B] = Some('|');

	table
};
