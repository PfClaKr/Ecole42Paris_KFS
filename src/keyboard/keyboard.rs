use crate::include::asm_utile;
use crate::keyboard::keymap;
use spin::Mutex;

const KEYBOARD_STATUS_PORT: u16 = 0x64;
const KEYBOARD_DATA_PORT: u16 = 0x60;
const SHIFT_LEFT: u8 = 0x2A;
const SHIFT_RIGHT: u8 = 0x36;
const SHIFT_LEFT_RELEASE: u8 = 0x2A + 0x80;
const SHIFT_RIGHT_RELEASE: u8 = 0x36 + 0x80;

static SHIFT_PRESSED: Mutex<bool> = Mutex::new(false);

pub fn init() {
	unsafe {
		asm_utile::outb(KEYBOARD_STATUS_PORT, 0xAE);

		while asm_utile::inb(KEYBOARD_STATUS_PORT) & 0x01 != 0 {
			let _ = asm_utile::inb(KEYBOARD_DATA_PORT);
		}

		asm_utile::outb(KEYBOARD_DATA_PORT, 0xF4);
	}
}

pub fn read() -> Option<char> {
	let scancode: u8;
	let key: Option<char>;
	let mut shift: spin::MutexGuard<'_, bool> = SHIFT_PRESSED.lock();

	unsafe {
		scancode = asm_utile::inb(KEYBOARD_DATA_PORT);
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
		_ => {
			if scancode & 0x80 == 0 {
				key = if *shift {
					keymap::TO_SHIFT_ASCII[scancode as usize]
				} else {
					keymap::TO_ASCII[scancode as usize]
				};
				return key;
			}
		}
	}
	None
}
