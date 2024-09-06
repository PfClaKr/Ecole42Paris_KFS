#![no_std]
#![no_main]
#![feature(naked_functions)]

mod include;
mod keyboard;
mod shell;
mod vga_buffer;

#[no_mangle]
pub extern "C" fn kernel_main() {
	println!("42");
	shell::new();
}
