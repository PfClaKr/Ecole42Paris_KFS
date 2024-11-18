// #[allow(unused_imports)]
// use crate::include::interrupts::{
// 	ALIGNMENT_CHECK, BOUND_RANGE_EXCEED, BREAKPOINT, CONTROL_PROTECTION_EXCEPTION,
// 	COPROC_NOT_AVAIL, COPROC_SEGMENT_OVERRUN, DEFAULT, DIV_BY_ZERO, DOUBLE_FAULT,
// 	FLOATING_POINT_EXCEPTION, GENERAL_PROTECTION_FAULT, INV_OPCODE, INV_TSS, MACHINE_CHECK, NMI,
// 	OVERFLOW, PAGE_FAULT, RESERVED, SEGMENT_NOT_PRESENT, SINGLE_STEP_INT, STACK_SEGMENT_FAULT,
// 	SYSCALL, TIMER_INTERRUPT, KEYBOARD_INTERRUPT, VIRTUALIZATION_EXCEPTION, SIMD_FLOATING_POINT_EXCEPTION,
// };

use crate::include::interrupts::{
	alignment_check, bound_range_exceed, breakpoint, control_protection_exception,
	coproc_not_avail, coproc_segment_overrun, default, div_by_zero, double_fault,
	floating_point_exception, general_protection_fault, inv_opcode, inv_tss, machine_check, nmi,
	overflow, page_fault, reserved, segment_not_present, single_step_int, stack_segment_fault,
	syscall, timer_interrupt, keyboard_interrupt, virtualization_exception, simd_floating_point_exception,
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

	crate::println!("set_idt: IDT      0x{:08x}", &idt as *const _ as usize);
	crate::println!("set_idt: IDT_BASE 0x{:08x}", &IDT_BASE as *const _ as usize);

	idt[0x00] = IdtEntry::new(div_by_zero as usize, 0x08, 0x8E);
	idt[0x01] = IdtEntry::new(single_step_int as usize, 0x08, 0x8E);
	idt[0x02] = IdtEntry::new(nmi as usize, 0x08, 0x8E);
	idt[0x03] = IdtEntry::new(breakpoint as usize, 0x08, 0x8E);
	idt[0x04] = IdtEntry::new(overflow as usize, 0x08, 0x8E);
	idt[0x05] = IdtEntry::new(bound_range_exceed as usize, 0x08, 0x8E);
	idt[0x06] = IdtEntry::new(inv_opcode as usize, 0x08, 0x8E);
	idt[0x07] = IdtEntry::new(coproc_not_avail as usize, 0x08, 0x8E);
	idt[0x08] = IdtEntry::new(double_fault as usize, 0x08, 0x8E);
	idt[0x09] = IdtEntry::new(coproc_segment_overrun as usize, 0x08, 0x8E);
	idt[0x0A] = IdtEntry::new(inv_tss as usize, 0x08, 0x8E);
	idt[0x0B] = IdtEntry::new(segment_not_present as usize, 0x08, 0x8E);
	idt[0x0C] = IdtEntry::new(stack_segment_fault as usize, 0x08, 0x8E);
	idt[0x0D] = IdtEntry::new(general_protection_fault as usize, 0x08, 0x8E);
	idt[0x0E] = IdtEntry::new(page_fault as usize, 0x08, 0x8E);
	// idt[0x0F] = IdtEntry::new(reserved as usize, 0x08, 0x8E);
	idt[0x10] = IdtEntry::new(floating_point_exception as usize, 0x08, 0x8E);
	idt[0x11] = IdtEntry::new(alignment_check as usize, 0x08, 0x8E);
	idt[0x12] = IdtEntry::new(machine_check as usize, 0x08, 0x8E);
	idt[0x13] = IdtEntry::new(simd_floating_point_exception as usize, 0x08, 0x8E);
	idt[0x14] = IdtEntry::new(virtualization_exception as usize, 0x08, 0x8E);
	// idt[0x15] = IdtEntry::new(control_protection_exception as usize, 0x08, 0x8E); // (only available with CET)
	// 0x16 ~ 0x1F : Reserved
	idt[0x20] = IdtEntry::new(timer_interrupt as usize, 0x08, 0x8E);
	idt[0x21] = IdtEntry::new(keyboard_interrupt as usize, 0x08, 0x8E);
	// 0x20 ~ 0x27 : Hardware IRQs 0-7
	// 0x70 ~ 0x77 : Hardware IRQs 8-15
	idt[0x80] = IdtEntry::new(syscall as usize, 0x08, 0xEE);
	// 0x81 ~ 0xFF : User-Defined Interrupts
}

pub unsafe fn load() {
	set_idt();
	let idtr = IdtPtr {
		limit: (core::mem::size_of::<IdtEntry>() * ENTRY_COUNT - 1) as u16,
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
	asm!("sti");
	// asm!("cli");
}
