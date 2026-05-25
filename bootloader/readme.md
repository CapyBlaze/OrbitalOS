cargo build --release
rust-objcopy -I elf64-x86-64 -O binary target/x86_64-os/release/os kernel.bin


nasm -f bin boot.asm -o boot.bin
cat boot.bin kernel.bin > os.img
qemu-system-x86_64 -drive format=raw,file=os.img