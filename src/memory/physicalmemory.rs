use crate::include::multiboot;
use crate::include::symbols;
#[allow(unused)]
use crate::println;
use spin::Mutex;

#[derive(Debug)]
pub enum PhysicalMemoryError {
	OutofMemory,
	NoFrameAvailable,
	FrameAlreadyUse,
	FrameNotInUse,
}

const N_FRAMES: usize = 1048576;
const BITMAP_LEN: usize = N_FRAMES / 32;

pub struct PhysicalMemory {
	pub bitmap: [u32; BITMAP_LEN],
	next: usize,
}

#[allow(unused)]
impl PhysicalMemory {
	fn next_available(&self) -> Result<usize, PhysicalMemoryError> {
		let idx = self
			.bitmap
			.iter()
			.skip(self.next)
			.position(|&x| x != 0xFFFFFFFF);

		idx.map_or(Err(PhysicalMemoryError::NoFrameAvailable), |i| {
			let mut j: usize = 0;
			let real_idx = i + self.next;
			while !self.bitmap[real_idx] & (0x80000000 >> j) == 0 {
				j += 1;
			}
			Ok((real_idx * 32 + j) * 0x1000)
		})
	}

	fn alloc_bitmap(&mut self, address: usize) -> Result<(), PhysicalMemoryError> {
		let index = address / 0x1000 / 0x20;
		let offset = address / 0x1000 % 0x20;

		if index >= BITMAP_LEN {
			return Err(PhysicalMemoryError::OutofMemory);
		}

		match self.bitmap[index] & (0x80000000 >> offset) == 0 {
			true => {
				self.bitmap[index] |= 0x80000000 >> offset;
				self.next = index;
				Ok(())
			}
			false => Err(PhysicalMemoryError::FrameAlreadyUse),
		}
	}

	pub fn free_frame(&mut self, address: usize) -> Result<(), PhysicalMemoryError> {
		let index = address / 0x1000 / 0x20;
		let offset = address / 0x1000 % 0x20;

		match self.bitmap[index] & (0x80000000 >> offset) != 0 {
			true => {
				self.bitmap[index] &= !(0x80000000 >> offset);
				self.next = index;
				Ok(())
			}
			false => Err(PhysicalMemoryError::FrameNotInUse),
		}
	}

	/// ## Alloc_frame
	/// Change the bitmap status the very next available frame. \
	/// Return with first frame address ```usize```. \
	/// Can panic with ```PhysicalMemoryError```
	pub fn alloc_frame(&mut self) -> Result<usize, PhysicalMemoryError> {
		let next = self.next_available()?;
		self.alloc_bitmap(next)?;
		Ok(next)
	}

	/// ## Alloc_frame_address
	/// Change the bitmap status with physical address. \
	/// No return but can panic.
	pub fn alloc_frame_address(&mut self, address: usize) -> Result<(), PhysicalMemoryError> {
		self.alloc_bitmap(address)
	}
}

pub static BITMAP: Mutex<PhysicalMemory> = Mutex::new(PhysicalMemory {
	bitmap: [0; BITMAP_LEN],
	next: 0,
});

/// ## Init physical memory
/// Take Multiboot memorymap and mark unuseable memory in bitmap. \
/// Mark the space of already take by kernel. ex) gdt, vga, ps2, etc...
pub fn init(memory_map: usize, multiboot_info: usize) {
	unsafe {
		let map = memory_map as *const multiboot::MultibootMemoryMapTag;
		let tag_size = (*map).size;
		let entry_size = (*map).entry_size as isize;

		let mut entry = map.offset(1) as *const multiboot::MultibootMemoryMapEntry;
		let entry_end = map as *const _ as usize + tag_size as usize;

		while (entry as usize) < entry_end {
			// println!(
			// 	"memory type: {}\nentry : {}\nentry_end : {}\n",
			// 	(*entry)._type,
			// 	entry as usize,
			// 	entry_end
			// );
			if (*entry)._type == 0 || (*entry)._type == 8 {
				break;
			}
			if (*entry)._type != 1 {
				let mut count = 0;
				let base_addr = (*entry).base_addr;
				let end_addr = (*entry).base_addr + (*entry).length;

				while base_addr + count < end_addr {
					BITMAP
						.lock()
						.alloc_frame_address((base_addr + count) as usize)
						.unwrap();
					count += 0x1000;
				}
			}
			entry = entry.offset(
				entry_size / core::mem::size_of::<multiboot::MultibootMemoryMapEntry>() as isize,
			);
			// println!("memory map entry: {:?}\n", (*entry));
		}

		BITMAP.lock().alloc_frame_address(0x0).unwrap();
		BITMAP.lock().alloc_frame_address(0xb8000).unwrap();

		let mut kernel_start = symbols::get_kernel_start() as usize & !0xFFF;
		let kernel_end = symbols::get_kernel_end() as usize & !0xFFF;
		while kernel_start <= kernel_end {
			BITMAP.lock().alloc_frame_address(kernel_start).unwrap();
			kernel_start += 0x1000;
		}

		let multiboot_info_frame = multiboot_info & !0xFFF;
		BITMAP
			.lock()
			.alloc_frame_address(multiboot_info_frame)
			.unwrap();
	}
}
