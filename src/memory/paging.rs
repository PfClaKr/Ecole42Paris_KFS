use core::arch::asm;
use crate::include::symbols;
use crate::{
	println,
	print
};
use crate::memory::physicalmemory::{
	BITMAP,
	PhysicalMemoryError,
};

#[derive(Debug)]
pub enum PageFault {
	DemandPaging, // Accessing the page that is not currently loaded in the memory
	InvalidMemoryAccess, // Access that memory which is it’s beyond access boundaries or not allocated
	ProcessViolation, // Write to a read-only page or violates memory protection rules
}

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
const USER_SPACE_LOW: u32 = 0x00000000;
const USER_SPACE_HIGH: u32 = 0xBFFFFFFF;
 
// Next 1GB
const KERNEL_SPACE_LOW: u32 = 0xC0000000;
const KERNEL_SPACE_HIGH: u32 = 0xFFFFFFFF;

type PageDirectoryEntry = u32;
type PageTableEntry = u32;

const ENTRY_COUNT: usize = 1024;
const PAGE_SIZE: usize = 4096; // 4KB

#[repr(C, align(4096))]
pub struct PageDirectory {
	entry: [PageDirectoryEntry; ENTRY_COUNT],
}

#[repr(C, align(4096))]
pub struct PageTable {
	entry: [PageTableEntry; ENTRY_COUNT],
}

impl PageDirectory {
	pub fn new() -> Self {
		PageDirectory {
			entry: [0; ENTRY_COUNT],
		}
	}

	pub fn get_page_table(&mut self, pdi: u32) -> &mut PageTable {
		// do we need to alloc new page if possible?
		//
        // if self.entry[pdi as usize] & 0x1 == 0 {
        //     let new_page_table = BITMAP.lock().alloc_frame().unwrap();
        //     self.entry[pdi as usize] = (new_page_table as u32) | 0x3;
        // }
        let page_table_address = self.entry[pdi as usize] & 0xFFFFF000;
        let page_table_ptr = page_table_address as *mut PageTable;

        unsafe { &mut *page_table_ptr }
    }

	pub fn map_page(&mut self, virtual_address: u32, physical_address: u32) -> Result<(), PhysicalMemoryError> {
		let pdi = (virtual_address >> 22) & 0x3FF; // 0b11111111_11
		let pti = (virtual_address >> 12) & 0x3FF;
		let offset = virtual_address & 0xFFF; // 0b11111111_1111

		// Handled in alloc_frame() function
		// if BITMAP.lock().is_frame_free(physical_address & 0xFFFFF000) {
		// 	Err(PhysicalMemoryError::FrameAlreadyInUse);
		// }
		if self.entry[pdi as usize] == 0 {
			let page_table = BITMAP.lock().alloc_frame()?;
			// BITMAP.lock().alloc_frame_address((physical_address & 0xFFFFF000) as usize).unwrap();
			// println!("0x{:x}", &page_table); // 0x001e0000
			self.entry[pdi as usize] = (&page_table as *const _ as u32) | 0x3; // 0x3 = 0b11
		}
		let page_table = self.get_page_table(pdi);
		BITMAP.lock().alloc_frame_address((physical_address & 0xFFFFF000) as usize | 0x3)?;
		page_table.entry[pti as usize] = (physical_address & 0xFFFFF000) | 0x3;

		// println!("0 0x{:x}", page_table.entry[0] & 0xFFFFF000);
		// println!("1 0x{:x}", page_table.entry[1] & 0xFFFFF000);
		// println!("2 0x{:x}", page_table.entry[2] & 0xFFFFF000);
		// println!("3 0x{:x}", page_table.entry[3] & 0xFFFFF000);
		println!("pti as {} 0x{:x}", pti as usize, page_table.entry[pti as usize] & 0xFFFFF000);

		// if (BITMAP.lock().is_frame_free((physical_address & 0xFFFFF000) as usize) == true) {
		// 	println!("frame is not allocated");
		// } else {
		// 	println!("frame is allocated");
		// }

		// let bitmap = BITMAP.lock().bitmap();
		// for i in 0..(1048576/32) {
		// 	if bitmap[i] != 0 {
		// 		print!("{}: {:x}, ", i, bitmap[i]);
		// 	}
		// }
		Ok(())
	}

