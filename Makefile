TARGET = i386-unknown-none
KERNEL = KFS.bin
ISO = kfs.iso

RUSTC = cargo
OBJCOPY = objcopy

LD = ld
LDFLAGS= -n -nostdlib -m elf_i386

all: $(ISO)

$(ISO): $(KERNEL)
	mkdir -p iso/boot/grub
	cp target/$(TARGET)/release/KFS iso/boot/kfs.bin
	cp grub.cfg iso/boot/grub/
	grub-mkrescue -o $(ISO) iso

$(KERNEL): kfs
	$(LD) $(LDFLAGS) -T ld-scripts/x86.ld -o $@ $^

kfs:
	$(RUSTC) build -Zbuild-std=core,alloc --release --target=$(TARGET).json
	$(OBJCOPY) -v -O binary --binary-architecture=i386 target/$(TARGET)/release/KFS $@

run:
	qemu-system-i386 -cdrom $(ISO) -no-reboot

clean:
	cargo clean
	rm -f *.o $(KERNEL) $(ISO)
	rm -rf iso

re: clean all

