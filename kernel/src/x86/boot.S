# x86-64 entry: GRUB (Multiboot2) -> 32-bit protected mode -> long mode -> kmain.
#
# GRUB loads the kernel ELF at 1 MiB and jumps to _start in 32-bit protected
# mode with paging off, eax = Multiboot2 magic and ebx = boot information.
# Long mode requires paging, so this stub identity-maps the first 1 GiB with
# 2 MiB pages, enables PAE + long mode + paging, loads a 64-bit GDT and
# jumps to 64-bit code, which zeroes .bss and calls kmain(magic, info).
#
# The pages are marked user-accessible (U/S = 1) because U-mode programs run
# from the kernel image; as on the RISC-V port, this is privilege separation
# (ring 3 cannot run privileged instructions), not memory isolation.


    .section .multiboot2, "a"
    .align 8
mb2_header:
    .long 0xE85250D6                  # Multiboot2 magic
    .long 0                           # architecture: i386 (protected mode)
    .long mb2_header_end - mb2_header
    .long 0x100000000 - (0xE85250D6 + (mb2_header_end - mb2_header))
    .align 8
    .short 5                          # framebuffer tag: ask GRUB for a graphics mode
    .short 1                          # optional: fall back to VGA text if unavailable
    .long 20
    .long 0                           # width: no preference, use GRUB's gfxmode list
    .long 0                           # height: no preference
    .long 32                          # bits per pixel
    .align 8
    .short 0                          # end tag
    .short 0
    .long 8
mb2_header_end:

    .section .text.boot32, "ax"
    .code32
    .globl _start
_start:
    cli
    mov esp, offset boot_stack_top
    mov edi, eax                      # keep magic and info for kmain
    mov esi, ebx

    # PML4[0] -> PDPT, PDPT[0..3] -> four PDs, PD entry i -> 2 MiB page i
    # (present, writable, user, huge): an identity map of the first 4 GiB,
    # which also covers the framebuffer GRUB sets up (often near 3-4 GiB).
    mov eax, offset boot_pdpt
    or eax, 0x7
    mov dword ptr [boot_pml4], eax
    xor ecx, ecx
0:
    mov eax, ecx
    shl eax, 12
    add eax, offset boot_pd
    or eax, 0x7
    mov dword ptr [boot_pdpt + ecx*8], eax
    inc ecx
    cmp ecx, 4
    jne 0b
    xor ecx, ecx
1:
    mov eax, ecx
    shl eax, 21
    or eax, 0x87
    mov dword ptr [boot_pd + ecx*8], eax
    inc ecx
    cmp ecx, 2048
    jne 1b

    mov eax, offset boot_pml4
    mov cr3, eax
    mov eax, cr4
    or eax, 0x20                      # CR4.PAE
    mov cr4, eax
    mov ecx, 0xC0000080               # EFER
    rdmsr
    or eax, 0x100                     # EFER.LME
    wrmsr
    mov eax, cr0
    or eax, 0x80000001                # CR0.PG | CR0.PE
    mov cr0, eax

    lgdt [boot_gdt_ptr]
    mov eax, offset long_mode
    push 0x08                         # far return into the 64-bit code segment
    push eax
    retf

    .code64
long_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    xor eax, eax
    mov fs, ax
    mov gs, ax
    mov rsp, offset boot_stack_top
    mov edi, edi                      # zero-extend magic and info pointer
    mov esi, esi

    push rdi
    push rsi
    lea rdi, [rip + __bss_start]
    lea rcx, [rip + __bss_end]
    sub rcx, rdi
    xor eax, eax
    rep stosb
    pop rsi
    pop rdi

    xor ebp, ebp                      # null frame pointer ends backtraces
    call kmain
2:
    hlt
    jmp 2b

    .section .data.boot, "aw"
    .align 4096
boot_pml4:
    .zero 4096
boot_pdpt:
    .zero 4096
boot_pd:
    .zero 4096 * 4

    .align 16
boot_gdt:
    .quad 0
    .quad 0x00AF9A000000FFFF          # 0x08: 64-bit kernel code
    .quad 0x00CF92000000FFFF          # 0x10: kernel data
boot_gdt_end:
boot_gdt_ptr:
    .short boot_gdt_end - boot_gdt - 1
    .quad boot_gdt

    .section .bss.boot_stack, "aw", @nobits
    .align 16
boot_stack:
    .skip 65536
boot_stack_top:

