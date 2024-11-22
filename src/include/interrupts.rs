use crate::io::shell::SHELL;

#[repr(usize)]
#[derive(Debug, PartialEq)]
#[allow(unused)]
pub enum InterruptIndex {
	DivByZero = 0x00,
	SingleStepInt = 0x01,
	Nmi = 0x02,
	Breakpoint = 0x03,
	Overflow = 0x04,
	BoundRangeExceed = 0x05,
	InvOpcode = 0x06,
	CoprocNotAvail = 0x07,
	DoubleFault = 0x08,
	CoprocSegmentOverrun = 0x09,
	InvTSS = 0x0A,
	SegmentNotPresent = 0x0B,
	StackSegmentFault = 0x0C,
	GeneralProtectionFault = 0x0D,
	PageFault = 0x0E,
	Reserved = 0x0F,
	FloatPointException = 0x10,
	AlignmentCheck = 0x11,
	MachineCheck = 0x12,
	SIMDFloatingPointException = 0x13,
	VirtualizationException = 0x14,
	ControlProtectionException = 0x15,
	Timer = PIC_1_OFFSET as usize,
	Keyboard,
}
// source: https://en.wikipedia.org/wiki/Interrupt_descriptor_table

use core::arch::asm;
use spin::Mutex;

use crate::include::asm_utile::{hlt, outb};

use super::pic::{ChainedPics, PIC_1_OFFSET, PIC_2_OFFSET};

macro_rules! create_isr {
	($handler_name: ident, $int_index: expr) => {
		pub extern "C" fn $handler_name(frame: IntStackFrame) {
			crate::println!("\x1b[4;mIDT: {:?}\x1b[15;m", $int_index);
			crate::println!("{:#x?}", frame);

			if ($int_index != InterruptIndex::Breakpoint) {
				hlt()
			}
		}
	};
	($handler_name: ident, $int_index: expr, $bool: expr) => {
		pub extern "C" fn() $handler_name(frame: IntStackFrame, error_code: u32) {
			crate::println!("\x1b[4;mIDT: {:?}\x1b[15;m", $int_index);
			crate::println!("\x1b[4;merror_code: {}\x1b[15;m", error_code);
			crate::println!("{:#x?", frame);
			hlt()
		}
	}
}

macro_rules! create_isr_iretd {
    ($name:ident, $handler_fn:expr) => {
        #[no_mangle]
        pub extern "C" fn $name(_: IntStackFrame) {
            unsafe {
                core::arch::asm!(
                    "push eax",
                    "push ebx",
                    "push ecx",
                    "push edx",
                    "push esi",
                    "push edi",
                    "push ebp",

                    "call {handler}",

                    "pop ebp",
                    "pop edi",
                    "pop esi",
                    "pop edx",
                    "pop ecx",
                    "pop ebx",
                    "pop eax",
                    "iretd",
                    handler = sym $handler_fn,
                    options(noreturn),
                );
            }
        }
    };
}

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

create_isr!(div_by_zero, InterruptIndex::DivByZero);
create_isr!(single_step_int, InterruptIndex::SingleStepInt);
create_isr!(nmi, InterruptIndex::Nmi);
create_isr!(breakpoint, InterruptIndex::Breakpoint);
create_isr!(overflow, InterruptIndex::Overflow);
create_isr!(bound_range_exceed, InterruptIndex::BoundRangeExceed);
create_isr!(inv_opcode, InterruptIndex::InvOpcode);
create_isr!(coproc_not_avail, InterruptIndex::CoprocNotAvail);
create_isr!(double_fault, InterruptIndex::DoubleFault);
create_isr!(coproc_segment_overrun, InterruptIndex::CoprocSegmentOverrun);
create_isr!(inv_tss, InterruptIndex::InvTSS);
create_isr!(segment_not_present, InterruptIndex::SegmentNotPresent);
create_isr!(stack_segment_fault, InterruptIndex::StackSegmentFault);
create_isr!(
	general_protection_fault,
	InterruptIndex::GeneralProtectionFault
);
create_isr!(page_fault, InterruptIndex::PageFault);
// create_isr!(reserved, InterruptIndex::Reserved);
create_isr!(
	floating_point_exception,
	InterruptIndex::FloatPointException
);
create_isr!(alignement_check, InterruptIndex::AlignmentCheck);
create_isr!(machine_check, InterruptIndex::MachineCheck);
create_isr!(
	simd_floating_point_exception,
	InterruptIndex::SIMDFloatingPointException
);
create_isr!(
	virtualization_exception,
	InterruptIndex::VirtualizationException
);
// create_isr!(
// 	control_protection_exception,
// 	InterruptIndex::ControlProtectionException
// );
create_isr_iretd!(timer_interrupt, timer_interrupt_handler);
create_isr_iretd!(keyboard_interrupt, keyboard_interrupt_handler);

pub static PIC: Mutex<ChainedPics> =
	Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

const BASE_FREQUENCY: u32 = 1193182; // Base PIT frequency in Hz.
#[allow(unused)]
const DESIRED_FREQUENCY: u32 = 100; // Desired timer interrupt frequency in Hz.

pub static mut TICKS: usize = 0;

pub unsafe fn configure_pit(frequency: u32) {
	let divisor = BASE_FREQUENCY / frequency;

	outb(0x43, 0x36);
	outb(0x40, (divisor & 0xFF) as u8);
	outb(0x40, (divisor >> 8) as u8);
}

fn timer_interrupt_handler() {
	unsafe {
		TICKS += 1;
		PIC.lock()
			.notify_end_of_interrupt(InterruptIndex::Timer as u8);
	}
}

static mut INPUT: &mut [u8; 77] = &mut [0u8; 77];
static mut LEN: &mut usize = &mut 0;

fn keyboard_interrupt_handler() {
	unsafe {
		SHELL.lock().read_input(INPUT, LEN);
		PIC.lock()
			.notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
	}
}

#[no_mangle]
pub extern "C" fn syscall(frame: IntStackFrame) {
	crate::println!("syscall");
	let eip = frame.eip;
	// crate::println!("error code: 0x{:X}", error_code);
	crate::println!("eip: 0x{:08x}", eip);
	hlt();
}

pub fn is_enabled() -> bool {
	let eflags: u32;
	unsafe {
		asm!(
			"pushfd",
			"pop {0:e}",
			out(reg) eflags
		);
	}
	(eflags & (1 << 9)) != 0
}
