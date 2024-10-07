use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use crate::memory::physicalmemory::{BITMAP, N_FRAMES};

const MAX_ORDER: usize = 10;
pub struct HeapAllocator {
	free_lists: [[usize; 64]; MAX_ORDER + 1],
	free_counts: [usize; MAX_ORDER + 1],
}

impl HeapAllocator {
	const fn new() -> Self {
		HeapAllocator {
			free_lists: [[0; 64]; MAX_ORDER + 1],
			free_counts: [0; MAX_ORDER + 1],
		}
	}

	pub fn init(&mut self) {
		let mut current_order = MAX_ORDER;
		let mut current_size = 1 << MAX_ORDER;
		let mut frame = 0;

		while frame < N_FRAMES {
			if BITMAP.lock().is_frame_free(frame) && frame + current_size <= N_FRAMES {
				let addr = frame * 0x1000;
				if self.free_counts[current_order] < 64 {
					self.free_lists[current_order][self.free_counts[current_order]] = addr;
					self.free_counts[current_order] += 1;
				}
				frame += current_size;
			} else if current_order == 0 {
				frame += 1;
			} else {
				current_order -= 1;
				current_size /= 2;
			}
		}
	}

	fn size_to_order(&self, size: usize) -> Option<usize> {
		let mut order = 0;
		let mut block_size = 0x1000;
		while block_size < size && order < MAX_ORDER {
			block_size *= 2;
			order += 1;
		}
		if block_size >= size {
			Some(order)
		} else {
			None
		}
	}

	fn allocate(&mut self, layout: Layout) -> *mut u8 {
		let size = layout.size().max(layout.align()).max(0x1000);
		if let Some(order) = self.size_to_order(size) {
			self.allocate_order(order)
		} else {
			null_mut()
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
			if self.free_counts[order] < 256 {
				self.free_lists[order][self.free_counts[order]] = buddy;
				self.free_counts[order] += 1;
			}
		}

		addr as *mut u8
	}

	fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
		let size = layout.size().max(layout.align()).max(0x1000);
		if let Some(order) = self.size_to_order(size) {
			self.deallocate_order(ptr as usize, order);
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

		if self.free_counts[order] < 64 {
			self.free_lists[order][self.free_counts[order]] = current_addr;
			self.free_counts[order] += 1;
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
