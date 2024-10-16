use crate::memory::physicalmemory::BITMAP;
use crate::memory::virtualmemory::PAGE_DIRECTORY;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

const MAX_ORDER: usize = 10;
const PAGE_SIZE: usize = 0x1000;
const LIST_COUNT: usize = 512;

#[derive(PartialEq)]
pub enum Privilege {
	Kernel,
	User,
	None,
}

pub struct HeapAllocator {
	free_lists: [[usize; LIST_COUNT]; MAX_ORDER + 1],
	free_counts: [usize; MAX_ORDER + 1],
	privilege: Privilege,
	next_virtual_addr: usize,
	paging_status: bool,
}

impl HeapAllocator {
	const fn new() -> Self {
		HeapAllocator {
			free_lists: [[0; LIST_COUNT]; MAX_ORDER + 1],
			free_counts: [0; MAX_ORDER + 1],
			privilege: Privilege::None,
			next_virtual_addr: 0,
			paging_status: false,
		}
	}

	#[allow(clippy::needless_range_loop)]
	pub fn init(
		&mut self,
		start_addr: usize,
		end_addr: usize,
		privilege: Privilege,
		paging_status: bool,
	) {
		assert!(start_addr % 0x1000 == 0, "Address is not 4KB aligned");
		assert!(end_addr % 0x1000 == 0, "Address is not 4KB aligned");
		self.privilege = privilege;
		self.next_virtual_addr = start_addr;
		self.paging_status = paging_status;

		let mut frame = start_addr / PAGE_SIZE;
		let end_frame = end_addr / PAGE_SIZE;
		let mut index = 0;
		let mut j;

		while frame < end_frame {
			let mut allocated = false;
			for order in 0..=MAX_ORDER {
				let block_size = 1 << order;
				if frame + block_size <= end_frame {
					let mut can_allocate = true;
					j = 0;
					while j < block_size && frame + j + index < 32767 {
						if !BITMAP.lock().is_frame_free(frame + j + index) {
							can_allocate = false;
							break;
						}
						j += 1;
					}
					index += j;

					if can_allocate {
						let addr = frame * PAGE_SIZE;
						if self.free_counts[order] < LIST_COUNT {
							self.free_lists[order][self.free_counts[order]] = addr;
							self.free_counts[order] += 1;
							frame += block_size;
							allocated = true;
							break;
						}
					}
				}
			}
			if !allocated {
				frame += 1;
			}
		}
		// crate::println!("list: {:?}", self.free_counts);
	}

	fn size_to_order(&self, size: usize) -> Option<usize> {
		let mut order = 0;
		let mut block_size = PAGE_SIZE;
		while block_size < size {
			block_size *= 2;
			order += 1;
		}
		if order <= MAX_ORDER {
			Some(order)
		} else {
			None
		}
	}

	fn allocate(&mut self, layout: Layout) -> *mut u8 {
		let size = layout.size().max(layout.align());
		let order = self.size_to_order(size);
		crate::println!("alloc size : {}, order : {}", size, order.unwrap());

		match order {
			Some(o) => {
				if self.free_counts[o] > 0 {
					let physical_address = self.free_lists[o][self.free_counts[o] - 1];
					self.free_counts[o] -= 1;
					self.allocate_address(physical_address, o)
				} else {
					let mut higher_order = o + 1;
					while higher_order <= MAX_ORDER && self.free_counts[higher_order] == 0 {
						higher_order += 1;
					}

					if higher_order <= MAX_ORDER {
						self.allocate_split(higher_order, o)
					} else {
						self.allocate_merge(o)
					}
				}
			}
			None => null_mut(),
		}
	}

	fn allocate_split(&mut self, higher_order: usize, target_order: usize) -> *mut u8 {
		let physical_address = self.free_lists[higher_order][self.free_counts[higher_order] - 1];
		self.free_counts[higher_order] -= 1;

		let mut current_order = higher_order;
		while current_order > target_order {
			current_order -= 1;
			let buddy = physical_address ^ (1 << (current_order + 12));
			if self.free_counts[current_order] < LIST_COUNT {
				self.free_lists[current_order][self.free_counts[current_order]] = buddy;
				self.free_counts[current_order] += 1;
			}
		}

		self.allocate_address(physical_address, target_order)
	}

