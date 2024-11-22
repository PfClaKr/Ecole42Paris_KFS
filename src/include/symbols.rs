#[allow(unused)]
extern "C" {
	pub fn kernel_start();
	pub fn kernel_end();
	pub fn first_page();
	pub fn stack_top();
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

pub fn get_stack_top() -> *const usize {
	unsafe { get_symbols(stack_top) }
}

#[allow(unused)]
pub fn get_first_page() -> *const usize {
	unsafe { get_symbols(first_page) }
}
