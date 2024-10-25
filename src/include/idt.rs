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
	const fn new(isr: u32, kernel_cs: u16, attr: u8) -> IdtEntry {
		IdtEntry {
			isr_low: (isr & 0x0000FFFF) as u16,
			kernel_cs,
			reserved: 0,
			attributes: attr,
			isr_high: ((isr >> 16) & 0xFFFF0000) as u16,
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

// static mut IDT_PTR: *mut Idt = 0x0004000 as *mut Idt;
static mut IDT: Idt = Idt { entries: [IdtEntry::new(0, 0, 0); 256] };

#[allow(unused)]
#[repr(C, packed)]
struct IdtPtr {
	limit: u16,
	base: u32,
}

#[macro_export]
macro_rules! handler {
	($isr: ident) => {{
		#[naked]
		extern "C" fn wrapper() {
			unsafe {
				naked_asm!(
					// disable interrupt
					"cli",
					// Set up stack frame
					"push ebp",
					"mov ebp, esp",
					// Save all general-purpose registers
					"pushad",
					// Calculate the correct stack frame pointer
					"mov eax, esp",
					// Call the actual interrupt handler
					"call {}",
					// Restore all general-purpose registers
					"popad",
					// Restore base pointer and return from interrupt
					"pop ebp",
					// enable interrupt
					"sti",
					"iretd",
					sym $isr,
				);
			}
		}
		wrapper as extern "C" fn()
	}};
}

// List of interrupts - informative purpose
// pub enum Interrupts {
// 	DivByZero = 0x00,
// 	SingleStepInt = 0x01,
// 	NMI = 0x02,
// 	Breakpoint = 0x03,
// 	Overflow = 0x04,
// 	BoundRangeExceed = 0x05,
// 	InvOpcode = 0x06,
// 	CoprocNotAvail = 0x07,
// 	DoubleFault = 0x08,
// 	CoprocSegmentOverrun = 0x09,
// 	InvTSS = 0x0A,
// 	SegmentNotPresent = 0x0B,
// 	StackSegmentFault = 0x0C,
// 	GeneralProtectionFault = 0x0D,
// 	PageFault = 0x0E,
// 	Reserved = 0x0F,
// 	FloatPointException = 0x10,
// 	AlignemntCheck = 0x11,
// 	MachineCheck = 0x12,
// 	SIMDFloatingPointException = 0x13,
// 	VirtualizationException = 0x14,
// 	ControlProtectionException = 0x15,
// }
// source: https://en.wikipedia.org/wiki/Interrupt_descriptor_table

static DIV_BY_ZERO: extern "C" fn() = handler!(div_by_zero);
// static SINGLE_STEP_INT: extern "C" fn() = handler!();
// static NMI: extern "C" fn() = handler!();
// static BREAKPOINT: extern "C" fn() = handler!();
// static OVERFLOW: extern "C" fn() = handler!();
// static BOUND_RANGE_EXCEED: extern "C" fn() = handler!();
// static INV_OPCODE: extern "C" fn() = handler!();
// static COPROC_NOT_AVAIL: extern "C" fn() = handler!();
// static DOUBLE_FAULT: extern "C" fn() = handler!();
// static COPROC_SEGMENT_OVERRUN: extern "C" fn() = handler!();
// static INV_TSS: extern "C" fn() = handler!();
// static SEGMENT_NOT_PRESENT: extern "C" fn() = handler!();
// static STACK_SEGMENT_FAULT: extern "C" fn() = handler!();
static GENERAL_PROTECTION_FAULT: extern "C" fn() = handler!(general_protection_fault);
static PAGE_FAULT: extern "C" fn() = handler!(page_fault);
// static RESERVED: extern "C" fn() = handler!();
// static FLOATING_POINT_EXCEPTION: extern "C" fn() = handler!();
// static ALIGNMENT_CHECK: extern "C" fn() = handler!();
// static MACHINE_CHECK: extern "C" fn() = handler!();
// static VIRTUALIZATION_EXCEPTION: extern "C" fn() = handler!();
// static CONTROL_PROTECTION_EXCEPTION: extern "C" fn() = handler!();

pub unsafe fn load() {
	IDT.entries[0x00] = IdtEntry::new(DIV_BY_ZERO as u32, 0x08, 0x8E);
	IDT.entries[0x0D] = IdtEntry::new(GENERAL_PROTECTION_FAULT as u32, 0x08, 0x8E);
	IDT.entries[0x0E] = IdtEntry::new(PAGE_FAULT as u32, 0x08, 0x8E);
	
	let idtr = IdtPtr {
		limit: (core::mem::size_of::<Idt>() - 1) as u16,
		base: &raw const IDT as *const _ as u32,
	};
	asm!("lidt [{}]", in(reg) &idtr as *const IdtPtr, options(nostack, preserves_flags, readonly));
	crate::println!("IDT loaded at 0x{:08x}", &raw const IDT as *const _ as u32);
	
	{
		let mut idtr = IdtPtr { limit: 0, base: 0 };
		asm!("sidt [{}]", in(reg) &mut idtr as *mut IdtPtr);
		let base = idtr.base;
		let limit = idtr.limit;
		crate::println!("IDT base: 0x{:08x}, limit: {}", base, limit);
	}
	
	asm!("sti");
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

#[no_mangle]
pub extern "C" fn div_by_zero(frame: &IntStackFrame, error_code: u32) {
	let eip = frame.eip;
    crate::println!("division by zero");
	crate::println!("error code: 0x{:X}", error_code);
    crate::println!("eip: 0x{:08x}", eip);
}

#[no_mangle]
pub extern "C" fn general_protection_fault(frame: &IntStackFrame, error_code: u32) {
	let eip = frame.eip;
	crate::println!("general protection fault");
	crate::println!("error code: 0x{:X}", error_code);
    crate::println!("eip: 0x{:08x}", eip);
}

#[no_mangle]
pub extern "C" fn page_fault(frame: &IntStackFrame, error_code: u32) {
	let eip = frame.eip;
	crate::println!("page fault");
	crate::println!("error code: 0x{:X}", error_code);
    crate::println!("eip: 0x{:08x}", eip);
}

