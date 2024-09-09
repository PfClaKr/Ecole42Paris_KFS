#![no_std]
#![no_main]
#![feature(naked_functions)]

mod include;
mod io;

use io::shell::Shell;

fn welcome_message() {
	println!("   ___  _____     ");
	println!("  /   |/ __  \\   ");
	println!(" / /| |`' / /' ");
	println!("/ /_| |  / /   ");
	println!("\\___  |./ /___ ");
	println!("    |_/\\_____/    ");

	println!("Knife Fork Spoon");
	println!("KFS 42 - ychun, schaehun");
}

#[no_mangle]
pub extern "C" fn kernel_main() {
	welcome_message();
	Shell::new().run();
}
