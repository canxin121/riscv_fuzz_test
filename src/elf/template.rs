/// 异常转储配置
#[derive(Debug, Clone)]
pub enum DumpException {
    /// 转储所有异常
    All,
    /// 转储指定MEPC地址的异常
    OnMepcMatch(Vec<u64>),
}

/// 寄存器转储配置
#[derive(Debug, Clone)]
pub enum DumpRegister {
    /// 转储所有寄存器
    All,
    /// 转储指定的GPR寄存器列表 (寄存器编号)
    Gpr(Vec<u32>),
    /// 转储指定的FPR寄存器列表 (寄存器编号)
    Fpr(Vec<u32>),
    /// 转储指定的GPR和FPR寄存器
    GprAndFpr { gpr: Vec<u32>, fpr: Vec<u32> },
}

/// 生成完整的RISC-V汇编模板（包含异常转储和寄存器转储）
pub fn generate_standard_asm(user_code: &str) -> String {
    generate_asm(user_code, Some(DumpException::All), Some(DumpRegister::All))
}

pub fn generate_minimal_asm(user_code: &str) -> String {
    generate_asm(user_code, None, None)
}

/// 生成自定义RISC-V汇编模板
pub fn generate_asm(
    user_code: &str, 
    dump_exception: Option<DumpException>, 
    dump_registers: Option<DumpRegister>
) -> String {
    format!(
        "{}{}{}{}",
        get_macro_definitions(),
        get_data_sections(),
        get_exception_handler(&dump_exception),
        get_main_program(user_code, &dump_registers)
    )
}

fn get_macro_definitions() -> &'static str {
    r#"# ============================================================================
# 宏定义 - Macro Definitions
# ============================================================================

# ----------------------------------------------------------------------------
# .macro SAVE_T_REGS / RESTORE_T_REGS
# ----------------------------------------------------------------------------
.macro SAVE_T_REGS save_area_label
    csrw mscratch, t6
    la   t6, \save_area_label
    sd   t0,   0(t6); sd   t1,   8(t6); sd   t2,  16(t6)
    sd   t3,  24(t6); sd   t4,  32(t6); sd   t5,  40(t6)
    csrr t5, mscratch
    sd   t5,  48(t6)
    csrr t6, mscratch
.endm

.macro RESTORE_T_REGS save_area_label
    csrw mscratch, t6
    la   t6, \save_area_label
    ld   t0,   0(t6); ld   t1,   8(t6); ld   t2,  16(t6)
    ld   t3,  24(t6); ld   t4,  32(t6); ld   t5,  40(t6)
    ld   t6,  48(t6)
    csrr t6, mscratch
.endm

# ----------------------------------------------------------------------------
# .macro HTIF_PRINT_RAW
# ----------------------------------------------------------------------------
.macro HTIF_PRINT_RAW data_label, data_size
    la   t0, htif_communication_buffer
    li   t1, 64; sd t1, 0(t0); li   t1, 1;   sd t1, 8(t0)
    la   t1, \data_label; sd t1, 16(t0); li   t1, \data_size;   sd t1, 24(t0)
    fence; la   t1, tohost; sd t0, 0(t1)
wait_htif_print_\@:
    la   t2, fromhost; ld t3, 0(t2); beqz t3, wait_htif_print_\@
    sd   zero, 0(t2); fence
.endm

# ----------------------------------------------------------------------------
# .macro HTIF_PRINT
# ----------------------------------------------------------------------------
.macro HTIF_PRINT temp_save_area, data_label, data_size
    SAVE_T_REGS \temp_save_area
    HTIF_PRINT_RAW \data_label, \data_size
    RESTORE_T_REGS \temp_save_area
.endm

