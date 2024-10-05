extern "C" {
	pub fn kernel_start();
	pub fn kernel_end();
}

const unsafe fn get_symbols(f: unsafe extern "C" fn()) -> *const usize {
	f as *const usize
}

pub fn get_kernel_start() -> *const usize {
	unsafe { get_symbols(kernel_start) }
}

pub fn get_kernel_end() -> *const usize {
	unsafe { get_symbols(kernel_end) }
}
