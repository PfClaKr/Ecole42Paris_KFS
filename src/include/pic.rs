use crate::include::asm_utile;

#[allow(unused)]
pub fn initialize_pic() {
	// Remap the PICs to avoid conflicts with CPU exception vectors.
	const PIC1_COMMAND: u16 = 0x20;
	const PIC1_DATA: u16 = 0x21;
	const PIC2_COMMAND: u16 = 0xA0;
	const PIC2_DATA: u16 = 0xA1;

	unsafe {
		// ICW1 - Start initialization
		asm_utile::outb(PIC1_COMMAND, 0x11);
		asm_utile::outb(PIC2_COMMAND, 0x11);

		// ICW2 - Remap vector offsets
		asm_utile::outb(PIC1_DATA, 0x20); // Master PIC mapped to 0x20-0x27
		asm_utile::outb(PIC2_DATA, 0x28); // Slave PIC mapped to 0x28-0x2F

		// ICW3 - Set up cascading
		asm_utile::outb(PIC1_DATA, 0x04); // Master PIC has a slave at IRQ2
		asm_utile::outb(PIC2_DATA, 0x02); // Slave PIC cascade identity

		// ICW4 - Set PIC to 80x86 mode
		asm_utile::outb(PIC1_DATA, 0x01);
		asm_utile::outb(PIC2_DATA, 0x01);

		// Mask all IRQs initially
		asm_utile::outb(PIC1_DATA, 0xFF);
		asm_utile::outb(PIC2_DATA, 0xFF);
	}
}
