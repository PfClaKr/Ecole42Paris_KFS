use crate::include::asm_utile::hlt;
use core::arch::naked_asm;

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

#[macro_export]
macro_rules! handler {
	($isr: ident) => {{
		#[naked]
		extern "C" fn wrapper() {
			unsafe {
				naked_asm!(
					"cli",
					"push ebp",
					"mov ebp, esp",
					"pushad",
					"mov eax, esp",
					"push eax",
					"call {}",
					"pop eax",
					"popad",
					"pop ebp",
					"sti",
					"iretd",
					sym $isr,
				);
			}
		}
		wrapper as extern "C" fn()
	}};
}

// Must be defined before enabling IDT
#[warn(unused)]
pub static DIV_BY_ZERO: extern "C" fn() = handler!(div_by_zero);
pub static SINGLE_STEP_INT: extern "C" fn() = handler!(single_step_int);
pub static NMI: extern "C" fn() = handler!(nmi);
pub static BREAKPOINT: extern "C" fn() = handler!(breakpoint);
pub static OVERFLOW: extern "C" fn() = handler!(overflow);
pub static BOUND_RANGE_EXCEED: extern "C" fn() = handler!(bound_range_exceed);
pub static INV_OPCODE: extern "C" fn() = handler!(inv_opcode);
pub static COPROC_NOT_AVAIL: extern "C" fn() = handler!(coproc_not_avail);
pub static DOUBLE_FAULT: extern "C" fn() = handler!(double_fault);
pub static COPROC_SEGMENT_OVERRUN: extern "C" fn() = handler!(coproc_segment_overrun);
pub static INV_TSS: extern "C" fn() = handler!(inv_tss);
pub static SEGMENT_NOT_PRESENT: extern "C" fn() = handler!(segment_not_present);
pub static STACK_SEGMENT_FAULT: extern "C" fn() = handler!(stack_segment_fault);
pub static GENERAL_PROTECTION_FAULT: extern "C" fn() = handler!(general_protection_fault);
pub static PAGE_FAULT: extern "C" fn() = handler!(page_fault);
pub static RESERVED: extern "C" fn() = handler!(reserved);
pub static FLOATING_POINT_EXCEPTION: extern "C" fn() = handler!(floating_point_exception);
pub static ALIGNMENT_CHECK: extern "C" fn() = handler!(alignment_check);
pub static MACHINE_CHECK: extern "C" fn() = handler!(machine_check);
pub static VIRTUALIZATION_EXCEPTION: extern "C" fn() = handler!(virtualization_exception);
pub static CONTROL_PROTECTION_EXCEPTION: extern "C" fn() = handler!(control_protection_exception);
pub static TIMER_INTERRUPT: extern "C" fn() = handler!(timer_interrupt);
pub static SYSCALL: extern "C" fn() = handler!(syscall);

#[allow(unused)]
pub static DEFAULT: extern "C" fn() = handler!(default);

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
pub extern "C" fn div_by_zero() {
	crate::println!("division by zero");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn single_step_int() {
	crate::println!("single_step_int");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn nmi() {
	crate::println!("nmi");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn breakpoint() {
	crate::println!("breakpoint");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn overflow() {
	crate::println!("overflow");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn bound_range_exceed() {
	crate::println!("bound_range_exceed");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn inv_opcode() {
	crate::println!("inv_opcode");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn coproc_not_avail() {
	crate::println!("coproc_not_avail");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn double_fault() {
	crate::println!("double_fault");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn coproc_segment_overrun() {
	crate::println!("coproc_segment_overrun");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn inv_tss() {
	crate::println!("inv_tss");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn segment_not_present() {
	crate::println!("segment_not_present");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn stack_segment_fault() {
	crate::println!("stack_segment_fault");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn general_protection_fault() {
	crate::println!("general protection fault");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn page_fault(frame: &IntStackFrame) {
	crate::println!("page fault");
	let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn reserved() {
	crate::println!("reserved");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn floating_point_exception() {
	crate::println!("floating_point_exception");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn alignment_check() {
	crate::println!("alignment_check");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn machine_check() {
	crate::println!("machine_check");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn virtualization_exception() {
	crate::println!("virtualization_exception");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn control_protection_exception() {
	crate::println!("control_protection_exception");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn timer_interrupt() {
	crate::println!("timer_interrupt");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn syscall() {
	crate::println!("syscall");
	// let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	// crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

#[no_mangle]
pub extern "C" fn default() {
	crate::println!("default");
	hlt();
}
