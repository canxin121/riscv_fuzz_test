use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::{
    emulators::EmulatorType,
    error::Result,
    output_parser::{
        ExceptionDump, OutputParser, RegistersDump,
        common::{self, OutputItem},
        util::{get_exception_description, get_register_name},
    },
};

/// Conversion statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionStats {
    /// Original exception dump count
    pub original_exception_count: usize,
    /// Original register dump count
    pub original_register_count: usize,
    /// Conversion success status
    pub conversion_successful: bool,
    /// Conversion warning messages
    pub warnings: Vec<String>,
}

/// Standardized execution output structure
/// Includes exception dumps and a single register dump
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardExecutionOutput {
    /// Emulator type
    pub emulator_type: EmulatorType,
    /// Exception dump list
    pub exceptions: Vec<ExceptionDump>,
    /// Register dump (usually only one)
    pub register_dump: Option<RegistersDump>,
    /// Statistics information during conversion
    pub conversion_stats: ConversionStats,
}

impl Default for StandardExecutionOutput {
    fn default() -> Self {
        Self {
            emulator_type: EmulatorType::Spike,
            exceptions: Vec::new(),
            register_dump: None,
            conversion_stats: ConversionStats {
                original_exception_count: 0,
                original_register_count: 0,
                conversion_successful: true,
                warnings: Vec::new(),
            },
        }
    }
}