# ----------------------------------------------------------------------------
# .macro DUMP_ALL_REGS_RAW 
# [MODIFIED] - 移除了可选和可能引起问题的CSRs
# ----------------------------------------------------------------------------
.macro DUMP_ALL_REGS_RAW
    csrw mscratch, t6
    la t6, register_dump_buffer

    # 转储所有通用寄存器 (x0-x31)
    sd  x0,    0(t6); sd  x1,    8(t6); sd  x2,   16(t6); sd  x3,   24(t6)
    sd  x4,   32(t6); sd  x5,   40(t6); sd  x6,   48(t6); sd  x7,   56(t6)
    sd  x8,   64(t6); sd  x9,   72(t6); sd x10,   80(t6); sd x11,   88(t6)
    sd x12,   96(t6); sd x13,  104(t6); sd x14,  112(t6); sd x15,  120(t6)
    sd x16,  128(t6); sd x17,  136(t6); sd x18,  144(t6); sd x19,  152(t6)
    sd x20,  160(t6); sd x21,  168(t6); sd x22,  176(t6); sd x23,  184(t6)
    sd x24,  192(t6); sd x25,  200(t6); sd x26,  208(t6); sd x27,  216(t6)
    sd x28,  224(t6); sd x29,  232(t6); sd x30,  240(t6)
    csrr t5, mscratch
    sd t5, 248(t6)  # x31 (t6) 的原始值

    # 转储核心的机器模式 CSR 寄存器
    csrr t0, mstatus;     sd t0, 256(t6) # 机器状态
    csrr t0, misa;        sd t0, 264(t6) # ISA 和扩展
    csrr t0, medeleg;     sd t0, 272(t6) # 机器异常委托
    csrr t0, mideleg;     sd t0, 280(t6) # 机器中断委托
    csrr t0, mie;         sd t0, 288(t6) # 机器中断使能
    csrr t0, mtvec;       sd t0, 296(t6) # 机器陷阱向量基地址
    csrr t0, mcounteren;  sd t0, 304(t6) # 机器计数器使能
    
    # 机器陷阱处理 CSRs
    csrr t0, mscratch;    sd t0, 312(t6) # 机器临时寄存器
    csrr t0, mepc;        sd t0, 320(t6) # 机器异常程序计数器
    csrr t0, mcause;      sd t0, 328(t6) # 机器陷阱原因
    csrr t0, mtval;       sd t0, 336(t6) # 机器坏地址或指令
    csrr t0, mip;         sd t0, 344(t6) # 机器中断挂起

    # 机器计数器/计时器 CSRs
    csrr t0, mcycle;      sd t0, 352(t6) # 机器周期计数器
    csrr t0, minstret;    sd t0, 360(t6) # 机器指令退役计数器
    
    # 机器信息 CSRs
    csrr t0, mvendorid;   sd t0, 368(t6) # 厂商ID
    csrr t0, marchid;     sd t0, 376(t6) # 架构ID
    csrr t0, mimpid;      sd t0, 384(t6) # 实现ID
    csrr t0, mhartid;     sd t0, 392(t6) # 硬件线程ID

    .set DUMP_SIZE_NO_FP, 400

#if __riscv_flen > 0
    # F/D 扩展存在: 转储浮点寄存器和状态
    csrr t0, mstatus
    li   t1, (1 << 13) # MSTATUS_FS_INITIAL
    or   t1, t0, t1
    csrw mstatus, t1

    # 浮点控制和状态寄存器
    csrr t1, fcsr;      sd t1, 400(t6)
    
    # 浮点寄存器 (f0-f31)
    fsd f0,   408(t6); fsd f1,   416(t6); fsd f2,   424(t6); fsd f3,   432(t6)
    fsd f4,   440(t6); fsd f5,   448(t6); fsd f6,   456(t6); fsd f7,   464(t6)
    fsd f8,   472(t6); fsd f9,   480(t6); fsd f10,  488(t6); fsd f11,  496(t6)
    fsd f12,  504(t6); fsd f13,  512(t6); fsd f14,  520(t6); fsd f15,  528(t6)
    fsd f16,  536(t6); fsd f17,  544(t6); fsd f18,  552(t6); fsd f19,  560(t6)
    fsd f20,  568(t6); fsd f21,  576(t6); fsd f22,  584(t6); fsd f23,  592(t6)
    fsd f24,  600(t6); fsd f25,  608(t6); fsd f26,  616(t6); fsd f27,  624(t6)
    fsd f28,  632(t6); fsd f29,  640(t6); fsd f30,  648(t6); fsd f31,  656(t6)

    .set DUMP_SIZE_WITH_FP, 664

    csrw mstatus, t0

    HTIF_PRINT_RAW full_reg_dump_prefix_with_fp, 8
    HTIF_PRINT_RAW register_dump_buffer, DUMP_SIZE_WITH_FP
#else
    HTIF_PRINT_RAW full_reg_dump_prefix_no_fp, 8
    HTIF_PRINT_RAW register_dump_buffer, DUMP_SIZE_NO_FP
