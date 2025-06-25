/// Exception dump configuration
#[derive(Debug, Clone)]
pub enum DumpException {
    /// Dump all exceptions
    All,
    /// Dump exceptions at specified MEPC addresses
    OnMepcMatch(Vec<u64>),
}

/// Register dump configuration
#[derive(Debug, Clone)]
pub enum DumpRegister {
    /// Dump all registers
    All,
    /// Dump specified GPR register list (register numbers)
    Gpr(Vec<u32>),
    /// Dump specified FPR register list (register numbers)
    Fpr(Vec<u32>),
    /// Dump specified GPR and FPR registers
    GprAndFpr { gpr: Vec<u32>, fpr: Vec<u32> },
}

/// Generate complete RISC-V assembly template (including exception dump and register dump)
pub fn generate_standard_asm(user_code: &str) -> String {
    generate_asm(user_code, Some(DumpException::All), Some(DumpRegister::All))
}

pub fn generate_minimal_asm(user_code: &str) -> String {
    generate_asm(user_code, None, None)
}

/// Generate custom RISC-V assembly template
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
# Macro Definitions
# ============================================================================

# ----------------------------------------------------------------------------
# .macro SAVE_T_REGS / RESTORE_T_REGS
# ----------------------------------------------------------------------------
.macro SAVE_T_REGS save_area_label
    csrw mscratch, t6
    la   t6, \save_area_label

    sd   t0,   0(t6)
    sd   t1,   8(t6)
    sd   t2,  16(t6)
    sd   t3,  24(t6)
    sd   t4,  32(t6)
    sd   t5,  40(t6)

    csrr t5, mscratch
    sd   t5,  48(t6)

    ld   t5,  40(t6)
    csrr t6, mscratch
.endm

.macro RESTORE_T_REGS save_area_label
    la   t6, \save_area_label
    ld   t0,   0(t6)
    ld   t1,   8(t6)
    ld   t2,  16(t6)
    ld   t3,  24(t6)
    ld   t4,  32(t6)
    ld   t5,  40(t6)
    ld   t6,  48(t6)
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
# ----------------------------------------------------------------------------
.macro DUMP_ALL_REGS_RAW
    csrw mscratch, t6
    la t6, register_dump_buffer

    # Dump all general-purpose registers (x0-x31)
    sd  x0,    0(t6); sd  x1,    8(t6); sd  x2,   16(t6); sd  x3,   24(t6)
    sd  x4,   32(t6); sd  x5,   40(t6); sd  x6,   48(t6); sd  x7,   56(t6)
    sd  x8,   64(t6); sd  x9,   72(t6); sd x10,   80(t6); sd x11,   88(t6)
    sd x12,   96(t6); sd x13,  104(t6); sd x14,  112(t6); sd x15,  120(t6)
    sd x16,  128(t6); sd x17,  136(t6); sd x18,  144(t6); sd x19,  152(t6)
    sd x20,  160(t6); sd x21,  168(t6); sd x22,  176(t6); sd x23,  184(t6)
    sd x24,  192(t6); sd x25,  200(t6); sd x26,  208(t6); sd x27,  216(t6)
    sd x28,  224(t6); sd x29,  232(t6); sd x30,  240(t6)
    csrr t5, mscratch
    sd t5, 248(t6)  # x31 (t6) original value

    # Dump core machine mode CSR registers
    csrr t0, mstatus;     sd t0, 256(t6) # Machine status
    csrr t0, misa;        sd t0, 264(t6) # ISA and extensions
    csrr t0, medeleg;     sd t0, 272(t6) # Machine exception delegation
    csrr t0, mideleg;     sd t0, 280(t6) # Machine interrupt delegation
    csrr t0, mie;         sd t0, 288(t6) # Machine interrupt enable
    csrr t0, mtvec;       sd t0, 296(t6) # Machine trap vector base address
    csrr t0, mcounteren;  sd t0, 304(t6) # Machine counter enable
    
    # Machine trap handling CSRs
    csrr t0, mscratch;    sd t0, 312(t6) # Machine scratch register
    csrr t0, mepc;        sd t0, 320(t6) # Machine exception program counter
    csrr t0, mcause;      sd t0, 328(t6) # Machine trap cause
    csrr t0, mtval;       sd t0, 336(t6) # Machine bad address or instruction
    csrr t0, mip;         sd t0, 344(t6) # Machine interrupt pending

    # Machine counter/timer CSRs
    csrr t0, mcycle;      sd t0, 352(t6) # Machine cycle counter
    csrr t0, minstret;    sd t0, 360(t6) # Machine instructions retired counter
    
    # Machine information CSRs
    csrr t0, mvendorid;   sd t0, 368(t6) # Vendor ID
    csrr t0, marchid;     sd t0, 376(t6) # Architecture ID
    csrr t0, mimpid;      sd t0, 384(t6) # Implementation ID
    csrr t0, mhartid;     sd t0, 392(t6) # Hardware thread ID

    .set DUMP_SIZE_NO_FP, 400

