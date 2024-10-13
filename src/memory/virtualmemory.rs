use crate::include::symbols;
use crate::memory::physicalmemory::{PhysicalMemoryError, BITMAP};
use core::arch::asm;
use core::ptr::NonNull;
use spin::Mutex;

pub const PDA: usize = 0x21000;

pub struct PageTableEntry(usize);

impl PageTableEntry {
	fn new(address: usize, flags: usize) -> PageTableEntry {
		assert_eq!(0, address & 0xFFF);
		PageTableEntry(address | flags)
	}

	pub fn is_present(&self) -> bool {
		self.0 & 0x1 != 0
	}

	pub fn page_frame_address(&self) -> usize {
		self.0 & 0xFFFFF000
	}
}

#[repr(C, align(4096))]
pub struct PageTable(pub NonNull<[PageTableEntry; 1024]>);

impl PageTable {
	pub fn ref_table(&self) -> &[PageTableEntry; 1024] {
		unsafe { self.0.as_ref() }
	}

	fn mut_table(&mut self) -> &mut [PageTableEntry; 1024] {
		unsafe { self.0.as_mut() }
	}

	fn clear(&mut self) {
		for i in 0..self.ref_table().len() {
			self.mut_table()[i] = PageTableEntry::new(0, 0)
		}
	}

	fn set_entry(&mut self, index: usize, address: usize, flags: usize) {
		crate::println!(
			"set_entry table: index : {}, address : 0x{:x}, flags : {}",
			index,
			address,
			flags
		);
		self.mut_table()[index] = PageTableEntry::new(address, flags)
	}
}

#[derive(Debug)]
pub struct PageDirectoryEntry(usize);

#[allow(unused)]
impl PageDirectoryEntry {
	fn new(address: usize, flags: usize) -> PageDirectoryEntry {
		assert_eq!(0, address & 0xFFF);
		PageDirectoryEntry(address | flags)
	}

	pub fn page_table_address(&self) -> usize {
		self.0 & 0xFFFFF000
	}

	pub fn is_present(&self) -> bool {
		self.0 & 0x1 != 0
	}
}

#[repr(C, align(4096))]
pub struct PageDirectory(pub NonNull<[PageDirectoryEntry; 1024]>, bool);

impl PageDirectory {
	pub fn ref_dir(&self) -> &[PageDirectoryEntry; 1024] {
		unsafe { self.0.as_ref() }
	}

	fn mut_dir(&mut self) -> &mut [PageDirectoryEntry; 1024] {
		unsafe { self.0.as_mut() }
	}

	pub fn clear(&mut self) {
		for i in 0..self.ref_dir().len() {
			self.mut_dir()[i] = PageDirectoryEntry::new(0, 0);
			if i == self.ref_dir().len() - 1 {}
		}
	}

	fn table_address_add(&self, offset: usize) -> usize {
		if self.1 {
			0xFFC00000usize + (offset << 12) // if recursive mapping on
		} else {
			self.ref_dir()[offset].page_table_address()
		}
	}

	pub fn set_entry(&mut self, index: usize, address: usize, flags: usize) {
		crate::println!(
			"set_entry directory: index : {}, address : 0x{:x}, flags : {}",
			index,
			address,
			flags
		);
		self.mut_dir()[index] = PageDirectoryEntry::new(address, flags)
	}

	pub fn map_page(
		&mut self,
		virtual_address: usize,
		physical_address: usize,
		flags: usize,
	) -> Result<(), PhysicalMemoryError> {
		assert!(virtual_address & 0xFFF == 0, "Address is not 4KB aligned");
		assert!(physical_address & 0xFFF == 0, "Address is not 4KB aligned");
		let pdi = virtual_address >> 22;
		let pti = (virtual_address & 0x3FF000) >> 12;

		assert!(pdi != 1023, "over 0xFFC00000 is reserved");

		let mut page_table: PageTable;
		let page_table_add: usize;
		if !self.ref_dir()[pdi].is_present() {
			page_table_add = BITMAP.lock().alloc_frame()?;
			self.set_entry(pdi, page_table_add, 0x3);
			page_table =
				unsafe { PageTable(NonNull::new_unchecked(self.table_address_add(pdi) as *mut _)) };
			page_table.clear();
		} else {
			page_table =
				unsafe { PageTable(NonNull::new_unchecked(self.table_address_add(pdi) as *mut _)) }
		}
		assert!(
			!page_table.ref_table()[pti].is_present(),
			"page entry already present."
		);
		page_table.set_entry(pti, physical_address, flags);
		Ok(())
	}

	#[allow(unused)]
	pub fn unmap_page(&mut self, virtual_address: usize) -> Result<(), PhysicalMemoryError> {
		let pdi = (virtual_address >> 22) & 0x3FF;
		let pti = (virtual_address >> 12) & 0x3FF;

		assert!(
			self.ref_dir()[pdi].is_present(),
			"Directory entry not preset."
		);

		let mut page_table =
			unsafe { PageTable(NonNull::new_unchecked(self.table_address_add(pdi) as *mut _)) };
		assert!(
			page_table.ref_table()[pti].is_present(),
			"page entry not present."
		);
		BITMAP
			.lock()
			.free_frame(page_table.ref_table()[pti].page_frame_address())?;
		page_table.set_entry(pti, 0x0, 0x0);
		Ok(())
	}
}

unsafe impl Send for PageDirectory {}

pub static PAGE_DIRECTORY: Mutex<PageDirectory> = Mutex::new(PageDirectory(
	unsafe { NonNull::new_unchecked(PDA as *mut _) },
	false,
));

pub fn init(multiboot_info: usize) {
	let mut kernel_start_page = symbols::get_kernel_start() as usize & !0xFFF;
	let kernel_end_page = symbols::get_kernel_end() as usize & !0xFFF;
	let multiboot_frame_add = multiboot_info & !0xFFF;

	PAGE_DIRECTORY.lock().clear();

	PAGE_DIRECTORY.lock().map_page(0x0, 0x0, 0x3).unwrap();
	PAGE_DIRECTORY
		.lock()
		.map_page(0xb8000, 0xb8000, 0x3)
		.unwrap();
	while kernel_start_page <= kernel_end_page {
		PAGE_DIRECTORY
			.lock()
			.map_page(kernel_start_page, kernel_start_page, 0x3)
			.unwrap();
		kernel_start_page += 0x1000;
	}
	if multiboot_info <= symbols::get_kernel_start() as usize
		&& multiboot_info >= symbols::get_kernel_end() as usize
	{
		PAGE_DIRECTORY
			.lock()
			.map_page(multiboot_frame_add, multiboot_frame_add, 0x3)
			.unwrap();
	}
	PAGE_DIRECTORY.lock().set_entry(1023, PDA, 0x3);
	enable(PDA);
	*PAGE_DIRECTORY.lock() = unsafe {
		PageDirectory(
			NonNull::new_unchecked((0x3FFusize << 22 | 0x3FFusize << 12) as *mut _),
			true,
		)
	};
}

fn enable(page_dir_address: usize) {
	unsafe {
		asm!(
			"mov cr3, {pda}",
			"mov {tmp}, cr0",
			"or {tmp}, 0x80000000",
			"mov cr0, {tmp}",
			pda = in(reg) page_dir_address,
			tmp = out(reg) _,
		);
	}
}
