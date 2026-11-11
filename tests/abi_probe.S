.text
.globl aic_probe
.type aic_probe, @function
aic_probe:
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    subq $88, %rsp

    fnstcw 16(%rsp)
    stmxcsr 20(%rsp)

    movq %rdi, -48(%rbp)
    movq %rsi, -56(%rbp)
    movq $0x1122, %rbx
    movq $0x2233, %r12
    movq $0x3344, %r13
    movq $0x4455, %r14
    movq $0x5566, %r15
    movq %rsp, (%rsp)
    movq %rbp, 8(%rsp)
    testq $15, %rsp
    jne .Laic_probe_failed

    # Exercise preserved controls with valid non-default rounding modes.
    movzwl 16(%rsp), %eax
    andw $0xf3ff, %ax
    orw $0x0400, %ax
    movw %ax, 24(%rsp)
    fldcw 24(%rsp)
    movl 20(%rsp), %eax
    andl $0xffff9fff, %eax
    orl $0x4000, %eax
    movl %eax, 28(%rsp)
    ldmxcsr 28(%rsp)
    cld
    movq -48(%rbp), %rdi
    call aic_main
    cmpq %rsp, (%rsp)
    jne .Laic_probe_failed
    cmpq 8(%rsp), %rbp
    jne .Laic_probe_rbp_failed
    movq -56(%rbp), %r10
    movq %rax, (%r10)

    pushfq
    popq %r11
    testq $0x400, %r11
    jne .Laic_probe_failed
    fnstcw 32(%rsp)
    movzwl 32(%rsp), %eax
    cmpw 24(%rsp), %ax
    jne .Laic_probe_failed
    stmxcsr 36(%rsp)
    movl 36(%rsp), %eax
    xorl 28(%rsp), %eax
    testl $0xffc0, %eax
    jne .Laic_probe_failed

    movl $1, %eax
    cmpq $0x1122, %rbx
    jne .Laic_probe_failed
    cmpq $0x2233, %r12
    jne .Laic_probe_failed
    cmpq $0x3344, %r13
    jne .Laic_probe_failed
    cmpq $0x4455, %r14
    jne .Laic_probe_failed
    cmpq $0x5566, %r15
    jne .Laic_probe_failed
    jmp .Laic_probe_done

.Laic_probe_failed:
    xorl %eax, %eax
    jmp .Laic_probe_done

.Laic_probe_rbp_failed:
    movq 8(%rsp), %rbp
    xorl %eax, %eax
.Laic_probe_done:
    fldcw 16(%rsp)
    ldmxcsr 20(%rsp)
    cld
    leaq -40(%rbp), %rsp
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    leave
    ret
.size aic_probe, .-aic_probe
.section .note.GNU-stack,"",@progbits
