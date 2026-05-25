.PHONY: all clean run FORCE

all: os.img

os.img: kernel.bin bootloader/boot.asm bootloader/stage2.asm
	nasm -f bin bootloader/boot.asm -o bootloader/boot.bin
	nasm -f bin bootloader/stage2.asm -o bootloader/stage2.bin
	cat bootloader/boot.bin bootloader/stage2.bin kernel.bin > os.img

kernel.bin: FORCE
	cd kernel && cargo build --release
	rust-objcopy -I elf64-x86-64 -O binary kernel/target/x86_64-os/release/os kernel.bin

run: os.img
	qemu-system-x86_64 -drive format=raw,file=os.img -serial stdio -vga std

clean:
	rm -f os.img kernel.bin bootloader/boot.bin bootloader/stage2.bin
	cd kernel && cargo clean

FORCE:
