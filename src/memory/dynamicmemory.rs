use crate::memory::physicalmemory::BITMAP;
use crate::println;
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

const MAX_ORDER: usize = 17; // 512mb
const PAGE_SIZE: usize = 0x1000;
const LIST_COUNT: usize = 256;

pub struct HeapAllocator {
	free_lists: [[usize; LIST_COUNT]; MAX_ORDER + 1],
	free_counts: [usize; MAX_ORDER + 1],
}

impl HeapAllocator {
	const fn new() -> Self {
		HeapAllocator {
			free_lists: [[0; LIST_COUNT]; MAX_ORDER + 1],
			free_counts: [0; MAX_ORDER + 1],
		}
	}

	#[allow(clippy::needless_range_loop)]
	pub fn init(&mut self, start_addr: usize, end_addr: usize) {
		let total_memory = end_addr - start_addr;

		let mut frame = start_addr / PAGE_SIZE;
		let end_frame = end_addr / PAGE_SIZE;

		let mut max_alloc_per_order = [usize::MAX; MAX_ORDER + 1];
		for order in 10..=MAX_ORDER {
			let block_size = 1 << order;
			max_alloc_per_order[order] = (total_memory / 4) / (block_size * PAGE_SIZE);
			// max_alloc_per_order[order] = max_alloc_per_order[order].min(128)
		}

		while frame < end_frame {
			let mut allocated = false;

			for order in (0..=MAX_ORDER).rev() {
				if order >= 10 && self.free_counts[order] >= max_alloc_per_order[order] {
					continue;
				}

				let block_size = 1 << order;
				if frame + block_size <= end_frame {
					let mut can_allocate = true;
					for i in 0..block_size {
						if !BITMAP.lock().is_frame_free(frame + i) {
							can_allocate = false;
							break;
						}
					}

					if can_allocate {
						let addr = frame * PAGE_SIZE;
						if self.free_counts[order] < self.free_lists[order].len() {
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
		println!("free_list: {:?}", self.free_counts);
	}

	fn size_to_order(&self, size: usize) -> Option<usize> {
		let mut order = 0;
		let mut block_size = PAGE_SIZE;
		while block_size < size && order <= MAX_ORDER {
			block_size *= 2;
			order += 1;
		}
		Some(order)
	}

	fn allocate(&mut self, layout: Layout) -> *mut u8 {
		let size = layout.size().max(layout.align()).max(PAGE_SIZE);
		let order = self.size_to_order(size).unwrap_or(MAX_ORDER);

		if order <= MAX_ORDER {
			self.allocate_order(order)
		} else {
			self.allocate_large(size)
		}
	}

	fn allocate_order(&mut self, mut order: usize) -> *mut u8 {
		while order <= MAX_ORDER && self.free_counts[order] == 0 {
			order += 1;
		}

		if order > MAX_ORDER {
			return null_mut();
		}

		let addr = self.free_lists[order][self.free_counts[order] - 1];
		self.free_counts[order] -= 1;

		BITMAP.lock().alloc_frame_address(addr).unwrap();

		while order > 0 {
			order -= 1;
			let buddy = addr ^ (1 << (order + 12));
			if self.free_counts[order] < 64 {
				self.free_lists[order][self.free_counts[order]] = buddy;
				self.free_counts[order] += 1;
			}
		}

		addr as *mut u8
	}

	fn allocate_large(&mut self, size: usize) -> *mut u8 {
		let mut remaining_size = size;
		let mut base_addr: *mut u8 = null_mut();
		let mut current_addr: *mut u8 = null_mut();

		while remaining_size > 0 {
			let order = (0..=MAX_ORDER)
				.rev()
				.find(|&o| {
					let block_size = 1 << (o + 12);
					block_size <= remaining_size && self.free_counts[o] > 0
				})
				.unwrap_or(0);

			let block_size = 1 << (order + 12);
			let addr = self.allocate_order(order);

			if addr.is_null() {
				if !base_addr.is_null() {
					self.deallocate_large(base_addr as usize, size - remaining_size);
				}
				return null_mut();
			}

			if base_addr.is_null() {
				base_addr = addr;
			} else if addr as usize
				!= current_addr as usize + (current_addr as usize - base_addr as usize)
			{
				self.deallocate_large(base_addr as usize, size - remaining_size);
				return null_mut();
			}

			current_addr = addr;
			remaining_size = remaining_size.saturating_sub(block_size);
		}

		base_addr
	}

	#[allow(clippy::manual_div_ceil)]
	fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
		let size = layout.size().max(layout.align()).max(PAGE_SIZE);
		let order = self.size_to_order(size).unwrap_or(MAX_ORDER);

		if order <= MAX_ORDER {
			self.deallocate_order(ptr as usize, order);
		} else {
			let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
			self.deallocate_large(ptr as usize, num_pages * PAGE_SIZE);
		}
	}

	fn deallocate_order(&mut self, addr: usize, mut order: usize) {
		let mut current_addr = addr;
		while order < MAX_ORDER {
			let buddy = current_addr ^ (1 << (order + 12));
			let buddy_index = self.free_lists[order][..self.free_counts[order]]
				.iter()
				.position(|&block| block == buddy);

			if let Some(index) = buddy_index {
				self.free_counts[order] -= 1;
				self.free_lists[order][index] = self.free_lists[order][self.free_counts[order]];
				current_addr = current_addr.min(buddy);
				order += 1;
			} else {
				break;
			}
		}

		BITMAP.lock().free_frame(addr).unwrap();

		if self.free_counts[order] < LIST_COUNT {
			self.free_lists[order][self.free_counts[order]] = current_addr;
			self.free_counts[order] += 1;
		}
	}

	#[allow(clippy::manual_div_ceil)]
	fn deallocate_large(&mut self, ptr: usize, size: usize) {
		let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
		for i in 0..num_pages {
			let page_addr = ptr + i * PAGE_SIZE;
			let order = self.size_to_order(PAGE_SIZE).unwrap();
			self.deallocate_order(page_addr, order);
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
pub static ALLOCATOR: Locked<HeapAllocator> = Locked::new(HeapAllocator::new());

unsafe impl GlobalAlloc for Locked<HeapAllocator> {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.lock().allocate(layout)
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.lock().deallocate(ptr, layout)
	}
}