	pub fn unmap_page(&mut self, virtual_address: u32) -> Result<(), PhysicalMemoryError> {
		let pdi = (virtual_address >> 22) & 0x3FF;
		let pti = (virtual_address >> 12) & 0x3FF;

		// if self.entry[pdi as usize] & 0x1 == 0 {
		// 	Err(PageFault::InvalidMemoryAccess);
		// }
		let page_table = self.get_page_table(pdi);
		let mut page_table_entry = page_table.entry[pti as usize];
		if page_table_entry & 0x1 != 0 {
			let frame = page_table_entry & 0xFFFFF000;
			BITMAP.lock().free_frame(frame as usize)?;
			// if let Some(physical_address) = self.translate(page_table_entry, page_table) {
			// 	BITMAP.lock().free_frame(physical_address as usize).unwrap();
			// }
			page_table_entry = 0;
			// Invalidate the page in the Translation Lookaside Buffer
			unsafe {
				asm!("invlpg [{}]", in(reg) virtual_address, options(nostack, preserves_flags));
			}
		}
		Ok(())
	}

	pub fn translate(&self, virtual_address: u32, page_table: &PageTable) -> Option<u32> {
		let pdi = (virtual_address >> 22) & 0x3FF;
		let pti = (virtual_address >> 12) & 0x3FF;
		let offset = virtual_address & 0xFFF;

		if self.entry[pdi as usize] & 0x1 == 0 {
			return None;
		}
		if page_table.entry[pti as usize] & 0x1 == 0 {
			return None;
		}
		let frame = page_table.entry[pti as usize] & 0xFFFFF000; // 0b11111111_11111111_11110000_00000000
		return Some(frame + offset);
	}
}

impl PageTable {
	pub fn new() -> Self {
		PageTable {
			entry: [0; ENTRY_COUNT],
		}
	}
}

pub fn init() {
	let mut page_directory = PageDirectory::new();
	let mut page_table = PageTable::new();

	// Identity Mapping first 4MB
	// virtual address == physical address
	// purpose: fast and secure initialization
	// of memory page (below) before kernel load
	// 	- kernel code and data
	// 	- essential hardware regions
	// 	- initial stack and heap
	//
	// Identity Mapped addresses are usually remapped after kernel load

	for i in 0..ENTRY_COUNT {
		// 0x1000 equals to 4096
		page_table.entry[i] = (i * 0x1000) as u32 | 0x3;
	}
	page_directory.entry[0] = (&page_table as *const _ as u32) | 0x3;

	let _: Result<(), PhysicalMemoryError>;
	_ = page_directory.map_page(0x1000 * 1024, 0x1000 * 1024).expect("failed");
	_ = page_directory.map_page(0x1000 * 1025, 0x1000 * 1026).expect("failed");
	_ = page_directory.map_page(0x1000 * 1026, 0x0022f000).expect("failed");
	_ = page_directory.map_page(0x1000 * 1027, 0x00240000).expect("failed");

	_ = page_directory.unmap_page(0x1000 * 1024).expect("failed");
	_ = page_directory.unmap_page(0x1000 * 1028).expect("failed");

	if (BITMAP.lock().is_frame_free(((0x1000 * 1024) as u32 & 0xFFFFF000u32) as usize) == true) {
		println!("frame is not allocated");
	} else {
		println!("frame is allocated");
	}
	if (BITMAP.lock().is_frame_free(((0x1000 * 1026) as u32 & 0xFFFFF000u32) as usize) == true) {
		println!("frame is not allocated");
	} else {
		println!("frame is allocated");
	}
	if (BITMAP.lock().is_frame_free((0x0022f000u32 & 0xFFFFF000u32) as usize) == true) {
		println!("frame is not allocated");
	} else {
		println!("frame is allocated");
	}
	if (BITMAP.lock().is_frame_free((0x00240000u32 & 0xFFFFF000u32) as usize) == true) {
		println!("frame is not allocated");
	} else {
		println!("frame is allocated");
	}

	// Map User Space and Kernel Space pages
	// 768 / 256
	//
	// TODO
	
	// map VGA buffer (at 0xB8000) usually the size of the VGA_BUFFER is 4KB
	// but can be up to 8KB depending on the display mode
	
	// let vga_index = VGA_BUFFER_ADDRESS / 0x1000;
	// page_table.entry[vga_index] = VGA_BUFFER_ADDRESS as u32 | 0x1 | 0x01;

	unsafe {
		load_page_directory(&page_directory);
		enable_paging();
	}
}

unsafe fn load_page_directory(page_directory: &PageDirectory) {
	let page_directory_address = page_directory as *const _ as u32;
	asm!(
		"mov cr3, {}",
		in(reg) page_directory_address,
		options(nostack)
	);
}

unsafe fn enable_paging() {
	let mut cr0: u32;
	asm!("mov {}, cr0", out(reg) cr0);
	cr0 |= 0x80000000; // 0b10000000_00000000_00000000_00000000
	asm!("mov cr0, {}", in(reg) cr0);
}
