#[repr(C, packed)]
struct GdtEntry {
	limit_low: u16,
	base_low: u16,
	base_middle: u8,
	access: u8,
	granularity: u8,
	base_high: u8,
}

impl GdtEntry {
	fn new(base: u32, limit: u32, access: u8, gran: u8) -> GdtEntry {
		GdtEntry {
			base_low: (base & 0xFFFF) as u16,
			base_middle: ((base >> 16) & 0xFF) as u8,
			base_high: ((base >> 24) & 0xFF) as u8,
			limit_low: (limit & 0xFFFF) as u16,
			access,
			granularity: ((limit >> 16) & 0x0F) as u8 | (gran << 4),
		}
	}
}

#[repr(C, packed)]
struct Gdt {
	null_segment: GdtEntry,
	kernel_code_segment: GdtEntry,
	kernel_data_segment: GdtEntry,
	kernel_stack_segment: GdtEntry,
	user_code_segment: GdtEntry,
	user_data_segment: GdtEntry,
	user_stack_segment: GdtEntry,
}

static mut GDT_PTR: *mut Gdt = 0x00000800 as *mut Gdt;

#[allow(unused)]
impl Gdt {
	pub fn new() -> Gdt {
		Gdt {
			null_segment: GdtEntry::new(0, 0, 0, 0),
			kernel_code_segment: GdtEntry::new(0x0, 0xFFFFF, 0x9A, 0xC),
			kernel_data_segment: GdtEntry::new(0x0, 0xFFFFF, 0x92, 0xC),
			kernel_stack_segment: GdtEntry::new(0x0, 0xFFFFF, 0x92, 0xC),
			user_code_segment: GdtEntry::new(0x0, 0xFFFFF, 0xFA, 0xC),
			user_data_segment: GdtEntry::new(0x0, 0xFFFFF, 0xF2, 0xC),
			user_stack_segment: GdtEntry::new(0x0, 0xFFFFF, 0xF2, 0xC),
		}
	}
}

#[repr(C, packed)]
struct GdtDescriptor {
	limit: u16,
	base: u32,
}

pub fn load() {
	unsafe {
		use core::arch::asm;
		use core::ptr;

		let gdt = Gdt::new();

		ptr::write_volatile(GDT_PTR, gdt);

		let gdtr = GdtDescriptor {
			limit: (core::mem::size_of::<Gdt>() - 1) as u16,
			base: GDT_PTR as u32,
		};

		asm!(
			"	lgdtl ({})
			ljmp $0x08, $1f

		1:
			movw $0x10, %ax
			movw %ax, %ds
			movw %ax, %es
			movw %ax, %fs
			movw %ax, %gs
			movw %ax, %ss",
			in(reg) &gdtr as *const GdtDescriptor,
			out("ax") _,
			options(att_syntax)
		);
	}
}
