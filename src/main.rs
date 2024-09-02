#![no_std]  // 표준 라이브러리 비활성화
#![no_main] // 표준 진입점 비활성화
#![feature(naked_functions)] // `naked` 함수 속성 활성화

use core::panic::PanicInfo;
use core::arch::asm; // `asm` 매크로 가져오기

/// Multiboot 헤더를 설정합니다.
#[repr(C)]
pub struct Multiboot {
    magic: u32,
    flags: u32,
    checksum: u32,
    header_addr: u32,
}

#[link_section = ".multiboot"]
#[no_mangle]
pub static MULTIBOOT: Multiboot = Multiboot {
    magic: 0xE85250D6,             // Magic number
    flags: 0x0,                    // Flags
    checksum: (0xE85250D6u32.wrapping_neg()), // Checksum (Magic + Flags + Checksum = 0이어야 함)
    header_addr: 0,                // 추가 데이터 (필요에 따라 사용)
};

/// Rust 진입점
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let vga_buffer = 0xb8000 as *mut u8;

    // 화면에 "42" 출력
    unsafe {
        *vga_buffer.offset(0) = b'4';
        *vga_buffer.offset(1) = 0x07; // 흰색
        *vga_buffer.offset(2) = b'2';
        *vga_buffer.offset(3) = 0x07; // 흰색
    }

    loop {}
}

/// 어셈블리 코드로 커널 진입점을 설정합니다.
#[naked]
#[no_mangle]
pub extern "C" fn start() -> ! {
    unsafe {
        asm!(
            "cli",                 // 인터럽트 비활성화
            "call {kernel_main}",  // Rust 커널의 진입점 호출
            kernel_main = sym kernel_main,
            options(noreturn)
        );
    }
}

/// 패닉 핸들러 (필수)
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
