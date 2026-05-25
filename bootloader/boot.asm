BITS 16
ORG 0x7C00
DEFAULT ABS

%define STAGE2_SECTORS 16

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

hang:
    hlt
    jmp hang

; --- SECTION DES DONNEES ET STRUCTURES ---
dap_stage2:
    db 0x10
    db 0x00
    dw STAGE2_SECTORS
    dw 0x0000
    dw 0x2000
    dq 0x0000000000000001

; Signature de boot magique (Parfaitement calibrée)
times 510-($-$$) db 0
dw 0xAA55

