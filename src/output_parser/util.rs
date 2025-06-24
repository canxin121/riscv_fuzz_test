pub fn get_register_name(reg_num: usize) -> &'static str {
    match reg_num {
        0 => "zero",
        1 => "ra",
        2 => "sp",
        3 => "gp",
        4 => "tp",
        5 => "t0",
        6 => "t1",
        7 => "t2",
        8 => "s0",
        9 => "s1",
        10 => "a0",
        11 => "a1",
        12 => "a2",
        13 => "a3",
        14 => "a4",
        15 => "a5",
        16 => "a6",
        17 => "a7",
        18 => "s2",
        19 => "s3",
        20 => "s4",
        21 => "s5",
        22 => "s6",
        23 => "s7",
        24 => "s8",
        25 => "s9",
        26 => "s10",
        27 => "s11",
        28 => "t3",
        29 => "t4",
        30 => "t5",
        31 => "t6",
        _ => "invalid",
    }
}

/// Get exception description
pub fn get_exception_description(mcause: u64) -> String {
    let interrupt = (mcause >> 63) & 1 == 1;
    let exception_code = mcause & 0x7FFFFFFFFFFFFFFF;

    if interrupt {
        match exception_code {
            0 => "User software interrupt".to_string(),
            1 => "Supervisor software interrupt".to_string(),
            3 => "Machine software interrupt".to_string(),
            4 => "User timer interrupt".to_string(),
            5 => "Supervisor timer interrupt".to_string(),
            7 => "Machine timer interrupt".to_string(),
            8 => "User external interrupt".to_string(),
            9 => "Supervisor external interrupt".to_string(),
            11 => "Machine external interrupt".to_string(),
            _ => format!("Unknown interrupt (code={})", exception_code),
        }
    } else {
        match exception_code {
            0 => "Instruction address misaligned".to_string(),
            1 => "Instruction access fault".to_string(),
            2 => "Illegal instruction".to_string(),
            3 => "Breakpoint".to_string(),
            4 => "Load address misaligned".to_string(),
            5 => "Load access fault".to_string(),
            6 => "Store/AMO address misaligned".to_string(),
            7 => "Store/AMO access fault".to_string(),
            8 => "Environment call from U-mode".to_string(),
            9 => "Environment call from S-mode".to_string(),
            11 => "Environment call from M-mode".to_string(),
            12 => "Instruction page fault".to_string(),
            13 => "Load page fault".to_string(),
            15 => "Store/AMO page fault".to_string(),
            _ => format!("Unknown exception (code={})", exception_code),
        }
    }
}

/// Get detailed register description
pub fn get_register_description(reg_num: usize) -> &'static str {
    match reg_num {
        0 => "Zero register (always 0)",
        1 => "Return address register",
        2 => "Stack pointer register",
        3 => "Global pointer register",
        4 => "Thread pointer register",
        5..=7 => "Temporary register",
        8 => "Frame pointer/Saved register",
        9 => "Saved register",
        10..=11 => "Function argument/return value register",
        12..=17 => "Function argument register",
        18..=27 => "Saved register",
        28..=31 => "Temporary register",
        _ => "Invalid register",
    }
}

/// Get floating-point register ABI name
pub fn get_float_register_name(reg_num: usize) -> String {
    match reg_num {
        0..=7 => format!("ft{}", reg_num),
        8..=9 => format!("fs{}", reg_num - 8),
        10..=17 => format!("fa{}", reg_num - 10),
        18..=27 => format!("fs{}", reg_num - 18 + 2),
        28..=31 => format!("ft{}", reg_num - 28 + 8),
        _ => "invalid".to_string(),
    }
}

/// Get detailed floating-point register description
pub fn get_float_register_description(reg_num: usize) -> &'static str {
    match reg_num {
        0..=7 => "Temporary floating-point register",
        8..=9 => "Saved floating-point register",
        10..=17 => "Floating-point argument/return value register",
        18..=27 => "Saved floating-point register",
        28..=31 => "Temporary floating-point register",
        _ => "Invalid floating-point register",
    }
}

/// Format hexadecimal value to human-readable format
pub fn format_hex_value(value: u64) -> String {
    format!("0x{:016X}", value)
}

/// Get detailed CSR register description
pub fn get_csr_description(csr_name: &str) -> &'static str {
    match csr_name {
        "mstatus" => "Machine status register - controls global interrupt enable, privilege level, etc.",
        "misa" => "Machine ISA register - indicates supported instruction set extensions",
        "medeleg" => "Machine exception delegation register - controls exception delegation",
        "mideleg" => "Machine interrupt delegation register - controls interrupt delegation",
        "mie" => "Machine interrupt enable register - controls various interrupt enables",
        "mtvec" => "Machine trap vector base address register - exception and interrupt handler address",
        "mcounteren" => "Machine counter enable register - controls performance counter access",
        "mscratch" => "Machine scratch register - used to save temporary data",
        "mepc" => "Machine exception program counter - saves PC when exception occurs",
        "mcause" => "Machine trap cause register - indicates exception or interrupt cause",
        "mtval" => "Machine trap value register - saves exception-related address or instruction",
        "mip" => "Machine interrupt pending register - shows currently pending interrupts",
        "mcycle" => "Machine cycle counter - records CPU cycle count",
        "minstret" => "Machine instructions retired counter - records executed instruction count",
        "mvendorid" => "Machine vendor ID register - identifies hardware vendor",
        "marchid" => "Machine architecture ID register - identifies microarchitecture",
        "mimpid" => "Machine implementation ID register - identifies specific implementation version",
        "mhartid" => "Machine hardware thread ID register - identifies current hart",
        "fcsr" => "Floating-point control and status register - controls floating-point operation mode and exceptions",
        _ => "Unknown CSR register",
    }
}

/// Generate current timestamp
pub fn get_current_timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
