#![no_std]
#![no_main]
#![feature(naked_functions)]

mod include;
mod io;

use io::shell;

#[no_mangle]
pub extern "C" fn kernel_main() {
	println!("42");
	shell::new();
}
