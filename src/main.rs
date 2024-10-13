#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate alloc;

mod include;
mod io;
mod memory;

use io::shell::Shell;
use memory::dynamicmemory::Privilege;

fn welcome_message() {
	println!("\x1b[5;m   ___  _____     ");
	println!("\x1b[9;m  /   |/ __  \\   ");
	println!("\x1b[14;m / /| |`' / /' ");
	println!("\x1b[3;m/ /_| |  / /   ");
	println!("\x1b[13;m\\___  |./ /___ ");
	println!("\x1b[10;m    |_/\\_____/    ");

	println!("\x1b[15;mKnife Fork Spoon");
	println!("KFS 42 - \x1b[9;mychun, \x1b[3;mschaehun\x1b[15;m");
}

#[allow(unused)]
fn init(multiboot_info: usize) {
	unsafe {
		include::gdt::load();
	}
	let memory_map_addr = include::multiboot::parse_multiboot_info(multiboot_info, 6);
	memory::physicalmemory::init(memory_map_addr.unwrap() as usize, multiboot_info);
	memory::virtualmemory::init(multiboot_info);
	memory::dynamicmemory::GLOBAL_ALLOCATOR
		.lock()
		.init(0x40000, 0xFFC00000, Privilege::Kernel);
	memory::dynamicmemory::USER_ALLOCATOR
		.lock()
		.init(0x0, 0x390000, Privilege::User);
	let mut a = alloc::string::String::new();
	a.push_str("Hello im yugeon");
	println!("{}, size: {}\n", a, a.len());
	// 	use alloc::vec;
	// 	let b = vec![[0; 1024 * 1000]];
}

#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, multiboot_info: *const u8) {
	assert_eq!(
		magic, 0x36d76289,
		"System have to load by Multiboot2 boot loader."
	);
	init(multiboot_info as usize);
	welcome_message();
	Shell::new().run();
}
