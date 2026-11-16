# x86-64 interrupt and exception entry/exit, and the entry into ring 3.
#
# Every IDT vector has a small stub that pushes an error code (0 if the CPU
# did not push one) and the vector number, then jumps to trap_common. The
# CPU has already pushed SS, RSP, RFLAGS, CS, RIP; for a trap from ring 3 it
# switched to TSS.rsp0 first, so the frame always lands on a kernel stack.
#
# TrapFrame (lowest address first, see trap.rs):
#   r15 r14 r13 r12 r11 r10 r9 r8 rbp rdi rsi rdx rcx rbx rax  (15 x 8)
#   vector error rip cs rflags rsp ss                           (7 x 8)
#
# On the way back to ring 3, TSS.rsp0 is set to the top of this task's
# kernel stack (the address just above the CPU's frame), so the next trap
# from that task lands on its own stack: the x86 counterpart of RISC-V's
# sscratch trick.

    .section .text

    .macro ISR_NOERR n
    .globl isr\n
isr\n:
    push 0
    push \n
    jmp trap_common
    .endm

    .macro ISR_ERR n
    .globl isr\n
isr\n:
    push \n
    jmp trap_common
    .endm

    ISR_NOERR 0
    ISR_NOERR 1
    ISR_NOERR 2
    ISR_NOERR 3
    ISR_NOERR 4
    ISR_NOERR 5
    ISR_NOERR 6
    ISR_NOERR 7
    ISR_ERR   8
    ISR_NOERR 9
    ISR_ERR   10
    ISR_ERR   11
    ISR_ERR   12
    ISR_ERR   13
    ISR_ERR   14
    ISR_NOERR 15
    ISR_NOERR 16
    ISR_ERR   17
    ISR_NOERR 18
    ISR_NOERR 19
    ISR_NOERR 20
    ISR_ERR   21
    ISR_NOERR 22
    ISR_NOERR 23
    ISR_NOERR 24
    ISR_NOERR 25
    ISR_NOERR 26
    ISR_NOERR 27
    ISR_NOERR 28
    ISR_ERR   29
    ISR_ERR   30
    ISR_NOERR 31
    ISR_NOERR 32
    ISR_NOERR 33
    ISR_NOERR 34
    ISR_NOERR 35
    ISR_NOERR 36
    ISR_NOERR 37
    ISR_NOERR 38
    ISR_NOERR 39
    ISR_NOERR 40
    ISR_NOERR 41
    ISR_NOERR 42
    ISR_NOERR 43
    ISR_NOERR 44
    ISR_NOERR 45
    ISR_NOERR 46
    ISR_NOERR 47
    ISR_NOERR 128

trap_common:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov rdi, rsp
    cld
    call trap_handler

    .globl __trap_return
__trap_return:
    test qword ptr [rsp + 144], 3     # saved CS: returning to ring 3?
    jz 1f
    lea rax, [rsp + 176]              # just above the CPU frame = kernel stack top
    mov qword ptr [rip + MICROOS_TSS + 4], rax
1:
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax
    add rsp, 16                       # vector and error code
    iretq

# __enter_user(entry: rdi, user_sp: rsi, arg: rdx) -> !
# Leave the calling kernel task for ring 3 at `entry` on `user_sp`, with
# `arg` in rdi. Interrupts stay off until iretq loads RFLAGS (IF = 1).
    .globl __enter_user
__enter_user:
    cli
    mov qword ptr [rip + MICROOS_TSS + 4], rsp   # kernel stack for traps from ring 3
    and rsi, -16
    sub rsi, 8                        # as if `entry` had been called
    push 0x1B                         # SS: user data, RPL 3
    push rsi                          # RSP
    push 0x202                        # RFLAGS: IF = 1
    push 0x23                         # CS: user code, RPL 3
    push rdi                          # RIP
    mov rdi, rdx
    xor eax, eax
    xor ebx, ebx
    xor ecx, ecx
    xor edx, edx
    xor esi, esi
    xor ebp, ebp
    xor r8d, r8d
    xor r9d, r9d
    xor r10d, r10d
    xor r11d, r11d
    iretq

    .section .rodata
    .align 8
    .globl microos_isr_table
microos_isr_table:
    .irp n, 0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47
    .quad isr\n
    .endr
    .globl microos_isr_syscall
microos_isr_syscall:
    .quad isr128