#endif
    csrw mscratch, zero
.endm

# ----------------------------------------------------------------------------
# .macro DUMP_ALL_REGS
# ----------------------------------------------------------------------------
.macro DUMP_ALL_REGS temp_save_area
    SAVE_T_REGS \temp_save_area
    DUMP_ALL_REGS_RAW
    RESTORE_T_REGS \temp_save_area
.endm

# ----------------------------------------------------------------------------
# .macro DUMP_GPR_RAW / DUMP_FPR_RAW
# ----------------------------------------------------------------------------
.macro DUMP_GPR_RAW register, register_index
    # 准备数据包: [ 8字节前缀 | 8字节索引 | 8字节值 ]
    la   t0, single_reg_dump_buffer
    la   t1, single_reg_dump_prefix_gpr
    ld   t1, 0(t1)
    sd   t1, 0(t0)                  # 存入GPR前缀
    li   t1, \register_index
    sd   t1, 8(t0)                  # 存入寄存器索引
    sd   \register, 16(t0)          # 存入寄存器值
    # 通过HTIF发送数据包
    HTIF_PRINT_RAW single_reg_dump_buffer, 24
.endm

#if __riscv_flen > 0
.macro DUMP_FPR_RAW register, register_index
    # 准备数据包: [ 8字节前缀 | 8字节索引 | 8字节值 ]
    la   t0, single_reg_dump_buffer
    la   t1, single_reg_dump_prefix_fpr
    ld   t1, 0(t1)
    sd   t1, 0(t0)                  # 存入FPR前缀
    li   t1, \register_index
    sd   t1, 8(t0)                  # 存入寄存器索引
    fsd  \register, 16(t0)          # 存入寄存器值 (使用fsd)
    # 通过HTIF发送数据包
    HTIF_PRINT_RAW single_reg_dump_buffer, 24
.endm
#endif

# ----------------------------------------------------------------------------
# .macro DUMP_GPR / DUMP_FPR
# ----------------------------------------------------------------------------
# 解释: 这些是用户友好宏，封装了暂存寄存器的保存和恢复。
# 参数:
#   - temp_save_area: 用于保存/恢复 t0-t6 寄存器的临时内存区标签
#   - register:       要转储的寄存器名称 (例如 x1, ra, f0, ft0)
#   - register_index: 寄存器的数字索引 (例如 x1 -> 1, f1 -> 1)
# 注意: 由于汇编宏的限制，您需要手动提供寄存器的数字索引。

.macro DUMP_GPR temp_save_area, register, register_index
    SAVE_T_REGS \temp_save_area
    DUMP_GPR_RAW \register, \register_index
    RESTORE_T_REGS \temp_save_area
.endm

#if __riscv_flen > 0
.macro DUMP_FPR temp_save_area, register, register_index
    SAVE_T_REGS \temp_save_area
    # 临时打开浮点单元以访问FPR
    csrr t2, mstatus
    li   t3, (1 << 13) # MSTATUS_FS_INITIAL
    or   t3, t2, t3
    csrw mstatus, t3
    
    DUMP_FPR_RAW \register, \register_index
    
    # 恢复 mstatus
    csrw mstatus, t2
    RESTORE_T_REGS \temp_save_area
.endm
#endif

# ----------------------------------------------------------------------------
# .macro DUMP_EXCEPTION_CSRS_RAW
# ----------------------------------------------------------------------------
.macro DUMP_EXCEPTION_CSRS_RAW
    la   t0, exception_csr_dump_buffer
    csrr t1, mstatus; sd t1,   0(t0); csrr t1, mcause;  sd t1,   8(t0)
    csrr t1, mepc;    sd t1,  16(t0); csrr t1, mtval;   sd t1,  24(t0)
    csrr t1, mie;     sd t1,  32(t0); csrr t1, mip;     sd t1,  40(t0)
    csrr t1, mtvec;   sd t1,  48(t0); csrr t1, mscratch;sd t1,  56(t0)
    csrr t1, mhartid; sd t1,  64(t0)
    HTIF_PRINT_RAW exc_csr_dump_prefix, 8
    HTIF_PRINT_RAW exception_csr_dump_buffer, 72
.endm

