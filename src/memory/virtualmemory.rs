use crate::{io::println, memory::physicalmemory::{PhysicalMemoryError, BITMAP}};
use core::{arch::asm, ops::Deref};
use spin::Mutex;

#[derive(Debug)]
#[allow(unused)]
pub enum PageFault {
	DemandPaging,        // Accessing the page that is not currently loaded in the memory
	InvalidMemoryAccess, // Access that memory which is it’s beyond access boundaries or not allocated
	ProcessViolation,    // Write to a read-only page or violates memory protection rules
}

// may be used later
// use crate::include::symbols;
//
// const SYM_KERNEL_START: *usize = symbols::get_kernel_start();
// const SYM_KERNEL_END: *usize = symbols::get_kernel_end();
// const SYM_PAGE_START: *usize = symbols::get_first_page();

// First 768 entries (0–767):
// These correspond to the lower 3GB (768 * 4MB = 3GB)
// and are used for user-space addresses.
// This is a common layout in OS design where user applications run.
//
// Last 256 entries (768–1023):
// These correspond to the upper 1GB (256 * 4MB = 1GB)
// and are used for kernel-space addresses.
// This is where the kernel runs and manages its resources,
// like kernel code, stacks, and device memory.

// First 3GB
// #[allow(unused)]
// const USER_SPACE_LOW: usize = 0x00000000;
// const USER_SPACE_HIGH: usize = 0xBFFFFFFF;

// Next 1GB
// #[allow(unused)]
// const KERNEL_SPACE_LOW: usize = 0xC0000000;
// const KERNEL_SPACE_HIGH: usize = 0xFFFFFFFF;

// use super::physicalmemory::PAGE_DIRECTORY_ADDRESS;

type PageDirectoryEntry = usize;
type PageTableEntry = usize;

#[allow(unused)]
const ENTRY_COUNT: usize = 1024;
const PAGE_SIZE: usize = 4096; // 4KB

#[repr(C, align(4096))]
pub struct PageDirectory {
	pub entry: [PageDirectoryEntry; ENTRY_COUNT],
}

#[repr(C, align(4096))]
pub struct PageTable {
	pub entry: [PageTableEntry; ENTRY_COUNT],
}

#[allow(unused)]
impl PageDirectory {
	pub fn new() -> Self {
		PageDirectory {
			entry: [0; ENTRY_COUNT],
		}
	}

	pub fn get_page_table(&self, address: usize) -> &mut PageTable {
		let ptr = (0xFFC00000 + (address * 0x1000)) as *mut PageTable;
		unsafe { &mut *ptr }
	}

	pub fn map_page(
		&mut self,
		virtual_address: usize,
		physical_address: usize,
	) -> Result<(), PhysicalMemoryError> {
		let pdi = (virtual_address >> 22) & 0x3FF;
		let pti = (virtual_address >> 12) & 0x3FF;
		
		if self.entry[pdi as usize] == 0 {
			self.entry[pdi as usize] = BITMAP.lock().alloc_frame().unwrap() as usize;
		}
		let page_table = self.get_page_table(pti);
		page_table.entry[pti] = (physical_address & 0xFFFFF000) | 0x3;
		Ok(())
	}

	pub fn unmap_page(&mut self, virtual_address: usize) -> Result<(), PhysicalMemoryError> {
		let pdi = (virtual_address >> 22) & 0x3FF;
		let pti = (virtual_address >> 12) & 0x3FF;

		let page_table = self.get_page_table(pdi);
		let mut page_table_entry = page_table.entry[pti];
		if page_table_entry & 0x1 != 0 {
			let frame = page_table_entry & 0xFFFFF000;
			BITMAP.lock().free_frame(frame)?;
			page_table_entry = 0;
			// Invalidate the page in the Translation Lookaside Buffer
			unsafe {
				asm!("invlpg [{0:e}]", in(reg) virtual_address, options(nostack, preserves_flags));
			}
		}
		Ok(())
	}

	pub fn translate(&self, virtual_address: usize, page_table: &PageTable) -> Option<usize> {
		let pdi = (virtual_address >> 22) & 0x3FF;
		let pti = (virtual_address >> 12) & 0x3FF;
		let offset = virtual_address & 0xFFF;

		if self.entry[pdi] & 0x1 == 0 {
			return None;
		}
		if page_table.entry[pti] & 0x1 == 0 {
			return None;
		}
		let frame = page_table.entry[pti] & 0xFFFFF000; // 0b11111111_11111111_11110000_00000000
		Some(frame + offset)
	}

	pub unsafe fn enable_recursive(&mut self) {
		self.entry[1023] = pda;
	}

	unsafe fn load_page_directory(&mut self) {
		let page_directory_address = pda;
		asm!(
			"mov cr3, {0:e}",
			in(reg) page_directory_address,
			options(nostack)
		);
	}
	
	unsafe fn enable_paging(&self) {
		let mut cr0: usize;
		asm!("mov {0:e}, cr0", out(reg) cr0);
		cr0 |= 0x80000000; // 0b10000000_00000000_00000000_00000000
		asm!("mov cr0, {0:e}", in(reg) cr0);
	}
}

