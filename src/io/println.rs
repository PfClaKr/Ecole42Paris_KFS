// macros to access from other modules with print!/println!

use crate::io::vga_buffer;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::io::println::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

use core::fmt;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
	use core::fmt::Write;
	vga_buffer::WRITER.lock().write_fmt(args).unwrap();
}