# ----------------------------------------------------------------------------
# .macro DUMP_EXCEPTION_CSRS_RAW_ON_MEPC_MATCH
# ----------------------------------------------------------------------------
# 解释: 此宏仅在 mepc 寄存器的值与指定的 target_mepc 相等时，
#       才执行寄存器状态转储。
# 参数:
#   - target_mepc: 您希望触发转储的、确切的指令地址（一个立即数）

.macro DUMP_EXCEPTION_CSRS_RAW_ON_MEPC_MATCH target_mepc
    # 使用 t4, t5 作为临时寄存器 (在 SAVE_T_REGS 的保护范围内)
    csrr t4, mepc
    li   t5, \target_mepc

    # 如果 mepc 和目标地址不匹配，则直接跳过转储逻辑
    bne  t4, t5, .L_skip_dump_\@

    # mepc 匹配，执行完整的状态转储
    DUMP_EXCEPTION_CSRS_RAW

.L_skip_dump_\@:
.endm

# ----------------------------------------------------------------------------
# .macro EXIT_SIM
# ----------------------------------------------------------------------------
.macro EXIT_SIM
    li   t0, 1; la   t1, tohost; sd   t0, 0(t1)
infinite_exit_loop_\@: j infinite_exit_loop_\@
.endm

# ----------------------------------------------------------------------------
# .macro RESET_MACHINE_STATE
# ----------------------------------------------------------------------------
.macro RESET_MACHINE_STATE
    # 临时寄存器
    li t0, 0
    li t1, 0

    # 步骤 1: M-Mode CSRs
    # ---------------------------------
    # 陷阱处理
    csrwi mstatus, 0
    csrwi mie, 0
    csrwi mip, 0
    csrwi mepc, 0
    csrwi mcause, 0
    csrwi mtval, 0
    csrwi mscratch, 0
    # csrwi mtvec, 0
    # 委托
    csrwi medeleg, 0
    csrwi mideleg, 0

    # 物理内存保护 (PMP) 相关 (禁用所有 PMP 区域以提供完全访问权限)
    # 清除前16个 PMP 地址寄存器 (这部分是安全的，因为未实现的寄存器读取为0，写入被忽略)
    csrw pmpaddr0, x0; csrw pmpaddr1, x0; csrw pmpaddr2, x0; csrw pmpaddr3, x0
    csrw pmpaddr4, x0; csrw pmpaddr5, x0; csrw pmpaddr6, x0; csrw pmpaddr7, x0
    csrw pmpaddr8, x0; csrw pmpaddr9, x0; csrw pmpaddr10, x0; csrw pmpaddr11, x0
    csrw pmpaddr12, x0; csrw pmpaddr13, x0; csrw pmpaddr14, x0; csrw pmpaddr15, x0
    # 根据架构宽度清除 PMP 配置寄存器
    # RV64中pmpcfg寄存器为偶数编号，RV32中为连续编号
    #if __riscv_xlen == 64
    csrw pmpcfg0, x0 # 覆盖 pmp0-7 的配置
    csrw pmpcfg2, x0 # 覆盖 pmp8-15 的配置
    #else # 默认为 __riscv_xlen == 32
    csrw pmpcfg0, x0 # 覆盖 pmp0-3 的配置
    csrw pmpcfg1, x0 # 覆盖 pmp4-7 的配置
    csrw pmpcfg2, x0 # 覆盖 pmp8-11 的配置
    csrw pmpcfg3, x0 # 覆盖 pmp12-15 的配置
    #endif 

    # 性能计数器 (HPM)
    csrwi mcounteren, 0
    csrwi scounteren, 0
    csrwi mcountinhibit, 0
    csrw mcycle, t0; csrw minstret, t0
    csrw mhpmcounter3, t0; csrw mhpmevent3, t0
    csrw mhpmcounter4, t0; csrw mhpmevent4, t0
    csrw mhpmcounter5, t0; csrw mhpmevent5, t0
    csrw mhpmcounter6, t0; csrw mhpmevent6, t0
    csrw mhpmcounter7, t0; csrw mhpmevent7, t0
    csrw mhpmcounter8, t0; csrw mhpmevent8, t0
    csrw mhpmcounter9, t0; csrw mhpmevent9, t0
    csrw mhpmcounter10, t0; csrw mhpmevent10, t0
    csrw mhpmcounter11, t0; csrw mhpmevent11, t0
    csrw mhpmcounter12, t0; csrw mhpmevent12, t0
    csrw mhpmcounter13, t0; csrw mhpmevent13, t0
    csrw mhpmcounter14, t0; csrw mhpmevent14, t0
    csrw mhpmcounter15, t0; csrw mhpmevent15, t0
    csrw mhpmcounter16, t0; csrw mhpmevent16, t0
    csrw mhpmcounter17, t0; csrw mhpmevent17, t0
    csrw mhpmcounter18, t0; csrw mhpmevent18, t0
    csrw mhpmcounter19, t0; csrw mhpmevent19, t0
    csrw mhpmcounter20, t0; csrw mhpmevent20, t0
    csrw mhpmcounter21, t0; csrw mhpmevent21, t0
    csrw mhpmcounter22, t0; csrw mhpmevent22, t0
    csrw mhpmcounter23, t0; csrw mhpmevent23, t0
    csrw mhpmcounter24, t0; csrw mhpmevent24, t0
    csrw mhpmcounter25, t0; csrw mhpmevent25, t0
    csrw mhpmcounter26, t0; csrw mhpmevent26, t0
    csrw mhpmcounter27, t0; csrw mhpmevent27, t0
    csrw mhpmcounter28, t0; csrw mhpmevent28, t0
    csrw mhpmcounter29, t0; csrw mhpmevent29, t0
    csrw mhpmcounter30, t0; csrw mhpmevent30, t0
    csrw mhpmcounter31, t0; csrw mhpmevent31, t0
    # 触发器
    csrwi tselect, 0; csrwi tdata1, 0; csrwi tdata2, 0
    csrwi tselect, 1; csrwi tdata1, 0; csrwi tdata2, 0
    csrwi tselect, 0
    
    # 步骤 2: S-Mode & U-Mode CSRs
    # ---------------------------------
    csrwi sstatus, 0; csrwi sie, 0; csrwi sip, 0
    csrwi sepc, 0; csrwi scause, 0; csrwi stval, 0
    csrwi sscratch, 0; csrwi stvec, 0; csrwi satp, 0

    # 步骤 3: 浮点扩展 (F/D)
    # ---------------------------------
