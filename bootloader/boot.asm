[BITS 16]
[ORG 0x7C00]

start:
    cld                        
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00


    mov byte [0x500], dl

    mov si, dap_stage2
    mov dl, [0x500]
    mov ah, 0x42
    int 0x13

    js disk_error

    jmp 0x7E00

disk_error:
    mov ah, 0x0E
    mov al, 'E'
    int 0x10

hang:
    hlt
    jmp hang

dap_stage2:
    db 0x10
    db 0x00
    dw 0x3 ; a peut etre augmenter nombre de secteur du stage 2
    dw 0x7E00
    dw 0x0000
    dq 0x1

times 510 - ($ - $$) db 0
dw 0xAA55
