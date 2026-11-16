# void __switch_context(Context *save, const Context *load)   (x86-64)
#
# Cooperative context switch between two kernel stacks. Only the registers
# the System V calling convention says a callee must preserve (rbx, rbp,
# r12-r15, rsp) are saved, plus where to resume (rip): the caller already
# assumes every other register is clobbered by a function call.
#
# Context layout: rip @0, rsp @8, rbx @16, rbp @24, r12 @32, r13 @40,
# r14 @48, r15 @56 (see task/mod.rs).

    .section .text
    .globl __switch_context
__switch_context:
    lea rax, [rip + 1f]
    mov [rdi + 0], rax
    mov [rdi + 8], rsp
    mov [rdi + 16], rbx
    mov [rdi + 24], rbp
    mov [rdi + 32], r12
    mov [rdi + 40], r13
    mov [rdi + 48], r14
    mov [rdi + 56], r15

    mov rsp, [rsi + 8]
    mov rbx, [rsi + 16]
    mov rbp, [rsi + 24]
    mov r12, [rsi + 32]
    mov r13, [rsi + 40]
    mov r14, [rsi + 48]
    mov r15, [rsi + 56]
    jmp qword ptr [rsi + 0]     # continue the loaded context
1:
    ret                         # a saved context resumes here, back in its caller
