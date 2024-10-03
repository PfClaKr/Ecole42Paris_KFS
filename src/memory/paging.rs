use core::arch::asm;
use crate::println;

const ENTRY_COUNT: usize = 1024;
const PAGE_SIZE: usize = 4096; // 4kb

type PageDirectoryEntry = u32;
type PageTableEntry = u32;

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

	pub fn map_page(&mut self, virtual_address: u32, physical_address: u32, page_table: &mut PageTable) {
		// 10 page directory index | 10 page table index | 12 offset
		let pdi = (virtual_address >> 22) & 0x3FF; // 0b11111111_11
		let pti = (virtual_address >> 12) & 0x3FF;
		let offset = virtual_address & 0xFFF; // 0b11111111_1111

		if self.entry[pdi as usize] == 0 {
			self.entry[pdi as usize] = page_table as *const _ as u32 | 0x3; // 0x3 = 0b11
		}
		page_table.entry[pti as usize] = physical_address | 0x3;
	}

	pub fn unmap_page() {
		//
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

pub fn init_page() {
	let mut page_directory = PageDirectory::new();
	let mut page_table = PageTable::new();

	for i in 0..ENTRY_COUNT {
		// Page Directory
		let mut page_table = PageTable::new();
		page_directory.entry[i] = &page_table as *const _ as u32 | 0x1 | 0x01;
		for j in 0..ENTRY_COUNT {
			// Page Table
			let physical_address = j * PAGE_SIZE;
			page_table.entry[j] = physical_address as u32 | 0x1 | 0x01;
		}
	}
	page_directory.entry[0] = &page_table as *const _ as u32 | 0x1 | 0x01;
	println!("page_directory: {}", page_directory.entry[0]);
	println!("page_table: {}", page_table.entry[42]);

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