#if __riscv_flen > 0
    csrr t0, mstatus
    li   t1, (1 << 13)
    or   t0, t0, t1
    csrw mstatus, t0
    csrwi fcsr, 0
    fmv.d.x f0, x0; fmv.d.x f1, x0; fmv.d.x f2, x0; fmv.d.x f3, x0
    fmv.d.x f4, x0; fmv.d.x f5, x0; fmv.d.x f6, x0; fmv.d.x f7, x0
    fmv.d.x f8, x0; fmv.d.x f9, x0; fmv.d.x f10, x0; fmv.d.x f11, x0
    fmv.d.x f12, x0; fmv.d.x f13, x0; fmv.d.x f14, x0; fmv.d.x f15, x0
    fmv.d.x f16, x0; fmv.d.x f17, x0; fmv.d.x f18, x0; fmv.d.x f19, x0
    fmv.d.x f20, x0; fmv.d.x f21, x0; fmv.d.x f22, x0; fmv.d.x f23, x0
    fmv.d.x f24, x0; fmv.d.x f25, x0; fmv.d.x f26, x0; fmv.d.x f27, x0
    fmv.d.x f28, x0; fmv.d.x f29, x0; fmv.d.x f30, x0; fmv.d.x f31, x0
#endif

    # 步骤 4: 向量扩展 (V)
    # ---------------------------------
#if defined(__riscv_v_intrinsic)
    csrr t0, mstatus
    li   t1, (1 << 9)
    or   t0, t0, t1
    csrw mstatus, t0
    csrwi vcsr, 0
    csrwi vxrm, 0
    csrwi vxsat, 0
    li t0, 0
    csrw vstart, t0
    csrw vl, t0
    csrw vtype, t0
    li t0, 8
    csrw vtype, t0
    csrr t1, vlenb
    slli t1, t1, 3 
    csrw vl, t1
    vmv.v.x v0, x0; vmv.v.x v1, x0; vmv.v.x v2, x0; vmv.v.x v3, x0
    vmv.v.x v4, x0; vmv.v.x v5, x0; vmv.v.x v6, x0; vmv.v.x v7, x0
    vmv.v.x v8, x0; vmv.v.x v9, x0; vmv.v.x v10, x0; vmv.v.x v11, x0
    vmv.v.x v12, x0; vmv.v.x v13, x0; vmv.v.x v14, x0; vmv.v.x v15, x0
    vmv.v.x v16, x0; vmv.v.x v17, x0; vmv.v.x v18, x0; vmv.v.x v19, x0
    vmv.v.x v20, x0; vmv.v.x v21, x0; vmv.v.x v22, x0; vmv.v.x v23, x0
    vmv.v.x v24, x0; vmv.v.x v25, x0; vmv.v.x v26, x0; vmv.v.x v27, x0
    vmv.v.x v28, x0; vmv.v.x v29, x0; vmv.v.x v30, x0; vmv.v.x v31, x0
    li t0, 0
    csrw vl, t0
    csrw vtype, t0
