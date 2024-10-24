use core::{arch::{asm, naked_asm}, ptr};

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
	isr_low: u16,
	kernel_cs: u16,
	reserved: u8,
	attributes: u8,
	isr_high: u16,
}

#[allow(unused)]
impl IdtEntry {
	fn new(isr: u32, kernel_cs: u16, attr: u8) -> IdtEntry {
		IdtEntry {
			isr_low: (isr & 0xFFFF) as u16,
			kernel_cs,
			reserved: 0,
			attributes: attr,
			isr_high: ((isr >> 16) & 0xFFFF) as u16,
		}
	}
}

#[repr(C, packed)]
struct Idt {
	entries: [IdtEntry; 256],
}

#[allow(unused)]
impl Idt {
	fn new() -> Idt {
		Idt {
			entries: [IdtEntry::new(0, 0, 0); 256],
		}
	}

	fn set_entry(&mut self, index: usize, isr: u32, kernel_cs: u16, attributes: u8) {
		self.entries[index] = IdtEntry::new(isr, kernel_cs, attributes);
	}
}

static mut IDT_PTR: *mut Idt = 0x0000400 as *mut Idt; // up to 4KB

#[allow(unused)]
#[repr(C, packed)]
struct IdtPtr {
	limit: u16,
	base: u32,
}

pub unsafe fn load() {
	let mut idt = Idt::new();
	idt.set_entry(0, isr0_handler as u32, 0x08, 0x8E);
	// idt.set_entry(1, isr0_handler as u32, 0x08, 0x8E);
	ptr::write_volatile(IDT_PTR, idt);
	
	let idtr = IdtPtr {
		limit: (core::mem::size_of::<Idt>() - 1) as u16,
		base: IDT_PTR as u32,
	};
	asm!("lidt [{}]", in(reg) &idtr as *const IdtPtr, options(nostack, preserves_flags, readonly));
	crate::println!("IDT loaded at 0x{:08x}", IDT_PTR as u32);
	
	{
		let mut idtr = IdtPtr { limit: 0, base: 0 };
		asm!("sidt [{}]", in(reg) &mut idtr as *mut IdtPtr);
		let base = idtr.base;
		let limit = idtr.limit;
		crate::println!("IDT base: 0x{:08x}, limit: {}", base, limit);
	}
	// asm!("cli");
	asm!("sti");
}

extern "C" fn test() {
	crate::println!("test function called");
	loop {}
}

#[naked]
extern "C" fn isr0_handler() {
    unsafe {
        naked_asm!(
			// "cli",

            // Set up stack frame
			"push ebp",
			"mov ebp, esp",

			// Save all general-purpose registers
			"pushad",

			// Calculate the correct stack frame pointer
			"mov eax, esp",

			// maybe unneccessary due CPU does it automatically
			// "add eax, 36", // Adjust for 'pushad' and possibly other pushed registers
			// "push eax", // Push stack frame pointer

			// Call the actual interrupt handler
			"call {}",

			// Restore all general-purpose registers
			// "pop eax", // Clean up the stack
			"popad",

			// Restore base pointer and return from interrupt
			"pop ebp",
			// "sti",
			"iretd",
			sym div_by_zero,
        );
    }
}

// Given by CPU
#[derive(Debug)]
#[allow(unused)]
#[repr(C, packed)]
pub struct IntStackFrame {
	pub eip: u32,
	pub cs: u32,
	pub eflags: u32,
	pub esp: u32,
	pub ss: u32,
}

pub extern "C" fn div_by_zero(frame: &IntStackFrame) {
	let eip = frame.eip;
    crate::println!("Division by zero at EIP: 0x{:08x}", eip);
	loop {}
}

