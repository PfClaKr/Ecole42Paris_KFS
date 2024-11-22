use crate::include::asm_utile::{inb, outb};
use crate::include::interrupts;

pub const PIC_1_OFFSET: u8 = 0x20;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

const CMD_INIT: u8 = 0x11;
const CMD_END_OF_INTERRUPT: u8 = 0x20;
const MODE_8086: u8 = 0x01;

struct Pic {
	offset: u8,
	command: u8,
	data: u8,
}

impl Pic {
	fn handles_interrupt(&self, interupt_id: u8) -> bool {
		self.offset <= interupt_id && interupt_id < self.offset + 8
	}

	unsafe fn end_of_interrupt(&mut self) {
		outb(self.command as u16, CMD_END_OF_INTERRUPT);
	}

	unsafe fn read_mask(&mut self) -> u8 {
		inb(self.data as u16)
	}

	unsafe fn write_mask(&mut self, mask: u8) {
		outb(self.data as u16, mask)
	}
}

pub struct ChainedPics {
	pics: [Pic; 2],
}

#[allow(unused)]
impl ChainedPics {
	pub const unsafe fn new(offset1: u8, offset2: u8) -> ChainedPics {
		ChainedPics {
			pics: [
				Pic {
					offset: offset1,
					command: 0x20,
					data: 0x21,
				},
				Pic {
					offset: offset2,
					command: 0xA0,
					data: 0xA1,
				},
			],
		}
	}

	pub unsafe fn initialize(&mut self) {
		let mut wait_port: u8 = 0x80;
		let mut wait = || outb(wait_port as u16, 0);

		let saved_masks = self.read_masks();

		outb(self.pics[0].command as u16, CMD_INIT);
		wait();
		outb(self.pics[1].command as u16, CMD_INIT);
		wait();
		outb(self.pics[0].data as u16, self.pics[0].offset);
		wait();
		outb(self.pics[1].data as u16, self.pics[1].offset);
		wait();
		outb(self.pics[0].data as u16, 4);
		wait();
		outb(self.pics[1].data as u16, 2);
		wait();
		outb(self.pics[0].data as u16, MODE_8086);
		wait();
		outb(self.pics[1].data as u16, MODE_8086);
		wait();

		self.write_masks(saved_masks[0], saved_masks[1])
	}

	pub unsafe fn read_masks(&mut self) -> [u8; 2] {
		[self.pics[0].read_mask(), self.pics[1].read_mask()]
	}

	pub unsafe fn write_masks(&mut self, mask1: u8, mask2: u8) {
		self.pics[0].write_mask(mask1);
		self.pics[1].write_mask(mask2);
	}

	pub unsafe fn disable(&mut self) {
		self.write_masks(u8::MAX, u8::MAX)
	}

	pub fn handles_interrupt(&self, interrupt_id: u8) -> bool {
		self.pics.iter().any(|p| p.handles_interrupt(interrupt_id))
	}

	pub unsafe fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
		if self.handles_interrupt(interrupt_id) {
			if self.pics[1].handles_interrupt(interrupt_id) {
				self.pics[1].end_of_interrupt();
			}
			self.pics[0].end_of_interrupt();
		}
	}
}

pub fn load() {
	unsafe {
		interrupts::PIC.lock().initialize();
		interrupts::configure_pit(100);
	}
}
