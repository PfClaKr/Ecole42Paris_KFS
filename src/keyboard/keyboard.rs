use crate::include::asm_utile;
use core::ptr::read_volatile;

const KEYBOARD_STATUS_PORT: u16 = 0x64;
const KEYBOARD_DATA_PORT: u16 = 0x60;

pub fn init() {
	unsafe {
		// while asm_utile::inb(KEYBOARD_STATUS_PORT) & 0x02 != 0 {}

		asm_utile::outb(KEYBOARD_STATUS_PORT, 0xAE);

		while asm_utile::inb(KEYBOARD_STATUS_PORT) & 0x01 != 0 {
			let _ = asm_utile::inb(KEYBOARD_DATA_PORT);
		}

		asm_utile::outb(KEYBOARD_DATA_PORT, 0xF4);

		// while asm_utile::inb(KEYBOARD_STATUS_PORT) & 0x02 != 0 {}
	}
}

pub fn read() -> Option<char> {
	let scancode: u8;

	unsafe {
		scancode = read_volatile(KEYBOARD_DATA_PORT as *const u8);
	}
	scancode_to_ascii(scancode)
}

pub fn scancode_to_ascii(scancode: u8) -> Option<char> {
	SCANCODE_TO_ASCII[scancode as usize]
}

static SCANCODE_TO_ASCII: [Option<char>; 256] = {
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

	table
};