	fn allocate_merge(&mut self, target_order: usize) -> *mut u8 {
		let mut current_order = target_order;
		while current_order > 0 {
			current_order -= 1;
			let required_blocks = 1 << (target_order - current_order);

			if self.free_counts[current_order] >= required_blocks {
				let mut base_addr =
					self.free_lists[current_order][self.free_counts[current_order] - 1];
				self.free_counts[current_order] -= 1;

				for _ in 1..required_blocks {
					let buddy = base_addr ^ (1 << (current_order + 12));
					if let Some(index) = self.free_lists[current_order]
						[..self.free_counts[current_order]]
						.iter()
						.position(|&x| x == buddy)
					{
						self.free_lists[current_order]
							.swap(index, self.free_counts[current_order] - 1);
						self.free_counts[current_order] -= 1;
					} else {
						return null_mut();
					}
					base_addr = base_addr.min(buddy);
				}
				return self.allocate_address(base_addr, 1 << target_order);
			}
		}
		null_mut()
	}

	fn allocate_address(&mut self, physical_address: usize, order: usize) -> *mut u8 {
		let virtual_address = self.next_virtual_addr;
		let num_pages = 1 << order;
		for i in 0..num_pages {
			let cur_virtual_addr = virtual_address + i * PAGE_SIZE;
			let cur_physical_addr = physical_address + i * PAGE_SIZE;

			crate::println!(
				"allocate : physical 0x{:x}, virtual 0x{:x}, num_pages {}, order {}",
				cur_physical_addr,
				cur_virtual_addr,
				num_pages,
				order
			);

			BITMAP
				.lock()
				.alloc_frame_address(cur_physical_addr)
				.unwrap();
			if self.paging_status {
				PAGE_DIRECTORY
					.lock()
					.map_page(
						cur_virtual_addr,
						cur_physical_addr,
						if self.privilege == Privilege::Kernel {
							0x3
						} else {
							0x7
						},
					)
					.unwrap();
			}
		}

		self.next_virtual_addr += num_pages * PAGE_SIZE;
		virtual_address as *mut u8
	}

	/// ### Similar to size_to_order but ends at MAX_ORDER and returns
	/// return value \
	/// - usize: 0 > usize > max_order \
	/// - -1: size < PAGE_SIZE
	fn match_order(&self, max_order: usize, size: usize) -> i32 {
		let mut order: i32 = -1;
		let mut block_size = PAGE_SIZE;
		while block_size < size && order <= max_order as i32 {
			block_size *= 2;
			order += 1;
		}
		order
	}

	#[allow(unused)]
    pub fn deallocate(&mut self, addr: *mut u8, layout: Layout) {
        let size = layout.size().max(layout.align());
        let order = self.size_to_order(size);
        let mut physical_address = addr as usize;

		// crate::println!("before list: {:?}", self.free_counts);
		match order {
			Some(order) => {
				// physical
				self.free_lists[order][self.free_counts[order] - 1] = physical_address;
				self.free_counts[order] += 1;
				// virtual
				// if paging == true
				let num_pages = 1 << order;
				for i in 0..num_pages {
					let virtual_addr = physical_address + i * PAGE_SIZE;
					PAGE_DIRECTORY.lock().unmap_page(virtual_addr).unwrap();
				}
				crate::println!("deallocate: size {}, order {}, pa 0x{:X}", size, order, physical_address);
			}
			None => {
				// goal     : handle deallocation of exceeding MAX_ORDER memory section
				// condition: order > MAX_ORDER
				// state    : non testable -> alloc more than MAX_ORDER causes page fault
				//
				let mut remain_size: usize = 0;
				let mut dealloc_size: usize = 0;
				let mut order: i32 = 0;
				let mut o = size / PAGE_SIZE;
				while order < 0 && remain_size == 0 {
					remain_size = size - dealloc_size; // size % PAGE_SIZE
					order = self.match_order(o, remain_size); // size / PAGE_SIZE
					if order < 0 && remain_size == 0 {
						crate::println!("deallocated successfully: no leaks");
						break;
					}
					// physical
					// why indexing error? over 512
					self.free_lists[order as usize][self.free_counts[order as usize] - 1] = physical_address;
					self.free_counts[order as usize] += 1;
					// virtual
					let num_pages = 1 << order;
					for i in 0..num_pages {
						let virtual_addr = physical_address + i * PAGE_SIZE;
						PAGE_DIRECTORY.lock().unmap_page(virtual_addr).unwrap();
					}
					dealloc_size = order as usize * PAGE_SIZE;
				}
				crate::println!("size: {}, addr: 0x{:X}", size, physical_address);
			}
		}
    }

