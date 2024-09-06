use crate::kernel_main;
use crate::println;
use core::arch::asm;
use core::panic::PanicInfo;

#[repr(C)]
pub struct Multiboot {
	magic: u32,
	flags: u32,
	checksum: u32,
	header_addr: u32,
}

#[link_section = ".multiboot"]
#[no_mangle]
pub static MULTIBOOT: Multiboot = Multiboot {
	magic: 0xE85250D6,
	flags: 0x0,
	checksum: (0xE85250D6u32.wrapping_neg()),
	header_addr: 0,
};

#[naked]
#[no_mangle]
pub extern "C" fn start() -> ! {
	unsafe {
		asm!(
			"cli",                 // 인터럽트 비활성화
			"call {kernel_main}",  // Rust 커널의 진입점 호출
			kernel_main = sym kernel_main,
			options(noreturn)
		);
	}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	println!("{}", _info);
	loop {}
}
