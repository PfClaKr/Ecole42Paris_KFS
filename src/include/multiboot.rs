use crate::include::panic::panic;
use crate::kernel_main;
use core::arch::naked_asm;

#[repr(C)]
struct MultibootHeader {
	magic: u32,
	architecture: u32,
	checksum: u32,
	header_addr: u32,
}

#[link_section = ".multiboot"]
#[no_mangle]
static MULTIBOOT: MultibootHeader = MultibootHeader {
	magic: 0xE85250D6,
	architecture: 0x0,
	checksum: (0xE85250D6u32.wrapping_neg()),
	header_addr: 0,
};

#[repr(C)]
#[derive(Debug)]
struct MultibootInfo {
	total_size: u32,
	reserved: u32,
}

#[repr(C)]
struct MultibootTag {
	_type: u32,
	size: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootMemoryMapTag {
	_type: u32,
	pub size: u32,
	pub entry_size: u32,
	entry_version: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct MultibootMemoryMapEntry {
	pub base_addr: u64,
	pub length: u64,
	pub _type: u32,
	reserved: u32,
}

#[link_section = ".stack"]
#[no_mangle]
static mut STACK: [u8; 8192] = [0; 8192];

#[naked]
#[no_mangle]
pub extern "C" fn start() -> ! {
	unsafe {
		naked_asm!(
			// "mov esp, {stack_end}",
			"lea esp, [STACK + 8192]",
			"xor ebp, ebp",

			"push eax",
			"push ecx",
			"push edx",

			"push ebx", // Physical address multiboot2 info
			"push eax", // magic value multiboot2 bootloader, 0x36d76289
			"call {kernel_main}",
			"call {panic}",
			"pop eax",
			"pop eax",

			"pop edx",
			"pop ecx",
			"pop eax",
			kernel_main = sym kernel_main,
			panic = sym panic,
			// stack_end = sym symbols::get_stack_end,
		);
	}
}

pub fn parse_multiboot_info(multiboot_info: usize, multiboot_tag: u32) -> Option<*const u8> {
	let info = unsafe { &*(multiboot_info as *const MultibootInfo) };
	let mut current_addr = multiboot_info + core::mem::size_of::<MultibootInfo>();
	let end_addr = multiboot_info + info.total_size as usize;

	while current_addr < end_addr {
		let tag = unsafe { &*(current_addr as *const MultibootTag) };
		if tag._type == multiboot_tag {
			return Some(current_addr as *const u8);
		}

		current_addr += core::mem::size_of::<MultibootTag>()
			+ (tag.size as usize - core::mem::size_of::<MultibootTag>());
		current_addr = (current_addr + 7) & !7;
	}

	None
}
