use crate::println;
use core::panic::PanicInfo;

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
	println!(
		"Panicked :\nlocation: {}\nreason: {}",
		_info.location().unwrap(),
		_info.message()
	);
	loop {}
}
