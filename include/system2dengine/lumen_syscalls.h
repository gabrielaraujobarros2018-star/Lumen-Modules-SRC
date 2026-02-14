#ifndef LUMEN_SYSCALLS_H
#define LUMEN_SYSCALLS_H

// Lumen OS Kernel Syscalls for sys2Dengine v1.0
static inline long lumen_syscall0(long n) {
    register long r0 asm("r0") = n;
    asm volatile("swi #0" : "=r"(r0) : "r"(r0));
    return r0;
}

static inline long lumen_syscall1(long n, long a1) {
    register long r0 asm("r0") = n;
    register long r1 asm("r1") = a1;
    asm volatile("swi #0" : "=r"(r0) : "r"(r0), "r"(r1));
    return r0;
}

// Syscall numbers (Lumen kernel)
#define LUMEN_SYSCALL_FB_MAP      300
#define LUMEN_SYSCALL_FB_UNMAP    301
#define LUMEN_SYSCALL_VSYNC_WAIT  302
#define LUMEN_SYSCALL_FB_SWAP     303
#define LUMEN_SYSCALL_AUDIO_INIT  310
#define LUMEN_SYSCALL_AUDIO_WRITE 311

#endif