	// #[allow(clippy::manual_div_ceil)]
	// fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
	// 	let size = layout.size().max(layout.align()).max(PAGE_SIZE);
	// 	let order = self.size_to_order(size);

	// 	// crate::println!("dealloc size : {}, order : {}", size, order.unwrap());
	// 	match order {
	// 		Some(o) => self.deallocate_order(ptr as usize, o),
	// 		_ => {
	// 			let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
	// 			self.deallocate_large(ptr as usize, num_pages * PAGE_SIZE);
	// 		}
	// 	}
	// }

	// fn deallocate_order(&mut self, addr: usize, mut order: usize) {
	// 	let num_pages = 1 << order;
	// 	// crate::println!(
	// 	// 	"deallocate addr : 0x{:x}, num_pages : {}, order : {}, virtual addr : 0x{:x}",
	// 	// 	addr,
	// 	// 	num_pages,
	// 	// 	order,
	// 	// 	addr * PAGE_SIZE
	// 	// );
	// 	for i in 0..num_pages {
	// 		let cur_virtual_addr = addr + i * PAGE_SIZE;
	// 		// BITMAP.lock().free_frame(cur_virtual_addr).unwrap();
	// 		PAGE_DIRECTORY.lock().unmap_page(cur_virtual_addr).unwrap();
	// 	}

	// 	let mut current_addr = addr;
	// 	while order < MAX_ORDER {
	// 		let buddy = current_addr ^ (1 << (order + 12));
	// 		let buddy_index = self.free_lists[order][..self.free_counts[order]]
	// 			.iter()
	// 			.position(|&block| block == buddy);

	// 		if let Some(index) = buddy_index {
	// 			self.free_counts[order] -= 1;
	// 			self.free_lists[order][index] = self.free_lists[order][self.free_counts[order]];
	// 			current_addr = current_addr.min(buddy);
	// 			order += 1;
	// 		} else {
	// 			break;
	// 		}
	// 	}
	// 	if self.free_counts[order] < LIST_COUNT {
	// 		self.free_lists[order][self.free_counts[order]] = current_addr;
	// 		self.free_counts[order] += 1;
	// 	}
	// }

	// #[allow(clippy::manual_div_ceil)]
	// fn deallocate_large(&mut self, ptr: usize, size: usize) {
	// 	let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
	// 	for i in 0..num_pages {
	// 		let page_addr = ptr + i * PAGE_SIZE;
	// 		let order = self.size_to_order(PAGE_SIZE).unwrap();
	// 		self.deallocate_order(page_addr, order);
	// 	}
	// }
}

pub struct Locked<A> {
	inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
	pub const fn new(inner: A) -> Self {
		Locked {
			inner: spin::Mutex::new(inner),
		}
	}

	pub fn lock(&self) -> spin::MutexGuard<A> {
		self.inner.lock()
	}
}

#[global_allocator]
pub static GLOBAL_ALLOCATOR: Locked<HeapAllocator> = Locked::new(HeapAllocator::new());

pub static USER_ALLOCATOR: Locked<HeapAllocator> = Locked::new(HeapAllocator::new());

unsafe impl GlobalAlloc for Locked<HeapAllocator> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		let address = self.lock().allocate(layout) as usize;
		if address == 0 {
			return null_mut();
		}
		address as *mut u8
		// loop {}
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.lock().deallocate(ptr, layout)
	}
}
