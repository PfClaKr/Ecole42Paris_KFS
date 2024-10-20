use crate::memory::physicalmemory::{BITMAP, N_FRAMES};
use crate::memory::virtualmemory::PAGE_DIRECTORY;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

const MAX_ORDER: usize = 10;
const PAGE_SIZE: usize = 0x1000;
const LIST_COUNT: usize = 1000;
const LIST_COUNT_INIT_MAX: usize = (LIST_COUNT / 10) * 3;

#[derive(PartialEq, Debug)]
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

		if paging_status {
			PAGE_DIRECTORY.lock().init_directory(start_addr, end_addr);
		}
		let mut frame = start_addr / PAGE_SIZE;
		let end_frame = end_addr / PAGE_SIZE;
		let mut index = 0;
		let mut j;

		while frame < end_frame {
			let mut allocated = false;
			for order in (0..=MAX_ORDER).rev() {
				let block_size = 1 << order;
				if frame + block_size <= end_frame {
					let mut can_allocate = true;
					j = 0;
					while j < block_size && frame + j + index < N_FRAMES {
						if !BITMAP.lock().is_frame_free(frame + j + index) {
							can_allocate = false;
							break;
						}
						j += 1;
					}
					index += j;

					if can_allocate {
						let addr = frame * PAGE_SIZE;
						if self.free_counts[order] < LIST_COUNT_INIT_MAX {
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
		// crate::println!(
		// 	"{} list: {:?}",
		// 	if self.privilege == Privilege::User {
		// 		"User"
		// 	} else {
		// 		"Kernel"
		// 	},
		// 	self.free_counts
		// );
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

	/// ### Similar to size_to_order but ends at MAX_ORDER and returns
	/// return value \
	/// - Some: 0 > usize > max_order \
	/// - None: size < PAGE_SIZE
	fn match_order(&self, size: usize) -> Option<usize> {
		let mut order: usize = 0;
		let mut block_size = PAGE_SIZE;

		if block_size > size || size % PAGE_SIZE != 0 {
			None
		} else {
			while block_size < size && order < MAX_ORDER {
				block_size *= 2;
				order += 1;
			}
			// crate::println!("{}, {} < {}, ", order, block_size, size);
			if block_size > size {
				order -= 1;
			}
			Some(order)
		}
	}

	pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
		let size = layout.size().max(layout.align());
		let order = self.size_to_order(size);
		// crate::println!("alloc size : {}, order : {}", size, order.unwrap_or(1000));

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
						// crate::println!("allocate_split in");
						self.allocate_split(higher_order, o)
					} else {
						// crate::println!("allocate_merge in higher_order: {}", higher_order);
						self.allocate_merge(o)
					}
				}
			}
			None => {
				// crate::println!("allocate_large in size : {}", size);
				assert!(
					self.paging_status,
					"Without paging, allocate max size is 4mb"
				);
				self.allocate_large(size)
			}
		}
	}

	fn allocate_large(&mut self, size: usize) -> *mut u8 {
		let mut remaining_size = size as i64;
		let mut base_addr: *mut u8 = null_mut();

		while remaining_size > 0 {
			let order = self.match_order(remaining_size as usize).unwrap();
			let block_size = (1 << order) * PAGE_SIZE as i64;
			// crate::println!(
			// 	"alloc large remaining_size : {}, block_size {}, order {}",
			// 	remaining_size,
			// 	block_size,
			// 	order
			// );
			if self.free_counts[order] > 0 {
				let physical_address = self.free_lists[order][self.free_counts[order] - 1];
				self.free_counts[order] -= 1;
				if base_addr.is_null() {
					base_addr = self.allocate_address(physical_address, order);
				} else {
					self.allocate_address(physical_address, order);
				}
				remaining_size -= block_size;
			} else {
				return null_mut();
			}
		}

		base_addr
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
				// crate::println!(
				// 	"alloc merge, current_order : {}, required_blocks : {}",
				// 	current_order,
				// 	required_blocks
				// );
				if self.free_counts[current_order] >= required_blocks {
					let base_addr =
						self.free_lists[current_order][self.free_counts[current_order] - 1];
					self.free_counts[current_order] -= 1;
					self.allocate_address(base_addr, current_order);
				}
				//maybe will be implement merge block for contiguous physical address
				// for _ in 1..required_blocks {
				// 	let buddy = base_addr ^ (1 << (current_order + 12));
				// 	if let Some(index) = self.free_lists[current_order]
				// 		[..self.free_counts[current_order]]
				// 		.iter()
				// 		.position(|&x| x == buddy)
				// 	{
				// 		self.free_lists[current_order]
				// 			.swap(index, self.free_counts[current_order] - 1);
				// 		self.free_counts[current_order] -= 1;
				// 	} else {
				// 		return null_mut();
				// 	}
				// 	base_addr = base_addr.min(buddy);
				// }
				// return self.allocate_address(base_addr, 1 << target_order);
			}
		}
		null_mut()
	}

	fn allocate_address(&mut self, physical_address: usize, order: usize) -> *mut u8 {
		let virtual_address = self.next_virtual_addr;
		let num_pages = 1 << order;
		for i in 0..num_pages {
			let cur_virtual_addr = virtual_address + (i * PAGE_SIZE);
			let cur_physical_addr = physical_address + (i * PAGE_SIZE);

			// if i == 0 {
			//     crate::println!(
			//         "allocate : physical 0x{:x}, virtual 0x{:x}, pages {}/{}, order {}",
			//         cur_physical_addr,
			//         cur_virtual_addr,
			//         i + 1,
			//         num_pages,
			//         order
			//     );
			// }
			// crate::println!(
			//     "allocate : physical 0x{:x}, virtual 0x{:x}, pages {}/{}, order {}",
			//     cur_physical_addr,
			//     cur_virtual_addr,
			//     i + 1,
			//     num_pages,
			//     order
			// );

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
		if self.paging_status {
			virtual_address as *mut u8
		} else {
			physical_address as *mut u8
		}
	}

	pub fn deallocate(&mut self, addr: *mut u8, layout: Layout) {
		let size = layout.size().max(layout.align());
		let order = self.size_to_order(size);
		let mut virtual_address = addr as usize;

		// crate::println!("{:?}", layout);
		// crate::println!("before list: {:?}", self.free_counts);
		// crate::println!("deallocate: size: {} order: {:?}", size, order);
		match order {
			Some(order) => {
				// physical
				// crate::println!("deallocate: size {}, order {}, pa 0x{:X}", size, order, virtual_address);
				self.free_lists[order][self.free_counts[order]] = virtual_address;
				self.free_counts[order] += 1;
				// virtual
				// if paging == true
				let num_pages = 1 << order;
				for i in 0..num_pages {
					let virtual_addr = virtual_address + i * PAGE_SIZE;
					if self.paging_status {
						PAGE_DIRECTORY.lock().unmap_page(virtual_addr).unwrap();
					} else {
						BITMAP.lock().free_frame(virtual_addr).unwrap();
					}
				}
			}
			None => {
				// crate::println!("Inside partial deallocate");
				assert!(self.paging_status, "Without paging, kfree max size is 4mb");
				let mut remain_size: usize = size;
				// let mut dealloc_size: usize = 0;
				while remain_size != 0 {
					// crate::println!("deallocate: inside loop: rs {}, ds {}, va 0x{:X}", remain_size, dealloc_size, virtual_address);
					// crate::println!("list: {:?}", self.free_counts);
					match self.match_order(remain_size) {
						Some(mut order) => {
							// crate::println!("deallocate: size {}, order {}, pa 0x{:X}", size, order, virtual_address);
							// crate::println!("deallocate: in some: order {} rs {}", order, remain_size);
							// crate::println!("order: {} free list: {:?} free count: {:?}", order, self.free_lists[order], self.free_counts);
							// physical
							while self.free_counts[order] == LIST_COUNT {
								// assert!(order == 0, "deallocation: {:?}", self.free_counts);
								order -= 1;
							}
							self.free_lists[order][self.free_counts[order]] = virtual_address;
							self.free_counts[order] += 1;
							let num_pages = 1 << order;
							// virtual
							for i in 0..num_pages {
								let virtual_addr = virtual_address + i * PAGE_SIZE;
								PAGE_DIRECTORY.lock().unmap_page(virtual_addr).unwrap();
							}
							// crate::println!("deallocated va: 0x{:X}", virtual_address + num_pages * PAGE_SIZE);
							virtual_address += num_pages * PAGE_SIZE;
							// dealloc_size = num_pages * PAGE_SIZE;
							remain_size -= num_pages * PAGE_SIZE;
							// crate::println!("deallocation process: freed {}, left {}", dealloc_size, remain_size);
						}
						None => {
							// crate::println!("deallocate: in None");
							// if remain_size == 0 {
							// 	crate::println!(
							// 		"deallocated successfully: no leaks ({})",
							// 		remain_size
							// 	);
							// } else {
							// 	crate::println!(
							// 		"deallocate failed: possible leaks ({})",
							// 		remain_size
							// 	);
							// }
							break;
						}
					}
				}
				// crate::println!("partial deallocation leaks: ({})", remain_size);
				// crate::println!("size: {}, addr: 0x{:X}", size, virtual_address);
			}
		}
		// crate::println!("after list: {:?}", self.free_counts);
	}

	pub fn print_free_list(&self) {
		crate::println!(
			"{}, count: {:?}",
			if self.privilege == Privilege::User {
				"User"
			} else {
				"Kernel"
			},
			self.free_counts
		);
	}
	/// ## kallocate
	/// alloc contiguous physical memory with request size. \
	/// return raw address of start of memory.\
	/// ## Warning
	/// It's your reponsibility of free the memory, if not you may have memory leak.
	/// ## Example
	/// ```
	/// let ptr = unsafe { Allocator::kallocate(size) };
	/// if !ptr.is_null() {
	///     unsafe {
	///         *ptr = 42;
	///         println!("the value of ptr: {}", *ptr);
	///         Allocator::kdeallocate(ptr, size);
	///     }
	/// }
	/// ```
	pub unsafe fn kallocate(&mut self, size: usize) -> *mut u8 {
		assert_eq!(
			self.privilege,
			Privilege::Kernel,
			"User allocator can not allocate kernel space"
		);
		let size = (size + 4095) & !4095;
		let order = self.size_to_order(size);
		// crate::println!("size = {}, order = {}", size, order.unwrap());

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
						// can not sure contiguous physical memory
						null_mut()
					}
				}
			}
			None => {
				// can not allocate several memory block
				null_mut()
			}
		}
	}

	pub unsafe fn kdeallocate(&mut self, ptr: *mut u8, size: usize) {
		assert_eq!(
			self.privilege,
			Privilege::Kernel,
			"User allocator can not deallocate kernel space"
		);
		let ptr = ptr as usize;
		let size = (size + 4095) & !4095;
		let order = self.size_to_order(size);

		match order {
			Some(order) => {
				self.free_lists[order][self.free_counts[order]] = ptr;
				self.free_counts[order] += 1;
				let num_pages = 1 << order;
				for i in 0..num_pages {
					let virtual_addr = ptr + i * PAGE_SIZE;
					if self.paging_status {
						PAGE_DIRECTORY.lock().unmap_page(virtual_addr).unwrap();
					} else {
						BITMAP.lock().free_frame(virtual_addr).unwrap();
					}
				}
			}
			None => {
				assert!(
					size > 0x400000,
					"kdeallocate size value error. can not excced 4mb"
				)
			}
		}
	}
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
pub static KERNEL_ALLOCATOR: Locked<HeapAllocator> = Locked::new(HeapAllocator::new());

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
