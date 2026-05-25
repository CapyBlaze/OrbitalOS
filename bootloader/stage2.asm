BITS 16
ORG 0x20000
DEFAULT ABS

%define STAGE2_SECTORS 16
%define KERNEL_SECTOR (1 + STAGE2_SECTORS)
%define KERNEL_SECTORS 48
%define KERNEL_LOAD_SEG 0x3000

start_stage2:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    ; 1. ACTIVER LE MODE GRAPHIQUE VESA VBE (16-bit)
    ; 0x119 = 1024x768x32bpp, bit 14 pour Linear Framebuffer
    mov ax, 0x4F02
    mov bx, 0x4119
    int 0x10
    cmp ax, 0x004F
    jne hang

    ; Récupérer les informations du mode VBE
    mov ax, 0x4F01
    mov cx, 0x4119
    mov di, 0x8000              ; Stockage temporaire des infos du mode à l'adresse 0x8000
    int 0x10
    cmp ax, 0x004F
    jne hang

    ; Vérifier que le mode supporte LFB et 32bpp
    test word [0x8000 + 0x00], 1 << 7
    jz hang
    cmp byte [0x8000 + 0x19], 32
    jne hang

    ; Sauvegarder l'adresse physique du Framebuffer (offset 0x28) dans l'étiquette en bas
    mov ax, [0x8000 + 0x10]     ; BytesPerScanLine
    mov [fb_pitch], ax
    mov ax, [0x8000 + 0x12]     ; XResolution
    mov [fb_width], ax
    mov ax, [0x8000 + 0x14]     ; YResolution
    mov [fb_height], ax
    mov al, [0x8000 + 0x19]     ; BitsPerPixel
    mov [fb_bpp], al
    mov eax, [dword 0x8000 + 0x28]
    mov [dword fb_physical_address], eax

    ; --- CHARGER LE KERNEL RUST ---
    mov si, dap_kernel
    mov dl, [0x500]
    mov ah, 0x42
    int 0x13
    jc hang

    xor ax, ax
    mov es, ax

    ; Activer la ligne A20
    in al, 0x92
    or al, 2
    out 0x92, al

    ; 2. PASSER EN MODE PROTEGE 32-BIT
    lgdt [gdt32_descriptor]
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp 0x08:init_32bit

BITS 32
init_32bit:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov esp, 0x90000

    ; --- TEST DESSIN EN 32-BIT ---
    mov edi, [fb_physical_address]  ; EDI pointe vers le debut de l'ecran
    mov ecx, 100000
    mov eax, 0x00FF0000
    rep stosd

    ; Deplacer notre kernel de 0x30000 vers sa destination finale 0x100000 (1 Mo)
    mov esi, 0x30000
    mov edi, 0x100000
    mov ecx, 30000
    rep movsd

    ; 3. CONFIGURER LA PAGINATION (Mapping de 4 Go)
    mov edi, 0x1000
    xor eax, eax
    mov ecx, 4096
    rep stosd

    ; PML4[0] pointe vers le PDP
    mov dword [0x1000], 0x2003

    ; On configure 4 PDP qui pointent vers 4 Page Directories (couvre 4 Go au total)
    mov dword [0x2000], 0x3003
    mov dword [0x2008], 0x4003
    mov dword [0x2010], 0x5003
    mov dword [0x2018], 0x6003

    ; Identity Mapping des 4 premiers Go (avec des Huge Pages de 2 Mo)
    mov edi, 0x3000
    mov ebx, 0x00000083
    mov ecx, 2048
.map_pages:
    mov [edi], ebx
    add ebx, 0x200000
    add edi, 8
    loop .map_pages

    mov eax, 0x1000
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    lgdt [gdt64_descriptor]
    jmp 0x08:init_64bit

BITS 64
DEFAULT REL
init_64bit:
    mov rsp, 0x90000
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    ; On passe les arguments directement via les registres a Rust !
    mov rdi, 0x0
    mov rsi, 0

    ; On met la VRAIE adresse brute du Framebuffer dans RDX
    mov edx, dword [fb_physical_address]

    movzx rcx, word [fb_width]
    movzx r8, word [fb_height]
    movzx r9, word [fb_pitch]

    mov rbx, 0x100000
    jmp rbx

hang:
    hlt
    jmp hang

; --- SECTION DES DONNEES ET STRUCTURES ---
dap_kernel:
    db 0x10
    db 0x00
    dw KERNEL_SECTORS
    dw 0x0000
    dw KERNEL_LOAD_SEG
    dq KERNEL_SECTOR

align 4
fb_physical_address: dd 0
fb_pitch: dw 0
fb_width: dw 0
fb_height: dw 0
fb_bpp: db 0

align 8
gdt32:
    dq 0x0000000000000000
    dq 0x00cf9a000000ffff
    dq 0x00cf92000000ffff

gdt32_descriptor:
    dw $ - gdt32 - 1
    dd gdt32

gdt64:
    dq 0x0000000000000000
    dq 0x0020980000000000
    dq 0x0000920000000000

gdt64_descriptor:
    dw $ - gdt64 - 1
    dq gdt64

; Remplissage a un multiple de 512 octets pour lecture par secteurs
align 512
