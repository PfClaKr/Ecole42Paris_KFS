Yugeon CHUN ([@ychun42](https://profile.intra.42.fr/users/ychun)), Chaehun SONG ([@schaehun42](https://profile.intra.42.fr/users/schaehun))

# KFS - Kernel From Scratch

KFS is a project developed as part of the advanced system programming curriculum at Ecole 42. The goal is to build a complete 32-bit Kernel From Scratch using the Rust programming language and run it on a virtualized i386 architecture.

## Project Objective

The objective of this project is to gain a deep understanding of low-level system operations by implementing a minimal but functional kernel. KFS demonstrates core OS concepts such as memory management, interrupt handling, and terminal I/O in a secure and modern programming environment.

## Technologies Used

- **Rust** – Safe and efficient systems programming language
- **QEMU** – Hardware virtualization emulator
- **GRUB** – Bootloader used to load the kernel
- **KVM** – Kernel-based Virtual Machine for performance

## Features

- Custom kernel written in Rust
- Multistage boot process using GRUB
- Implementation of GDT (Global Descriptor Table)
- Implementation of IDT (Interrupt Descriptor Table)
- Basic shell with custom commands
- Multiple virtual terminals with switch support
- Keyboard input handling
- Dynamic heap allocation with custom memory manager

## Project Structure

The project is divided into four progressive stages:

### KFS-1 (2024/09/09)
- Initial Rust environment setup
- Create a bare minimum kernel that boots via GRUB
- Basic VGA text output
- Add keyboard input support

### KFS-2 (2024/09/19)
- Setup GDT and IDT
- Create interrupt handlers
- Start building a simple interactive shell

### KFS-3 (2024/10/24)
- Implement physical and virtual memory management
- Create a heap allocator for dynamic memory

### KFS-4 (2024/11/22)
- Enhance terminal management
- Extend the shell with more commands

![image](https://github.com/user-attachments/assets/d9edcfb1-6da7-4991-8ebe-c9ca34bc3602)

## Requirements
Check CPU virtualization compatibility
```sh
$> kvm-ok
INFO: /dev/kvm exists
KVM acceleration can be use
```
Rust Cargo
```sh
$> cargo version
```
## Usage
```sh
$> make
$> make run
```