#endif

    # 步骤 5: 虚拟机扩展 (H)
    # ---------------------------------
    csrwi hstatus, 0; csrwi hedeleg, 0; csrwi hideleg, 0
    csrwi hie, 0; csrwi hip, 0
    csrwi hgeie, 0; 
    csrwi htval, 0; csrwi htinst, 0
    csrwi hgatp, 0; csrwi hcounteren, 0
    li t0, 0
  #if __riscv_xlen == 32
    csrw htimedeltah, t0
  #endif
    csrwi vsstatus, 0; csrwi vsie, 0; csrwi vsip, 0
    csrwi vstvec, 0; csrwi vsscratch, 0; csrwi vsepc, 0
    csrwi vscause, 0; csrwi vstval, 0; csrwi vsatp, 0

    # 步骤 6: 通用寄存器 (GPRs)
    # ---------------------------------
    mv x1,  zero; mv x2,  zero; mv x3,  zero; mv x4,  zero
    mv x5,  zero; mv x6,  zero; mv x7,  zero; mv x8,  zero
    mv x9,  zero; mv x10, zero; mv x11, zero; mv x12, zero
    mv x13, zero; mv x14, zero; mv x15, zero; mv x16, zero
    mv x17, zero; mv x18, zero; mv x19, zero; mv x20, zero
    mv x21, zero; mv x22, zero; mv x23, zero; mv x24, zero
    mv x25, zero; mv x26, zero; mv x27, zero; mv x28, zero
    mv x29, zero; mv x30, zero; mv x31, zero

.endm

"#
}

fn get_data_sections() -> String {
    String::from(
        r#"# ============================================================================
# 内存与数据区定义
# ============================================================================
.section .bss
.align 4
register_dump_buffer:       .zero 1024
exception_csr_dump_buffer: .zero 72
framework_temp_save_area:   .zero 64
single_reg_dump_buffer:     .zero 24

.section .data
.align 6
htif_communication_buffer: .zero 64
# 寄存器转储前缀标识符
# 魔数编码: 0xFEEDC0DE + 类型标识
# 类型标识: 0x1000 = 整数寄存器 + 浮点寄存器, 0x2000 = 仅整数寄存器
#if __riscv_flen > 0
full_reg_dump_prefix_with_fp:
    .dword 0xFEEDC0DE1000
#endif

#if __riscv_flen == 0
full_reg_dump_prefix_no_fp:
    .dword 0xFEEDC0DE2000
#endif

# 单个寄存器转储前缀标识符
single_reg_dump_prefix_gpr:
    .dword 0xFEEDC0DE1001

#if __riscv_flen > 0
single_reg_dump_prefix_fpr:
    .dword 0xFEEDC0DE1002
#endif

# 异常CSR转储前缀标识符
# 魔数编码: 0xBADC0DE + 类型标识
# 类型标识: 0x1000 = 异常转储
exc_csr_dump_prefix:
    .dword 0xBADC0DE1000

.section .tohost, "aw", @progbits
.align 6
.globl tohost
tohost:   .dword 0
.globl fromhost
fromhost: .dword 0

.section .text
.globl _start

"#,
    )
}

