#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused)]

use crate::memory;
use crate::memory::dynamicmemory::{KERNEL_ALLOCATOR, USER_ALLOCATOR};
use crate::println;
use alloc::vec;
use core::alloc::Layout;

fn simple_str_test() {
	{
		println!("----------simple str alloc test------------");
		KERNEL_ALLOCATOR.lock().print_free_list();
		let mut a = alloc::string::String::new();
		a.push_str("Hello It's 42 project, kfs3");
		println!("str result: {}, size: {}", a, a.len());
		KERNEL_ALLOCATOR.lock().print_free_list();
	}
	println!("***************out of block, all drop***************");
	KERNEL_ALLOCATOR.lock().print_free_list();
	println!();
}

fn kalloc_test() {
	println!("----------kalloc test------------");
	unsafe {
		let ptr = KERNEL_ALLOCATOR.lock().kallocate(4097);
		if ptr.is_null() {
			println!("Allocated failed.");
			return;
		}
		*ptr = 42;
		println!("ptr value: {}, address {:p}", *ptr, ptr);
		let ptr2 = KERNEL_ALLOCATOR.lock().kallocate(4097);
		if ptr2.is_null() {
			println!("Allocated failed.");
			return;
		}
		*ptr2 = 42;
		println!(
			"ptr without deallocate value: {}, address {:p}",
			*ptr2, ptr2
		);
		KERNEL_ALLOCATOR.lock().kdeallocate(ptr, 4097);
	}
}

fn user_alloc_test(paging_status: bool) {
	println!("----------User space alloc test------------");

	let a1 = Layout::from_size_align(10, 4096).unwrap();
	let a2 = Layout::from_size_align(20, 4096).unwrap();
	let a3 = Layout::from_size_align(65536, 4096).unwrap(); // 64KB
	let a4 = Layout::from_size_align(262144, 4096).unwrap(); // 128KB
	let a5 = Layout::from_size_align(524288, 4096).unwrap(); // 512KB
	let a6 = Layout::from_size_align(67108864, 4096).unwrap(); // 64MB
	let a7 = Layout::from_size_align(134217728, 4096).unwrap(); // 128MB
	let a8 = Layout::from_size_align(188743680, 4096).unwrap(); // 180MB

	{
		let mut allocator = USER_ALLOCATOR.lock();
		allocator.print_free_list();
	}

	let ptr1 = USER_ALLOCATOR.lock().allocate(a1);
	let ptr2 = USER_ALLOCATOR.lock().allocate(a2);
	let ptr3 = USER_ALLOCATOR.lock().allocate(a3);
	let ptr4 = USER_ALLOCATOR.lock().allocate(a4);
	let ptr5 = USER_ALLOCATOR.lock().allocate(a5);

	if paging_status {
		let ptr6 = USER_ALLOCATOR.lock().allocate(a6);
		let ptr7 = USER_ALLOCATOR.lock().allocate(a7);
		let ptr8 = USER_ALLOCATOR.lock().allocate(a8);

		println!("----------after allocation------------");

		{
			let mut allocator = USER_ALLOCATOR.lock();
			allocator.print_free_list();
			allocator.deallocate(ptr1, a1);
			allocator.deallocate(ptr2, a2);
			allocator.deallocate(ptr3, a3);
			allocator.deallocate(ptr4, a4);
			allocator.deallocate(ptr5, a5);
			allocator.deallocate(ptr6, a6);
			allocator.deallocate(ptr7, a7);
			allocator.deallocate(ptr8, a8);
		}
	} else {
		println!("----------after allocation------------");

		{
			let mut allocator = USER_ALLOCATOR.lock();
			allocator.print_free_list();
			allocator.deallocate(ptr1, a1);
			allocator.deallocate(ptr2, a2);
			allocator.deallocate(ptr3, a3);
			allocator.deallocate(ptr4, a4);
			allocator.deallocate(ptr5, a5);
		}
	}

	println!("----------after deallocation------------");

	{
		let mut allocator = USER_ALLOCATOR.lock();
		allocator.print_free_list();
	}
	println!();
}

fn vec_alloc_test(paging_status: bool) {
	println!("----------vec alloc test------------");
	{
		KERNEL_ALLOCATOR.lock().print_free_list();
	}
	{
		let vec1: vec::Vec<u8> = vec![0; 4 * 1024]; // 4KB
		let vec2: vec::Vec<u8> = vec![0; 8 * 1024]; // 8KB
		let vec3: vec::Vec<u8> = vec![0; 16 * 1024]; // 16KB
		let vec4: vec::Vec<u8> = vec![0; 32 * 1024]; // 32KB
		let vec5: vec::Vec<u8> = vec![0; 64 * 1024]; // 64KB
		let vec6: vec::Vec<u8> = vec![0; 128 * 1024]; // 128KB
		let vec7: vec::Vec<u8> = vec![0; 1024 * 1024]; // 1MB
		if paging_status {
			let vec8 = vec![0; 10 * 1024 * 1024]; // 10MB
			let vec9 = vec![0; 100 * 1024 * 1024]; // 100MB
		}
		println!("----------after allocation------------");
		{
			KERNEL_ALLOCATOR.lock().print_free_list();
		}
	}
	println!("***************out of block, all drop***************");
	{
		KERNEL_ALLOCATOR.lock().print_free_list();
	}
	println!();
}

pub fn alloc_test(paging_status: bool) {
	simple_str_test();
	vec_alloc_test(paging_status);
	user_alloc_test(paging_status);
	kalloc_test();
}