#[allow(unused)]
impl PageTable {
	pub fn new() -> Self {
		PageTable {
			entry: [0; ENTRY_COUNT],
		}
	}
}

pub static PAGE_DIRECTORY: Mutex<PageDirectory> = Mutex::new(PageDirectory {
	entry: [0; ENTRY_COUNT],
});

pub static mut pda: usize = 0;

use crate::println; // remove

pub fn init() {
	// Identity Mapping first 4MB
	// virtual address == physical address
	// purpose: fast and secure initialization
	// of memory page (below) before kernel load
	// 	- kernel code and data
	// 	- essential hardware regions
	// 	- initial stack and heap
	//
	// Identity Mapped addresses are usually remapped after kernel load

	// let mut page_table = PageTable::new();
	// for i in 0..ENTRY_COUNT {
	// 	page_table.entry[i] = (i * 0x1000) as usize | 0x3;
	// }
	// PAGE_DIRECTORY.lock().entry[0] = (&page_table as *const _ as usize) | 0x3;
	
	let mut pd_guard = PAGE_DIRECTORY.lock();
	
	unsafe {
		pd_guard.enable_recursive();
		pda = &*pd_guard as *const _ as usize;
		for i in 0..ENTRY_COUNT {
			pd_guard.map_page(i * 0x1000, i * 0x1000).unwrap();
		}
	}

	unsafe {
		// println!("PAGE_DIRECTORY: {:X}", &PAGE_DIRECTORY as *const _ as usize);
		// println!("PAGE_DIRECTORY: {:X}", &PAGE_DIRECTORY.lock() as *const _ as usize);
		// println!("PAGE_DIRECTORY: {:X}", &PAGE_DIRECTORY.lock().entry as *const _ as usize);
		// println!("PAGE_DIRECTORY: {:X}", &PAGE_DIRECTORY.lock().entry[0] as *const _ as usize);
		// println!("PAGE_DIRECTORY: {:X}", &PAGE_DIRECTORY.lock().entry[0]);
		// println!("...");
		// println!("PAGE_DIRECTORY VALUE at [0]: {:X}", PAGE_DIRECTORY.lock().entry[0]);
		// println!("PAGE_DIRECTORY VALUE at [0]: {:X}", &PAGE_DIRECTORY.lock().entry[0]);		
		// println!("PAGE_DIRECTORY VALUE at [0]: {:X}", &PAGE_DIRECTORY.lock().entry[0] as *const _ as usize);		
		// println!("...");
		// println!("PAGE_DIRECTORY VALUE at [1]: {:X}", PAGE_DIRECTORY.lock().entry[1]);
		// println!("PAGE_DIRECTORY VALUE at [1]: {:X}", &PAGE_DIRECTORY.lock().entry[1]);		
		// println!("PAGE_DIRECTORY VALUE at [1]: {:X}", &PAGE_DIRECTORY.lock().entry[1] as *const _ as usize);		
		// println!("...");
		// println!("PAGE_DIRECTORY VALUE at [1022]: {:X}", PAGE_DIRECTORY.lock().entry[1022]);
		// println!("PAGE_DIRECTORY VALUE at [1022]: {:X}", &PAGE_DIRECTORY.lock().entry[1022]);
		// println!("PAGE_DIRECTORY VALUE at [1022]: {:X}", &PAGE_DIRECTORY.lock().entry[1022] as *const _ as usize);
		// println!("...");
		// println!("PAGE_DIRECTORY VALUE at [1023]: {:X}", PAGE_DIRECTORY.lock().entry[1023]);
		// println!("PAGE_DIRECTORY VALUE at [1023]: {:X}", &PAGE_DIRECTORY.lock().entry[1023]);
		// println!("PAGE_DIRECTORY VALUE at [1023]: {:X}", &PAGE_DIRECTORY.lock().entry[1023] as *const _ as usize);
		// println!("...");
		// println!("PAGE_DIRECTORY_ADDRESS: {:X}", PAGE_DIRECTORY_ADDRESS);
	}

	// pd_guard.map_page(0x0, 0x0).unwrap();
	// pd_guard.map_page(0xb8000, 0xb8000).unwrap();
	// pd_guard.map_page(0x400000, 0x400000).unwrap();

	// for i in 0..ENTRY_COUNT {
	// 	PAGE_DIRECTORY
	// 		.lock()
	// 		.map_page(i, i)
	// 		.unwrap();
	// }

	// Map User Space and Kernel Space pages
	// 768 / 256

	// for entry in PAGE_DIRECTORY.lock().entry.iter_mut().enumerate() {
	// 	if entry.0 < 256 {
	// 		// kernel space
	// 	} else {
	// 		// user space
	// 	}
	// }

	// map VGA buffer (at 0xB8000) usually the size of the VGA_BUFFER is 4KB
	// but can be up to 8KB depending on the display mode

	// let vga_index = VGA_BUFFER_ADDRESS / 0x1000;
	// page_table.entry[vga_index] = VGA_BUFFER_ADDRESS | 0x1 | 0x01;

	unsafe {
		pd_guard.load_page_directory();
		pd_guard.enable_paging();
	}

	println!("test");
	loop {
		
	}
}

