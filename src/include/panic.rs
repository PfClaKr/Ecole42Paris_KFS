use crate::include::asm_utile;
use crate::include::symbols;
use crate::io::vga_buffer::WRITER;
use crate::println;
use core::arch::asm;
use core::panic::PanicInfo;

pub fn clean_regs_save_stack() {
	unsafe {
		asm!(
			"xor eax, eax",
			"xor ebx, ebx",
			"xor ecx, ecx",
			"xor edx, edx",
			"xor esi, esi",
			"xor edi, edi",
			"xor ebp, ebp",
			"mov esp, {x}",
			x = in(reg) symbols::get_stack_top
		)
	}
}

pub fn clean_regs() {
	unsafe {
		asm!(
			"xor eax, eax",
			"xor ebx, ebx",
			"xor ecx, ecx",
			"xor edx, edx",
			"xor esi, esi",
			"xor edi, edi",
			"xor ebp, ebp",
		)
	}
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
	for _i in 0..25 {
		WRITER.lock().clear_row(_i);
	}
	println!(
		"\n\x1b[4;mPanicked :\n\x1b[15;mlocation: {}\nreason: {}",
		_info.location().unwrap(),
		_info.message()
	);
	unsafe { asm!("cli") };
	println!("Clear register ...");
	println!("Save current stack ...");
	clean_regs_save_stack();
	loop {
		asm_utile::hlt();
	}
}
