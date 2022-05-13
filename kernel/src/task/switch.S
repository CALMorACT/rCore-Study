# os/src/task/switch.S

.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch
__switch:
    # 阶段 [1]
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext,
    #     next_task_cx_ptr: *const TaskContext
    # )
    # 阶段 [2]
    # save kernel stack of current task （save sp）
    sd sp, 8(a0)
    # save ra & s0~s11 of current execution （save ra）
    sd ra, 0(a0)
    .set n, 0
    # repeat save s0~s11
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # 阶段 [3]
    # restore ra & s0~s11 of next execution
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # restore kernel stack of next task
    # 入参 第二项 是 next_task_cx_ptr (a1)，则 ra = a1, sp = a1 + 8(8字节，也就是context中的第二个项)
    ld sp, 8(a1)
    # 阶段 [4]
    ret