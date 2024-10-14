TARGET = i386-unknown-none
KERNEL = KFS.bin
ISO = kfs.iso
QEMU = qemu-system-i386

RUSTC = cargo

LD = ld
LDFLAGS= -n -nostdlib -m elf_i386

all: $(ISO)

$(ISO): kfs
	mkdir -p iso/boot/grub
	cp target/$(TARGET)/release/KFS iso/boot/kfs.bin
	cp scripts/grub/grub.cfg iso/boot/grub/
	grub-mkrescue -d arch-i386/grub-i386-pc -o $(ISO) iso

kfs:
	$(RUSTC) build -Zbuild-std=core,alloc --release --target=arch-i386/$(TARGET).json

run:
	$(QEMU) -cdrom $(ISO) -no-reboot -d int

debug-run:
	$(QEMU) -s -S -cdrom $(ISO) -no-reboot -d int,cpu_reset
#	gdb -x scripts/debug/debug.gdb target/i386-unknown-none/release/KFS

clean:
	cargo clean
	rm -f *.o $(ISO) kfs
	rm -rf iso

re: clean all

.PONEY : debug