#if __riscv_flen > 0
    # F/D extension present: dump floating-point registers and status
    csrr t0, mstatus
    li   t1, (1 << 13) # MSTATUS_FS_INITIAL
    or   t1, t0, t1
    csrw mstatus, t1

    # Floating-point control and status register
    csrr t1, fcsr;      sd t1, 400(t6)
    
    # Floating-point registers (f0-f31)
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
    # Prepare data packet: [ 8-byte prefix | 8-byte index | 8-byte value ]
    la   t0, single_reg_dump_buffer
    la   t1, single_reg_dump_prefix_gpr
    ld   t1, 0(t1)
    sd   t1, 0(t0)                  # Store GPR prefix
    li   t1, \register_index
    sd   t1, 8(t0)                  # Store register index
    sd   \register, 16(t0)          # Store register value
    # Send data packet via HTIF
    HTIF_PRINT_RAW single_reg_dump_buffer, 24
.endm

#if __riscv_flen > 0
.macro DUMP_FPR_RAW register, register_index
    # Prepare data packet: [ 8-byte prefix | 8-byte index | 8-byte value ]
    la   t0, single_reg_dump_buffer
    la   t1, single_reg_dump_prefix_fpr
    ld   t1, 0(t1)
    sd   t1, 0(t0)                  # Store FPR prefix
    li   t1, \register_index
    sd   t1, 8(t0)                  # Store register index
    fsd  \register, 16(t0)          # Store register value (using fsd)
    # Send data packet via HTIF
    HTIF_PRINT_RAW single_reg_dump_buffer, 24
.endm
#endif

# ----------------------------------------------------------------------------
# .macro DUMP_GPR / DUMP_FPR
# ----------------------------------------------------------------------------
# Description: These are user-friendly macros that encapsulate saving and restoring temporary registers.
# Parameters:
#   - temp_save_area: label for temporary memory area to save/restore t0-t6 registers
#   - register:       register name to dump (e.g. x1, ra, f0, ft0)
#   - register_index: numeric index of the register (e.g. x1 -> 1, f1 -> 1)
# Note: Due to assembly macro limitations, you need to manually provide the numeric index of the register.

.macro DUMP_GPR temp_save_area, register, register_index
    SAVE_T_REGS \temp_save_area
    DUMP_GPR_RAW \register, \register_index
    RESTORE_T_REGS \temp_save_area
.endm

#if __riscv_flen > 0
.macro DUMP_FPR temp_save_area, register, register_index
    SAVE_T_REGS \temp_save_area
    # Temporarily enable floating-point unit to access FPR
    csrr t2, mstatus
    li   t3, (1 << 13) # MSTATUS_FS_INITIAL
    or   t3, t2, t3
    csrw mstatus, t3
    
    DUMP_FPR_RAW \register, \register_index
    
    # Restore mstatus
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
# Description: This macro only executes register state dump when the value of the mepc register
#              equals the specified target_mepc.
# Parameters:
#   - target_mepc: The exact instruction address where you want to trigger the dump (an immediate value)

