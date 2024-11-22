use crate::include::asm_utile;
use crate::include::symbols;
use crate::println;
use core::arch::asm;
use core::panic::PanicInfo;

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
			"mov esp, {x}",
			x = in(reg) symbols::get_stack_top
		)
	}
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
	println!(
		"Panicked :\nlocation: {}\nreason: {}",
		_info.location().unwrap(),
		_info.message()
	);
	println!("Clear register ...");
	clean_regs();
	loop {
		asm_utile::hlt();
	}
}
