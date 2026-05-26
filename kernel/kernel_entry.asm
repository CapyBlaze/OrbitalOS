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

    mov byte [0xb8000], 'K'
    mov byte [0xb8001], 0x0f

    mov rdi, 0x7000
    mov rsi, 0x8000
    movzx rdx, word [0x9000]

    mov rax, _start
    call rax

hang:
    hlt
    jmp hang