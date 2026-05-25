[BITS 64]

global kernel_entry
extern _start

section .text

kernel_entry:
    cli

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov rsp, 0x90000
    mov rdi, 0x7000

    mov byte [0xb8000], 'X'
    mov byte [0xb8001], 0x0f

    mov rax, _start
    call rax

hang:
    hlt
    jmp hang