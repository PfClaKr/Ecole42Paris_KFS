#![no_std]
#![no_main]
#![feature(naked_functions)]

mod include;
mod io;
mod memory;

use io::shell::Shell;

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

#[no_mangle]
pub extern "C" fn kernel_main() {
	// let mut page_directory = memory::paging::PageDirectory::new();
	// let mut page_table = memory::paging::PageTable::new();
	unsafe {
		include::gdt::load();
		memory::paging::init_page();
		// page_directory.map_page(0xCAFEBABE, 0x12345000, &mut page_table);
		// if let Some(physical_address) = page_directory.translate(0xCAFEBABE, &page_table) {
		// 	println!("0xCAFEBABE maps to physical address: 0x{:X}", physical_address);
		// } else {
		// 	println!("Page not present");
		// }
	}
	welcome_message();
	Shell::new().run();
}
