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
    subq $40, %rsp

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

    movq -48(%rbp), %rdi
    call aic_main
    cmpq %rsp, (%rsp)
    jne .Laic_probe_failed
    cmpq 8(%rsp), %rbp
    jne .Laic_probe_rbp_failed
    movq -56(%rbp), %r10
    movq %rax, (%r10)

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