fn get_exception_handler(dump_config: &Option<DumpException>) -> String {
    let mut handler = String::from(
        r#"# ============================================================================
# 异常处理程序
# ============================================================================
exception_handler:
    # 一次性保存寄存器，避免嵌套
    SAVE_T_REGS framework_temp_save_area
    
"#,
    );

    match dump_config {
        Some(DumpException::All) => {
            handler.push_str(
                r#"    # 转储异常CSR信息 - 使用RAW版本避免额外的保存/恢复
    DUMP_EXCEPTION_CSRS_RAW
"#,
            );
        }
        Some(DumpException::OnMepcMatch(mepc_list)) => {
            for &mepc in mepc_list {
                handler.push_str(&format!(
                    r#"    # 转储指定MEPC地址(0x{:x})的异常信息
    DUMP_EXCEPTION_CSRS_RAW_ON_MEPC_MATCH 0x{:x}
"#,
                    mepc, mepc
                ));
            }
        }
        None => {
            // 不转储异常信息
        }
    }

    handler.push_str(
        r#"    # 获取异常指令地址
    csrr t0, mepc
    
    # 读取异常指令的内容来判断长度
    lhu t1, 0(t0)
    andi t2, t1, 0x3
    li t3, 0x3
    bne t2, t3, compressed_inst
    
    # 标准指令(4字节)
    addi t0, t0, 4
    j update_mepc
    
compressed_inst:
    # 压缩指令(2字节)
    addi t0, t0, 2
    
update_mepc:
    csrw mepc, t0
    csrwi mcause, 0
    csrwi mtval, 0
    csrwi mip, 0

    # 一次性恢复寄存器
    RESTORE_T_REGS framework_temp_save_area
    mret

"#,
    );

    handler
}

fn get_dump_registers_code(dump_config: &DumpRegister) -> String {
    match dump_config {
        DumpRegister::All => {
            r#"
    DUMP_ALL_REGS framework_temp_save_area
"#
            .to_string()
        }
        DumpRegister::Gpr(gpr_list) => {
            let mut code = String::new();
            for &reg_idx in gpr_list {
                code.push_str(&format!(
                    r#"
    DUMP_GPR framework_temp_save_area, x{}, {}
"#,
                    reg_idx, reg_idx
                ));
            }
            code
        }
        DumpRegister::Fpr(fpr_list) => {
            let mut code = String::from(
                r#"
#if __riscv_flen > 0
"#,
            );
            for &reg_idx in fpr_list {
                code.push_str(&format!(
                    r#"    DUMP_FPR framework_temp_save_area, f{}, {}
"#,
                    reg_idx, reg_idx
                ));
            }
            code.push_str(
                r#"#endif
"#,
            );
            code
        }
        DumpRegister::GprAndFpr { gpr, fpr } => {
            let mut code = String::new();
            
            // 转储GPR寄存器
            for &reg_idx in gpr {
                code.push_str(&format!(
                    r#"
    DUMP_GPR framework_temp_save_area, x{}, {}
"#,
                    reg_idx, reg_idx
                ));
            }
            
            // 转储FPR寄存器
            if !fpr.is_empty() {
                code.push_str(
                    r#"
#if __riscv_flen > 0
"#,
                );
                for &reg_idx in fpr {
                    code.push_str(&format!(
                        r#"    DUMP_FPR framework_temp_save_area, f{}, {}
"#,
                        reg_idx, reg_idx
                    ));
                }
                code.push_str(
                    r#"#endif
"#,
                );
            }
            
            code
        }
    }
}

fn get_main_program(user_code: &str, dump_config: &Option<DumpRegister>) -> String {
    let mut program = format!(
        r#"# ============================================================================
# 程序入口与执行
# ============================================================================
_start:

_init:
    la t0, exception_handler
    csrw mtvec, t0

    RESET_MACHINE_STATE

_user_code:
{}

"#,
        user_code
    );

    if let Some(dump_config) = dump_config {
        program.push_str(
            r#"
_dump_regs:
"#,
        );
        program.push_str(&get_dump_registers_code(dump_config));
    }

    program.push_str(
        r#"
_exit:
    EXIT_SIM
"#,
    );

    program
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_generation() {
        let user_code = "    addi t0, t0, 1";

        let full_template = generate_standard_asm(user_code);
        std::fs::write("full_template.S", &full_template).unwrap();

        let minimal_template = generate_minimal_asm(user_code);
        std::fs::write("minimal_template.S", &minimal_template).unwrap();
        
        // 测试自定义配置
        let custom_template = generate_asm(
            user_code,
            Some(DumpException::OnMepcMatch(vec![0x1000, 0x2000])),
            Some(DumpRegister::GprAndFpr { 
                gpr: vec![1, 2, 3], 
                fpr: vec![0, 1] 
            })
        );
        std::fs::write("custom_template.S", &custom_template).unwrap();
    }
}
