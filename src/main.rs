#![no_std]
#![no_main]
#![feature(naked_functions)]

mod include;
mod io;

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
	welcome_message();
	Shell::new().run();
}
