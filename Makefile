.PHONY: all clean run tools run-gui FORCE

all: os.img

BOOT_MANIFEST = bootloader/boot_manifest.bin
KERNEL_SECTORS_INC = bootloader/kernel_sectors.inc
PAYLOAD_BIN = kernel/resources/bad_apple.bin

os.img: kernel.bin bootloader/boot.bin bootloader/stage2.bin
	cargo run --manifest-path tools/image_builder/Cargo.toml --release

$(KERNEL_SECTORS_INC): kernel.bin
	cargo run --manifest-path tools/image_builder/Cargo.toml --release -- kernel-sectors

bootloader/boot.bin: bootloader/boot.asm
	nasm -f bin bootloader/boot.asm -o bootloader/boot.bin

bootloader/stage2.bin: bootloader/stage2.asm $(KERNEL_SECTORS_INC)
	nasm -f bin bootloader/stage2.asm -o bootloader/stage2.bin

kernel.bin: FORCE
	nasm -f elf64 kernel/kernel_entry.asm -o kernel/kernel_entry.o
	cd kernel && cargo build --release
	rust-objcopy -O binary kernel/target/x86_64-os/release/os kernel.bin

run: os.img
	qemu-system-x86_64 -drive format=raw,file=os.img -serial stdio -vga std

run-gui: FORCE
	qemu-system-x86_64 -drive format=raw,file=os.img -serial stdio -vga std

tools: FORCE
	cd tools/badapple_converter && cargo run --release
	cd tools/fonts_builder && cargo run --release
	@echo "Bad Apple binary generated at $(PAYLOAD_BIN)"

clean:
	rm -f os.img kernel.bin 
	rm -f bootloader/boot.bin 
	rm -f bootloader/stage2.bin 
	rm -f kernel/kernel_entry.o 
	rm -f bootloader/boot_manifest.bin 
	rm -f bootloader/kernel_sectors.inc
	cd kernel && cargo clean