.macro DUMP_EXCEPTION_CSRS_RAW_ON_MEPC_MATCH target_mepc
    # Use t4, t5 as temporary registers (within SAVE_T_REGS protection scope)
    csrr t4, mepc
    li   t5, \target_mepc

    # If mepc and target address don't match, skip dump logic directly
    bne  t4, t5, .L_skip_dump_\@

    # mepc matches, execute complete state dump
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
    # Temporary registers
    li t0, 0
    li t1, 0

    # Step 1: M-Mode CSRs
    # ---------------------------------
    # Trap handling
    csrwi mstatus, 0
    csrwi mie, 0
    csrwi mip, 0
    csrwi mepc, 0
    csrwi mcause, 0
    csrwi mtval, 0
    csrwi mscratch, 0
    # csrwi mtvec, 0
    # Delegation
    csrwi medeleg, 0
    csrwi mideleg, 0

    # Comment and uncomment has the same results
    # # Physical Memory Protection (PMP) related (disable all PMP regions to provide full access)
    # # Clear first 16 PMP address registers (this is safe as unimplemented registers read as 0, writes are ignored)
    # csrw pmpaddr0, x0; csrw pmpaddr1, x0; csrw pmpaddr2, x0; csrw pmpaddr3, x0
    # csrw pmpaddr4, x0; csrw pmpaddr5, x0; csrw pmpaddr6, x0; csrw pmpaddr7, x0
    # csrw pmpaddr8, x0; csrw pmpaddr9, x0; csrw pmpaddr10, x0; csrw pmpaddr11, x0
    # csrw pmpaddr12, x0; csrw pmpaddr13, x0; csrw pmpaddr14, x0; csrw pmpaddr15, x0
    # # Clear PMP configuration registers according to architecture width
    # # In RV64 pmpcfg registers are even-numbered, in RV32 they are consecutive
    # #if __riscv_xlen == 64
    # csrw pmpcfg0, x0 # Covers pmp0-7 configuration
    # csrw pmpcfg2, x0 # Covers pmp8-15 configuration
    # #else # Default to __riscv_xlen == 32
    # csrw pmpcfg0, x0 # Covers pmp0-3 configuration
    # csrw pmpcfg1, x0 # Covers pmp4-7 configuration
    # csrw pmpcfg2, x0 # Covers pmp8-11 configuration
    # csrw pmpcfg3, x0 # Covers pmp12-15 configuration
    # #endif 

    # Performance counters (HPM)
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
    # Triggers
    csrwi tselect, 0; csrwi tdata1, 0; csrwi tdata2, 0
    csrwi tselect, 1; csrwi tdata1, 0; csrwi tdata2, 0
    csrwi tselect, 0
    
    # Step 2: S-Mode & U-Mode CSRs
    # ---------------------------------
    csrwi sstatus, 0; csrwi sie, 0; csrwi sip, 0
    csrwi sepc, 0; csrwi scause, 0; csrwi stval, 0
    csrwi sscratch, 0; csrwi stvec, 0; csrwi satp, 0

    # Step 3: Floating-point extension (F/D)
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

    # Step 4: Vector extension (V)
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

    # Step 5: Hypervisor extension (H)
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

    # Step 6: General-purpose registers (GPRs)
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
# Memory and Data Section Definitions
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
# Register dump prefix identifiers
# Magic number encoding: 0xFEEDC0DE + type identifier
# Type identifier: 0x1000 = integer registers + floating-point registers, 0x2000 = integer registers only
#if __riscv_flen > 0
full_reg_dump_prefix_with_fp:
    .dword 0xFEEDC0DE1000
#endif

#if __riscv_flen == 0
full_reg_dump_prefix_no_fp:
    .dword 0xFEEDC0DE2000
#endif

# Single register dump prefix identifiers
single_reg_dump_prefix_gpr:
    .dword 0xFEEDC0DE1001

#if __riscv_flen > 0
single_reg_dump_prefix_fpr:
    .dword 0xFEEDC0DE1002
#endif

# Exception CSR dump prefix identifier
# Magic number encoding: 0xBADC0DE + type identifier
# Type identifier: 0x1000 = exception dump
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
# Exception Handler
# ============================================================================
exception_handler:
    # Save registers once to avoid nesting
    SAVE_T_REGS framework_temp_save_area
    
"#,
    );

    match dump_config {
        Some(DumpException::All) => {
            handler.push_str(
                r#"    # Dump exception CSR information - use RAW version to avoid additional save/restore
    DUMP_EXCEPTION_CSRS_RAW
"#,
            );
        }
        Some(DumpException::OnMepcMatch(mepc_list)) => {
            for &mepc in mepc_list {
                handler.push_str(&format!(
                    r#"    # Dump exception information for specified MEPC address (0x{:x})
    DUMP_EXCEPTION_CSRS_RAW_ON_MEPC_MATCH 0x{:x}
"#,
                    mepc, mepc
                ));
            }
        }
        None => {
            // Don't dump exception information
        }
    }

    handler.push_str(
        r#"    # Get exception instruction address
    csrr t0, mepc
    
    # Read exception instruction content to determine length
    lhu t1, 0(t0)
    andi t2, t1, 0x3
    li t3, 0x3
    bne t2, t3, compressed_inst
    
    # Standard instruction (4 bytes)
    addi t0, t0, 4
    j update_mepc
    
compressed_inst:
    # Compressed instruction (2 bytes)
    addi t0, t0, 2
    
update_mepc:
    csrw mepc, t0
    csrwi mcause, 0
    csrwi mtval, 0
    csrwi mip, 0

    # Restore registers once
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
            
            // Dump GPR registers
            for &reg_idx in gpr {
                code.push_str(&format!(
                    r#"
    DUMP_GPR framework_temp_save_area, x{}, {}
"#,
                    reg_idx, reg_idx
                ));
            }
            
            // Dump FPR registers
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
# Program Entry and Execution
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
        
        // Test custom configuration
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
