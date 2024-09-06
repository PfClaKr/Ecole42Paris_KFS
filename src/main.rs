#![no_std]
#![no_main]
#![feature(naked_functions)]

mod include;
mod keyboard;
mod shell;

#[no_mangle]
pub extern "C" fn kernel_main() -> () {
	shell::new();
}