// Implement fmt::Display for StandardExecutionOutput
impl std::fmt::Display for StandardExecutionOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "# 🎯 RISC-V Standard Execution Output")?;
        writeln!(f)?;
        writeln!(f, "**Emulator Type:** `{}`", self.emulator_type)?;
        writeln!(f)?;

        // Basic information table
        writeln!(f, "## 📊 Basic Information")?;
        writeln!(f)?;
        writeln!(f, "| Item | Value |")?;
        writeln!(f, "|------|-------|")?;
        writeln!(f, "| Exception Count | `{}` |", self.exceptions.len())?;
        writeln!(
            f,
            "| Register Dump | `{}` |",
            if self.register_dump.is_some() {
                "Present"
            } else {
                "None"
            }
        )?;
        writeln!(f)?;

        // Conversion statistics
        writeln!(f, "## 🔄 Conversion Statistics")?;
        writeln!(f)?;
        writeln!(f, "| Statistics Item | Value | Status |")?;
        writeln!(f, "|-----------------|-------|--------|")?;
        writeln!(
            f,
            "| Original Exception Count | `{}` | - |",
            self.conversion_stats.original_exception_count
        )?;
        writeln!(
            f,
            "| Original Register Dump Count | `{}` | - |",
            self.conversion_stats.original_register_count
        )?;
        writeln!(
            f,
            "| Conversion Successful | `{}` | {} |",
            self.conversion_stats.conversion_successful,
            if self.conversion_stats.conversion_successful {
                "✅"
            } else {
                "❌"
            }
        )?;
        writeln!(
            f,
            "| Warning Count | `{}` | {} |",
            self.conversion_stats.warnings.len(),
            if self.conversion_stats.warnings.is_empty() {
                "✅"
            } else {
                "⚠️"
            }
        )?;
        writeln!(f)?;

        if !self.conversion_stats.warnings.is_empty() {
            writeln!(f, "### ⚠️ Conversion Warnings (Complete List)")?;
            writeln!(f)?;
            // Show all warnings without truncation
            for (i, warning) in self.conversion_stats.warnings.iter().enumerate() {
                writeln!(f, "{}. `{}`", i + 1, warning)?;
            }
            writeln!(f)?;
        }

        // Exception list
        if !self.exceptions.is_empty() {
            writeln!(f, "## 🚨 `{}` Exception List", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "**Total:** `{} exceptions`", self.exceptions.len())?;
            writeln!(f)?;

            writeln!(f, "| # | MEPC | MCAUSE | Exception Description | MTVAL | Position |")?;
            writeln!(f, "|---|------|--------|----------------------|-------|----------|")?;

            // Show all exceptions without truncation
            for (i, ex) in self.exceptions.iter().enumerate() {
                let desc = get_exception_description(ex.csrs.mcause);
                writeln!(
                    f,
                    "| {} | `0x{:016X}` | `0x{:016X}` | {} | `0x{:016X}` | `{}` |",
                    i + 1,
                    ex.csrs.mepc,
                    ex.csrs.mcause,
                    desc,
                    ex.csrs.mtval,
                    ex.position
                )?;
            }
            writeln!(f)?;
        } else {
            writeln!(f, "## 🚨 `{}` Exception List", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "> ✅ **No exception records**")?;
            writeln!(f)?;
        }

        // Register dump
        if let Some(dump) = &self.register_dump {
            writeln!(f, "## 📝 `{}` Final Register Dump", self.emulator_type)?;
            writeln!(f)?;
            writeln!(
                f,
                "**Dump Type:** `{:?}` | **Position:** `{}`",
                dump.dump_type, dump.position
            )?;
            writeln!(f)?;

            // Complete list of core registers
            writeln!(f, "### 🎯 All Integer Registers")?;
            writeln!(f)?;
            writeln!(f, "| Register | Value | Description |")?;
            writeln!(f, "|----------|-------|-------------|")?;
            for i in 0..32 {
                let reg_name = get_register_name(i);
                let description = match i {
                    0 => "Zero register",
                    1 => "Return address",
                    2 => "Stack pointer",
                    3 => "Global pointer",
                    4 => "Thread pointer",
                    5..=7 => "Temporary register",
                    8 => "Frame pointer/Saved register",
                    9 => "Saved register",
                    10..=11 => "Function argument/return value",
                    12..=17 => "Function argument",
                    18..=27 => "Saved register",
                    28..=31 => "Temporary register",
                    _ => "Unknown register",
                };
                writeln!(
                    f,
                    "| `{}` (x{}) | `0x{:016X}` | {} |",
                    reg_name, i, dump.int_registers[i], description
                )?;
            }
            writeln!(f)?;

            // Complete list of core CSRs
            writeln!(f, "### ⚙️ All Core CSRs")?;
            writeln!(f)?;
            writeln!(f, "| CSR | Value | Description |")?;
            writeln!(f, "|-----|-------|-------------|")?;
            writeln!(
                f,
                "| `mstatus` | `0x{:016X}` | Machine status register |",
                dump.core_csrs.mstatus
            )?;
            writeln!(
                f,
                "| `misa` | `0x{:016X}` | ISA and extensions |",
                dump.core_csrs.misa
            )?;
            writeln!(
                f,
                "| `medeleg` | `0x{:016X}` | Machine exception delegation |",
                dump.core_csrs.medeleg
            )?;
            writeln!(
                f,
                "| `mideleg` | `0x{:016X}` | Machine interrupt delegation |",
                dump.core_csrs.mideleg
            )?;
            writeln!(
                f,
                "| `mie` | `0x{:016X}` | Machine interrupt enable |",
                dump.core_csrs.mie
            )?;
            writeln!(
                f,
                "| `mtvec` | `0x{:016X}` | Machine trap vector base address |",
                dump.core_csrs.mtvec
            )?;
            writeln!(
                f,
                "| `mcounteren` | `0x{:016X}` | Machine counter enable |",
                dump.core_csrs.mcounteren
            )?;
            writeln!(
                f,
                "| `mscratch` | `0x{:016X}` | Machine scratch register |",
                dump.core_csrs.mscratch
            )?;
            writeln!(
                f,
                "| `mepc` | `0x{:016X}` | Machine exception program counter |",
                dump.core_csrs.mepc
            )?;
            writeln!(
                f,
                "| `mcause` | `0x{:016X}` | Machine trap cause |",
                dump.core_csrs.mcause
            )?;
            writeln!(
                f,
                "| `mtval` | `0x{:016X}` | Machine bad address or instruction |",
                dump.core_csrs.mtval
            )?;
            writeln!(
                f,
                "| `mip` | `0x{:016X}` | Machine interrupt pending |",
                dump.core_csrs.mip
            )?;
            writeln!(
                f,
                "| `mcycle` | `0x{:016X}` | Machine cycle counter |",
                dump.core_csrs.mcycle
            )?;
            writeln!(
                f,
                "| `minstret` | `0x{:016X}` | Machine instructions retired counter |",
                dump.core_csrs.minstret
            )?;
            writeln!(
                f,
                "| `mvendorid` | `0x{:016X}` | Vendor ID |",
                dump.core_csrs.mvendorid
            )?;
            writeln!(
                f,
                "| `marchid` | `0x{:016X}` | Architecture ID |",
                dump.core_csrs.marchid
            )?;
            writeln!(
                f,
                "| `mimpid` | `0x{:016X}` | Implementation ID |",
                dump.core_csrs.mimpid
            )?;
            writeln!(
                f,
                "| `mhartid` | `0x{:016X}` | Hardware thread ID |",
                dump.core_csrs.mhartid
            )?;
            writeln!(f)?;

            if let Some(fp_regs) = &dump.float_registers {
                writeln!(f, "### 🔣 All Floating-Point Registers")?;
                writeln!(f)?;
                writeln!(f, "| Register | Value |")?;
                writeln!(f, "|----------|-------|")?;
                // Show all floating-point registers
                for (i, &val) in fp_regs.iter().enumerate() {
                    writeln!(f, "| `f{}` | `0x{:016X}` |", i, val)?;
                }
                writeln!(f)?;
            }

            if let Some(fcsr) = dump.float_csr {
                writeln!(f, "**Floating-Point CSR:** `fcsr = 0x{:016X}`", fcsr)?;
                writeln!(f)?;
            }
        } else {
            writeln!(f, "## 📝 `{}` Final Register Dump", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "> ❌ **No register dump**")?;
            writeln!(f)?;
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "*Generated at: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

impl OutputParser for StandardExecutionOutput {
    fn parse_from_file<P: AsRef<Path>>(
        log_path: P,
        dump_path: P,
        emulator_type: EmulatorType,
    ) -> Result<Self> {
        parse_standard_output_from_file(log_path, dump_path, emulator_type)
    }
}

pub fn parse_standard_output_from_file<P: AsRef<Path>>(
    file_path: P,
    dump_path: P,
    emulator_type: EmulatorType,
) -> Result<StandardExecutionOutput> {
    let common_output = common::parse_common_output_from_file(file_path, dump_path, emulator_type)?;
    let mut warnings = Vec::new();
    let mut conversion_successful = true;

    // Extract final exceptions and register dump from common_output
    let final_exceptions = common_output.exception_dumps; // Already Vec<ExceptionDump>
    let final_register_dump = common_output.register_dumps.last().cloned(); // Already Option<RegistersDump>

    let original_exception_count = final_exceptions.len();
    let original_register_count = common_output.register_dumps.len(); // Total register dumps found by common parser

    // The loop for processing common_output.output_items to find the *final* dump
    // and all exceptions is now simplified by using common_output.register_dumps
    // and common_output.exception_dumps directly.
    // The logic for 'warnings' and 'conversion_successful' might still depend on
    // specific items if needed, but for now, we assume common_output is sufficient.

    // Example: Check for unknown binary data as a warning
    for item in common_output.output_items.iter() {
        if let OutputItem::UnknownBinary { data, position } = item {
            warnings.push(format!(
                "Unknown binary data ({} bytes) at position {}",
                data.len(),
                position
            ));
            conversion_successful = false; // Or handle as a less severe warning
        }
    }

    // Deduplicate exceptions based on (mepc, mcause, mtval)
    // This was part of the original logic, let's ensure it's preserved or adapted.
    // The `final_exceptions` from `common_output.exception_dumps` are already `ExceptionDump`.
    // If deduplication is needed here, it should be applied to `final_exceptions`.
    // For now, we assume `common_output.exception_dumps` is the desired list.

    Ok(StandardExecutionOutput {
        emulator_type,
        exceptions: final_exceptions,
        register_dump: final_register_dump,
        conversion_stats: ConversionStats {
            original_exception_count,
            original_register_count,
            conversion_successful,
            warnings,
        },
    })
}
