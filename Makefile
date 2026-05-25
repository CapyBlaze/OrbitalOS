.PHONY: all clean run FORCE

all: os.img

os.img: kernel.bin bootloader/boot.bin bootloader/stage2.bin
	dd if=/dev/zero of=os.img bs=512 count=2048 2>/dev/null
	dd if=bootloader/boot.bin of=os.img bs=512 seek=0 conv=notrunc 2>/dev/null
	dd if=bootloader/stage2.bin of=os.img bs=512 seek=1 conv=notrunc 2>/dev/null
	dd if=kernel.bin of=os.img bs=512 seek=5 conv=notrunc 2>/dev/null

bootloader/boot.bin: bootloader/boot.asm
	nasm -f bin bootloader/boot.asm -o bootloader/boot.bin

bootloader/stage2.bin: bootloader/stage2.asm
	nasm -f bin bootloader/stage2.asm -o bootloader/stage2.bin

kernel.bin: FORCE
	nasm -f elf64 kernel/kernel_entry.asm -o kernel/kernel_entry.o
	cd kernel && cargo build --release
	rust-objcopy -O binary kernel/target/x86_64-os/release/os kernel.bin

run: os.img
	qemu-system-x86_64 -drive format=raw,file=os.img -serial stdio -vga std

clean:
	rm -f os.img kernel.bin bootloader/boot.bin bootloader/stage2.bin kernel/kernel_entry.o
	cd kernel && cargo clean
