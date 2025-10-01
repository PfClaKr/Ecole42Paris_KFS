Yugeon CHUN ([@ychun42](https://profile.intra.42.fr/users/ychun)), Chaehun SONG ([@schaehun42](https://profile.intra.42.fr/users/schaehun))

# KFS (Kernel From Scratch) - Custom Operating System in Rust

## About

**KFS (Kernel From Scratch)** is an operating system development project at École 42 that involves building a functional kernel using Rust. This multi-stage project explores low-level systems programming, hardware interaction, and operating system architecture by creating a bootable kernel with progressively more advanced features across four distinct stages.

The project leverages Rust's safety guarantees and modern language features while working at the bare metal level, demonstrating how modern systems programming can benefit from memory safety without sacrificing performance or low-level control.

![image](https://github.com/user-attachments/assets/d9edcfb1-6da7-4991-8ebe-c9ca34bc3602)

## Learning Objectives

- **Operating System Architecture**: Understanding kernel design and OS fundamentals
- **Rust Systems Programming**: Using Rust for bare-metal and kernel development
- **Hardware Interaction**: Direct communication with CPU, memory, and peripherals
- **Boot Process**: Understanding bootloader operation and kernel initialization
- **Memory Management**: Implementing physical and virtual memory systems
- **Interrupt Handling**: Managing hardware and software interrupts
- **Shell Development**: Building interactive command-line interfaces
- **Terminal Management**: Creating robust terminal I/O systems

## Project Timeline and Progression

### KFS-1 (September 9, 2024)
Foundation stage focusing on basic kernel boot and output

### KFS-2 (September 19, 2024)
System tables and interrupt infrastructure implementation

### KFS-3 (October 24, 2024)
Memory management subsystem development

### KFS-4 (November 22, 2024)
Enhanced user interaction and shell functionality

---

## KFS-1: Initial Kernel Boot and Basic I/O

### Stage Overview
The first stage establishes the foundation by setting up a Rust development environment for kernel programming and creating a minimal bootable kernel with basic input/output capabilities.

### Core Features

#### Rust Environment Setup
- **Bare Metal Rust**: Configuring Rust for `no_std` environment
- **Target Specification**: Custom target for x86 kernel development
- **Build Configuration**: Setting up cargo and build scripts for kernel
- **Dependency Management**: Managing kernel-specific Rust crates
- **Panic Handler**: Implementing custom panic behavior for kernel

#### GRUB Bootloader Integration
- **Multiboot Compliance**: Creating multiboot-compliant kernel header
- **GRUB Configuration**: Setting up GRUB to load the kernel
- **Kernel Binary Format**: Generating properly formatted kernel executable
- **Boot Information**: Accessing bootloader-provided system information

#### VGA Text Mode Output
- **VGA Buffer Access**: Direct memory-mapped VGA buffer manipulation
- **Text Rendering**: Character and color attribute writing
- **Screen Management**: Cursor positioning and screen clearing
- **Color Support**: 16-color foreground and background support
- **Print Macros**: Rust-style formatting macros for kernel output

#### Keyboard Input Support
- **PS/2 Keyboard Driver**: Implementing keyboard controller interface
- **Scancode Handling**: Processing keyboard scancodes
- **Character Mapping**: Converting scancodes to ASCII characters
- **Special Key Handling**: Managing modifier keys and special characters

### Key Achievements
- Successfully booting custom Rust kernel via GRUB
- Establishing basic I/O capabilities for development
- Creating foundation for interactive kernel development
- Implementing safe Rust wrappers for hardware access

### Skills Demonstrated
- **Rust for Systems**: Using Rust in bare-metal environment
- **Bootloader Integration**: Working with GRUB multiboot protocol
- **Hardware Programming**: Direct VGA and keyboard hardware access
- **Memory Safety**: Maintaining Rust safety guarantees at kernel level

---

## KFS-2: System Tables and Interrupt Infrastructure

### Stage Overview
Building upon KFS-1, this stage implements critical system tables and interrupt handling infrastructure, enabling the kernel to respond to hardware events and manage CPU execution.

### Core Features

#### Global Descriptor Table (GDT)
- **Segment Descriptors**: Defining code and data segments
- **GDT Structure**: Creating and loading GDT in memory
- **Segment Selectors**: Configuring segment register values
- **Protected Mode**: Proper protected mode segment setup

#### Interrupt Descriptor Table (IDT)
- **IDT Entries**: 256 interrupt descriptor entries
- **Gate Descriptors**: Interrupt and trap gate configuration
- **IDT Loading**: Loading IDT register with table address
- **Handler Registration**: Mapping interrupts to handler functions
- **Exception Handling**: CPU exception interrupt setup

#### Interrupt Handlers
- **Exception Handlers**: CPU fault and exception handling
- **Hardware Interrupts**: Timer, keyboard, and other device interrupts
- **Handler Dispatch**: Routing interrupts to appropriate handlers
- **Context Preservation**: Saving and restoring CPU state
- **Error Handling**: Managing interrupt-related errors safely

#### Interactive Shell Foundation
- **Command Parser**: Basic command-line parsing logic
- **Command Execution**: Framework for executing shell commands
- **Basic Commands**: Initial set of simple shell commands
- **User Interaction**: Processing keyboard input for shell

### Key Achievements
- Establishing robust interrupt handling system
- Creating foundation for hardware event processing
- Building interactive shell infrastructure
- Implementing safe interrupt handler abstractions in Rust

### Skills Demonstrated
- **Interrupt Management**: Understanding x86 interrupt architecture
- **System Tables**: GDT and IDT configuration and management
- **Event-Driven Programming**: Handling asynchronous hardware events
- **Shell Development**: Building interactive command interfaces

---

## KFS-3: Memory Management Subsystem

### Stage Overview
This stage implements sophisticated memory management, including physical memory tracking, virtual memory with paging, and dynamic memory allocation through a heap allocator.

### Core Features

#### Physical Memory Management
- **Memory Detection**: Reading available physical memory from bootloader
- **Frame Allocator**: Managing physical memory in page-sized frames
- **Bitmap Allocator**: Efficient frame allocation tracking
- **Memory Regions**: Managing reserved and available memory areas
- **Frame Allocation/Deallocation**: Allocating and freeing physical pages

#### Virtual Memory Management
- **Paging Implementation**: x86 page table setup and management
- **Page Table Hierarchy**: Managing multi-level page tables
- **Address Translation**: Virtual to physical address mapping
- **Page Mapping**: Mapping virtual addresses to physical frames
- **Memory Isolation**: Separate address spaces foundation

#### Heap Allocator
- **Dynamic Allocation**: Implementing kernel heap for dynamic memory
- **GlobalAlloc Trait**: Rust's allocator trait implementation
- **Allocation Strategies**: Efficient memory allocation algorithms
- **Memory Safety**: Leveraging Rust's ownership for safe allocation
- **Heap Management**: Growing and managing heap memory space

#### Memory Operations
- **Box Support**: Enabling Rust standard collections
- **Memory Debugging**: Tracking allocations and detecting leaks
- **Memory Statistics**: Monitoring memory usage and availability

### Key Achievements
- Implementing complete memory management subsystem
- Enabling dynamic memory allocation in kernel
- Supporting Rust's allocation-dependent features
- Creating foundation for process memory management

### Skills Demonstrated
- **Memory Management**: Advanced memory allocation and paging
- **Rust Allocators**: Implementing Rust's allocator interfaces
- **Data Structures**: Complex kernel data structure implementation
- **Performance**: Efficient memory management strategies

---

## KFS-4: Enhanced Terminal and Extended Shell

### Stage Overview
The final stage enhances user interaction by improving terminal management and extending shell functionality with additional commands and features.

### Core Features

#### Enhanced Terminal Management
- **Terminal Abstraction**: Clean interface for terminal operations
- **Buffer Management**: Improved screen buffer handling
- **Scrolling**: Automatic screen scrolling
- **Terminal State**: Managing cursor, colors, and display state
- **Input History**: Command history and recall functionality

#### Extended Shell Commands
- **System Information**: Commands to display kernel/system info
- **Memory Commands**: Displaying memory statistics and usage
- **Process Commands**: Basic process information and management
- **Utility Commands**: Clear, help, echo, and other utilities

#### Command Infrastructure
- **Command Registry**: Organized command registration system
- **Argument Parsing**: Robust command argument handling
- **Help System**: Built-in documentation for commands
- **Error Handling**: User-friendly error messages

#### User Experience Improvements
- **Colored Output**: Using colors for different output types
- **Formatted Display**: Structured and readable command output
- **Interactive Feedback**: Real-time feedback for user actions
- **Error Recovery**: Graceful handling of invalid commands

### Key Achievements
- Creating polished, user-friendly terminal interface
- Implementing comprehensive shell command set
- Establishing foundation for future OS features
- Demonstrating complete, functional kernel environment

### Skills Demonstrated
- **User Interface Design**: Creating intuitive command-line interfaces
- **Software Architecture**: Well-organized command system
- **User Experience**: Focus on usability and feedback
- **Code Organization**: Clean, maintainable codebase structure

---

## Overall Project Impact

### Rust for Kernel Development

The KFS project demonstrates Rust's viability for operating system development:
- **Memory Safety**: Preventing common kernel bugs through Rust's ownership system
- **Zero-Cost Abstractions**: High-level code without performance penalty
- **Modern Language**: Leveraging modern language features for kernel code
- **Type Safety**: Compile-time guarantees reducing runtime errors

### Progressive Development Approach

The four-stage structure provides systematic skill building:
- **Stage 1**: Foundation and basic I/O
- **Stage 2**: System infrastructure and interrupts
- **Stage 3**: Memory management complexity
- **Stage 4**: User-facing features and polish

### Comprehensive Learning Experience

The project covers essential OS concepts:
- **Hardware to Software**: From boot to user interaction
- **Theory to Practice**: Implementing textbook OS concepts
- **Problem Solving**: Debugging low-level kernel issues
- **Systems Integration**: Connecting kernel components

## Development and Testing

### Development Environment
- **Rust Toolchain**: Nightly Rust with custom target support
- **Build System**: Cargo with custom build scripts
- **Cross Compilation**: Building for x86 target from any platform
- **Version Control**: Git-based development workflow

### Testing Approaches
- **QEMU Emulation**: Primary development and testing platform
- **Real Hardware**: Final validation on physical machines
- **Debug Output**: Screen logging

### Debugging Techniques
- **GDB Integration**: Debugging with QEMU and GDB
- **Print Debugging**: Strategic kernel print statements
- **Memory Analysis**: Examining memory contents and layout
- **Panic Messages**: Detailed panic output for error diagnosis

## Requirements
Check CPU virtualization compatibility
```sh
$> kvm-ok
INFO: /dev/kvm exists
KVM acceleration can be use
```
Check Rust Cargo
```sh
$> cargo version
```

## Usage
```sh
$> make
$> make run
```

## Skills Demonstrated

### Technical Competencies
- **Rust Programming**: Expert-level Rust for systems programming
- **Operating Systems**: Deep understanding of OS internals
- **Computer Architecture**: Knowledge of x86 architecture
- **Memory Management**: Advanced memory allocation and paging
- **Hardware Interface**: Direct hardware programming
- **Interrupt Handling**: Managing asynchronous hardware events

### Modern Development Practices
- **Safe Systems Programming**: Memory-safe kernel development
- **Type Safety**: Leveraging strong type systems for correctness
- **Modern Tooling**: Using contemporary development tools
- **Code Organization**: Clean, modular kernel architecture

## Real-World Applications

The knowledge gained from KFS is directly applicable to:
- **Operating System Development**: Contributing to Rust-based OS projects
- **Embedded Systems**: Firmware development
- **Systems Programming**: Low-level systems software
- **Driver Development**: Hardware drivers
- **Security-Critical Systems**: Memory-safe systems development
- **IoT Firmware**: Resource-constrained embedded systems

## Related Projects and Technologies

### Career Opportunities
- **Systems Engineer**: Systems programming roles
- **Embedded Developer**: Firmware and embedded development
- **OS Developer**: Operating system and kernel engineering
- **Security Engineer**: Memory-safe systems development
- **Rust Engineer**: Specialized Rust development positions

## Project Evolution and Completion

### Completion Status
The project progressed through all four stages over approximately 4 months, demonstrating steady advancement through increasingly complex operating system concepts while building a functional, interactive kernel environment.

## Notes

The KFS project represents a modern approach to operating system education, combining traditional kernel development concepts with Rust's memory safety guarantees. It demonstrates that systems programming can benefit from modern language features without sacrificing performance or low-level control.

The progressive four-stage structure allowed systematic mastery of kernel development, from basic boot and I/O through sophisticated memory management to a polished user interface. Each stage built essential foundation for the next while producing tangible, testable functionality.

Completing KFS in Rust showcases not only deep systems programming knowledge but also the ability to work with cutting-edge systems programming languages and paradigms, positioning the developer at the forefront of modern operating system development.

---

*Developed as part of the École 42 curriculum - building a modern kernel from scratch using Rust for safe systems programming.*
