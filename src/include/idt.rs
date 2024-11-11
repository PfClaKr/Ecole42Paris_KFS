#[allow(unused_imports)]
use crate::include::interrupts::{
	ALIGNMENT_CHECK, BOUND_RANGE_EXCEED, BREAKPOINT, CONTROL_PROTECTION_EXCEPTION,
	COPROC_NOT_AVAIL, COPROC_SEGMENT_OVERRUN, DEFAULT, DIV_BY_ZERO, DOUBLE_FAULT,
	FLOATING_POINT_EXCEPTION, GENERAL_PROTECTION_FAULT, INV_OPCODE, INV_TSS, MACHINE_CHECK, NMI,
	OVERFLOW, PAGE_FAULT, RESERVED, SEGMENT_NOT_PRESENT, SINGLE_STEP_INT, STACK_SEGMENT_FAULT,
	SYSCALL, TIMER_INTERRUPT, VIRTUALIZATION_EXCEPTION,
};

use core::arch::asm;

const ENTRY_COUNT: usize = 256;

#[repr(C, packed)]
struct Idt {
	entries: [IdtEntry; ENTRY_COUNT],
}

#[allow(unused)]
impl Idt {
	fn new() -> Idt {
		Idt {
			entries: [IdtEntry::new(0, 0, 0); ENTRY_COUNT],
		}
	}

	fn set_entry(&mut self, index: usize, isr: usize, kernel_cs: u16, attributes: u8) {
		self.entries[index] = IdtEntry::new(isr, kernel_cs, attributes);
	}
}

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
	const fn new(isr: usize, kernel_cs: u16, attr: u8) -> IdtEntry {
		IdtEntry {
			isr_low: (isr & 0x0000FFFF) as u16,
			kernel_cs,
			reserved: 0,
			attributes: attr,
			isr_high: ((isr & 0xFFFF0000) >> 16) as u16,
		}
	}
}

#[allow(unused)]
#[repr(C, packed)]
struct IdtPtr {
	limit: u16,
	base: usize,
}

#[link_section = ".idt"]
static IDT_BASE: [IdtEntry; ENTRY_COUNT] = [IdtEntry::new(0, 0, 0); ENTRY_COUNT];

static mut IDT: *mut [IdtEntry; ENTRY_COUNT] = core::ptr::null_mut();

unsafe fn set_idt() {
	IDT = (&IDT_BASE as *const _ as usize) as *mut [IdtEntry; ENTRY_COUNT];
	let idt: &mut [IdtEntry; ENTRY_COUNT] = &mut *IDT;

	crate::println!("set_idt: IDT_BASE 0x{:08x}", &IDT_BASE as *const _ as usize);

	idt[0x00] = IdtEntry::new(DIV_BY_ZERO as usize, 0x08, 0x8E);
	idt[0x01] = IdtEntry::new(SINGLE_STEP_INT as usize, 0x08, 0x8E);
	idt[0x02] = IdtEntry::new(NMI as usize, 0x08, 0x8E);
	idt[0x03] = IdtEntry::new(BREAKPOINT as usize, 0x08, 0x8E);
	idt[0x04] = IdtEntry::new(OVERFLOW as usize, 0x08, 0x8E);
	idt[0x05] = IdtEntry::new(BOUND_RANGE_EXCEED as usize, 0x08, 0x8E);
	idt[0x06] = IdtEntry::new(INV_OPCODE as usize, 0x08, 0x8E);
	idt[0x07] = IdtEntry::new(COPROC_NOT_AVAIL as usize, 0x08, 0x8E);
	idt[0x08] = IdtEntry::new(DOUBLE_FAULT as usize, 0x08, 0x8E);
	idt[0x09] = IdtEntry::new(COPROC_SEGMENT_OVERRUN as usize, 0x08, 0x8E);
	idt[0x0A] = IdtEntry::new(INV_TSS as usize, 0x08, 0x8E);
	idt[0x0B] = IdtEntry::new(SEGMENT_NOT_PRESENT as usize, 0x08, 0x8E);
	idt[0x0C] = IdtEntry::new(STACK_SEGMENT_FAULT as usize, 0x08, 0x8E);
	idt[0x0D] = IdtEntry::new(GENERAL_PROTECTION_FAULT as usize, 0x08, 0x8E);
	idt[0x0E] = IdtEntry::new(PAGE_FAULT as usize, 0x08, 0x8E);
	idt[0x0F] = IdtEntry::new(RESERVED as usize, 0x08, 0x8E);
	idt[0x10] = IdtEntry::new(FLOATING_POINT_EXCEPTION as usize, 0x08, 0x8E);
	idt[0x11] = IdtEntry::new(ALIGNMENT_CHECK as usize, 0x08, 0x8E);
	idt[0x12] = IdtEntry::new(MACHINE_CHECK as usize, 0x08, 0x8E);
	idt[0x13] = IdtEntry::new(VIRTUALIZATION_EXCEPTION as usize, 0x08, 0x8E);
	idt[0x14] = IdtEntry::new(CONTROL_PROTECTION_EXCEPTION as usize, 0x08, 0x8E);
	idt[0x20] = IdtEntry::new(TIMER_INTERRUPT as usize, 0x08, 0x8E);
	idt[0x80] = IdtEntry::new(SYSCALL as usize, 0x08, 0xEE);
}

pub unsafe fn load() {
	set_idt();
	let idtr = IdtPtr {
		limit: (core::mem::size_of::<Idt>()) as u16,
		base: &IDT_BASE as *const _ as usize,
	};
	asm!("lidt [{}]", in(reg) &idtr as *const IdtPtr, options(nostack, preserves_flags, readonly));
	crate::println!(
		"IDT_BASE loaded at 0x{:08x}",
		&IDT_BASE as *const _ as usize
	);

	{
		let mut idtr2 = IdtPtr { limit: 0, base: 0 };
		asm!("sidt [{}]", in(reg) &mut idtr2 as *mut IdtPtr);
		let base = idtr2.base;
		let limit = idtr2.limit;
		crate::println!("IDT base: 0x{:08x}, limit: {}", base, limit);
	}

	// Turn off to run without IDT to avoid crash
	// asm!("sti");
}
