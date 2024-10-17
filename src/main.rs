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

fn alloc_test() {
	let mut a = alloc::string::String::new();
	a.push_str("Hello im yugeon");
	println!("{}, size: {}\n", a, a.len());
	let mut b = alloc::string::String::new();
	b.push_str("Hello im yugeon");
	println!("{}, size: {}\n", b, b.len());

	use alloc::vec;
	let _b = vec![[0; 4096 * 200]];
}

#[allow(unused)]
fn init(multiboot_info: usize, paging_status: bool) {
	include::gdt::load();
	memory::physicalmemory::init(multiboot_info);
	memory::virtualmemory::init(multiboot_info, paging_status);
	memory::dynamicmemory::USER_ALLOCATOR.lock().init(
		0x100000,
		0x800B_5000,
		Privilege::User,
		paging_status,
	);
	memory::dynamicmemory::GLOBAL_ALLOCATOR.lock().init(
		0x800B_6000,
		0xBFFE_0000,
		Privilege::Kernel,
		paging_status,
	);
	alloc_test();
}

#[no_mangle]
pub extern "C" fn kernel_main(magic: u32, multiboot_info: usize) {
	assert_eq!(
		magic, 0x36d76289,
		"System have to load by Multiboot2 boot loader."
	);
	init(multiboot_info, true);
	welcome_message();
	Shell::new().run();
}
