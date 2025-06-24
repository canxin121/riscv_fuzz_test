use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path; // Added for Display trait

use crate::output_parser::common::parse_common_output_from_file;
use crate::output_parser::util::get_register_name;
use crate::{
    emulators::EmulatorType,
    error::Result,
    output_parser::{
        ExceptionCSRs, // Added back as it's used in DebugExecutionOutputItem
        MarkerType,
        OutputParser,
        RegistersDump,      // Removed CoreCSRs (unused in this file directly)
        common::OutputItem, // Removed common::self
    },
}; // Added import

/// Debug output single parsing item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugExecutionOutputItem {
    /// Marker
    Marker(MarkerType, usize), // MarkerType, Position
    /// Register dump info (not full data, just metadata)
    RegisterDumpInfo(MarkerType, usize, usize), // MarkerType, RegisterCount, Position
    /// Exception info (not full data, just metadata)
    ExceptionInfo(ExceptionCSRs, usize), // ExceptionCSRs, Position
    /// Text
    Text(String),
    /// Unknown data block
    Unknown(usize, usize), // Length, Position
}

impl fmt::Display for DebugExecutionOutputItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugExecutionOutputItem::Marker(marker_type, pos) => {
                write!(f, "Marker @{}: {:?}", pos, marker_type)
            }
            DebugExecutionOutputItem::RegisterDumpInfo(marker_type, count, pos) => {
                write!(
                    f,
                    "Register dump info @{}: {:?} ({} registers)",
                    pos, marker_type, count
                )
            }
            DebugExecutionOutputItem::ExceptionInfo(csrs, pos) => {
                write!(
                    f,
                    "Exception info @{}: MEPC=0x{:X}, MCAUSE=0x{:X}",
                    pos, csrs.mepc, csrs.mcause
                )
            }
            DebugExecutionOutputItem::Text(text) => {
                // Remove truncation, show complete text content
                write!(f, "Text: \"{}\"", text.replace('\n', "\\n"))
            }
            DebugExecutionOutputItem::Unknown(len, pos) => {
                write!(f, "Unknown data @{}: {} bytes", pos, len)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugExecutionOutput {
    /// Emulator type
    pub emulator_type: EmulatorType,
    /// Raw data length
    pub raw_data_length: usize,
    /// Parsed debug items
    pub parsed_debug_items: Vec<DebugExecutionOutputItem>,
    /// Valid register dumps
    pub register_dumps: Vec<RegistersDump>,
    /// Total dump count (including valid and invalid)
    pub total_dumps: usize,
}

impl Default for DebugExecutionOutput {
    fn default() -> Self {
        Self {
            emulator_type: EmulatorType::Spike,
            raw_data_length: 0,
            // output_items: Vec::new(), // Removed field
            parsed_debug_items: Vec::new(),
            register_dumps: Vec::new(),
            total_dumps: 0,
        }
    }
}

impl OutputParser for DebugExecutionOutput {
    fn parse_from_file<P: AsRef<Path>>(
        log_path: P,
        dump_path: P,
        emulator_type: EmulatorType,
    ) -> Result<Self> {
        parse_debug_output_from_file(log_path, dump_path, emulator_type)
    }
}

pub fn parse_debug_output_from_file<P: AsRef<Path>>(
    log_path: P,
    dump_path: P,
    emulator_type: EmulatorType,
) -> Result<DebugExecutionOutput> {
    let common_output = parse_common_output_from_file(log_path, dump_path, emulator_type)?;

    let mut parsed_debug_items = Vec::new();
    let mut total_dumps_encountered = 0;

    for item in &common_output.output_items {
        match item {
            OutputItem::MagicMarker {
                marker_type,
                position,
                ..
            } => {
                total_dumps_encountered += 1;
                parsed_debug_items.push(DebugExecutionOutputItem::Marker(
                    marker_type.clone(),
                    *position,
                )); // Clone marker_type
            }
            OutputItem::RegisterData {
                marker_type,
                registers,
                position,
            } => {
                // This case implies a valid register dump was found by the common parser
                // We need to reconstruct a RegistersDump from this if common_output.register_dumps
                // isn't already what we want.
                // For simplicity, let's assume common_output.register_dumps is the source of truth
                // for valid dumps. This loop is more for counting and creating DebugExecutionOutputItem.
                parsed_debug_items.push(DebugExecutionOutputItem::RegisterDumpInfo(
                    marker_type.clone(), // Clone marker_type
                    registers.len(),     // or a fixed size based on marker_type
                    *position,
                ));
            }
            OutputItem::ExceptionData { csrs, position } => {
                total_dumps_encountered += 1; // Counting exceptions as a "dump" type for total_dumps
                parsed_debug_items.push(DebugExecutionOutputItem::ExceptionInfo(
                    csrs.clone(),
                    *position,
                ));
            }
            OutputItem::AsciiText(text) => {
                parsed_debug_items.push(DebugExecutionOutputItem::Text(text.clone()));
            }
            OutputItem::UnknownBinary { data, position } => {
                parsed_debug_items.push(DebugExecutionOutputItem::Unknown(data.len(), *position));
            }
        }
    }

    // Use the register_dumps directly from common_output
    let parsed_dumps = common_output.register_dumps; // Assign directly

    Ok(DebugExecutionOutput {
        emulator_type,
        raw_data_length: common_output.raw_data_length,
        // output_items: common_output.output_items, // Removed field
        parsed_debug_items,
        register_dumps: parsed_dumps,
        total_dumps: total_dumps_encountered,
    })
}

impl fmt::Display for DebugExecutionOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# 🔧 RISC-V Debug Execution Output")?;
        writeln!(f)?;
        writeln!(f, "**Emulator Type:** `{}`", self.emulator_type)?;
        writeln!(f)?;

        // Basic information table
        writeln!(f, "## 📊 Basic Information")?;
        writeln!(f)?;
        writeln!(f, "| Item | Value |")?;
        writeln!(f, "|------|-------|")?;
        writeln!(f, "| Raw Data Length | `{} bytes` |", self.raw_data_length)?;
        writeln!(
            f,
            "| Parsed Debug Item Count | `{}` |",
            self.parsed_debug_items.len()
        )?;
        writeln!(
            f,
            "| Valid Register Dump Count | `{}` |",
            self.register_dumps.len()
        )?;
        writeln!(f, "| Total Dump Markers Encountered | `{}` |", self.total_dumps)?;
        writeln!(f)?;

        // Debug item details
        if !self.parsed_debug_items.is_empty() {
            writeln!(f, "## 📋 `{}` Parsed Debug Items", self.emulator_type)?;
            writeln!(f)?;

            // Count different types of debug items
            let mut marker_count = 0;
            let mut register_info_count = 0;
            let mut exception_info_count = 0;
            let mut text_count = 0;
            let mut unknown_count = 0;

            for item in &self.parsed_debug_items {
                match item {
                    DebugExecutionOutputItem::Marker(_, _) => marker_count += 1,
                    DebugExecutionOutputItem::RegisterDumpInfo(_, _, _) => register_info_count += 1,
                    DebugExecutionOutputItem::ExceptionInfo(_, _) => exception_info_count += 1,
                    DebugExecutionOutputItem::Text(_) => text_count += 1,
                    DebugExecutionOutputItem::Unknown(_, _) => unknown_count += 1,
                }
            }

            writeln!(f, "### 📈 Debug Item Type Statistics")?;
            writeln!(f)?;
            writeln!(f, "| Type | Count | Description |")?;
            writeln!(f, "|------|-------|-------------|")?;
            writeln!(f, "| 🔻 Marker Items | `{}` | Data segment markers |", marker_count)?;
            writeln!(
                f,
                "| 📋 Register Dump Info | `{}` | Register dump metadata |",
                register_info_count
            )?;
            writeln!(
                f,
                "| 🚨 Exception Info | `{}` | Exception and interrupt info |",
                exception_info_count
            )?;
            writeln!(f, "| 📝 Text Items | `{}` | Readable text output |", text_count)?;
            writeln!(f, "| ❓ Unknown Data | `{}` | Unrecognized data blocks |", unknown_count)?;
            writeln!(f)?;

            writeln!(f, "### 🔍 Debug Item Details (Complete List)")?;
            writeln!(f)?;

            // Show all debug items completely without truncation
            for (i, item) in self.parsed_debug_items.iter().enumerate() {
                match item {
                    DebugExecutionOutputItem::Marker(marker_type, pos) => {
                        writeln!(
                            f,
                            "**[{}]** 🔻 **Marker:** `{:?}` @position`{}`",
                            i + 1,
                            marker_type,
                            pos
                        )?;
                    }
                    DebugExecutionOutputItem::RegisterDumpInfo(marker_type, count, pos) => {
                        writeln!(
                            f,
                            "**[{}]** 📋 **Register Dump Info:** `{:?}` ({} registers) @position`{}`",
                            i + 1,
                            marker_type,
                            count,
                            pos
                        )?;
                    }
                    DebugExecutionOutputItem::ExceptionInfo(csrs, pos) => {
                        writeln!(
                            f,
                            "**[{}]** 🚨 **Exception Info:** MEPC=`0x{:X}`, MCAUSE=`0x{:X}` @position`{}`",
                            i + 1,
                            csrs.mepc,
                            csrs.mcause,
                            pos
                        )?;
                    }
                    DebugExecutionOutputItem::Text(text) => {
                        // Show complete text content without truncation
                        writeln!(f, "**[{}]** 📝 **Text:** `{}`", i + 1, text)?;
                    }
                    DebugExecutionOutputItem::Unknown(len, pos) => {
                        writeln!(
                            f,
                            "**[{}]** ❓ **Unknown Data:** `{} bytes` @position`{}`",
                            i + 1,
                            len,
                            pos
                        )?;
                    }
                }
            }
            writeln!(f)?;
        }

        // Valid register dump details
        if !self.register_dumps.is_empty() {
            writeln!(f, "## 📝 `{}` Valid Register Dumps", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "**Total:** `{} valid dumps`", self.register_dumps.len())?;
            writeln!(f)?;

            // Show all dumps completely without truncation
            for (i, dump) in self.register_dumps.iter().enumerate() {
                writeln!(f, "### 📊 Dump #{} (Position: `{}`)", i + 1, dump.position)?;
                writeln!(f)?;
                writeln!(f, "**Dump Type:** `{:?}`", dump.dump_type)?;
                writeln!(f)?;

                // Key register overview - show all registers
                writeln!(f, "#### 🎯 All Integer Registers")?;
                writeln!(f)?;
                writeln!(f, "| Register | ABI Name | Value | Description |")?;
                writeln!(f, "|----------|----------|-------|-------------|")?;

                for reg_idx in 0..32 {
                    let reg_name = get_register_name(reg_idx);
                    let value = dump.int_registers[reg_idx];
                    let description = match reg_idx {
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
                        "| `x{:02}` | `{}` | `0x{:016X}` | {} |",
                        reg_idx, reg_name, value, description
                    )?;
                }
                writeln!(f)?;

                // Core CSR overview - show all CSRs
                writeln!(f, "#### ⚙️ All Core CSRs")?;
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

                // Floating-point register details - show all floating-point registers
                if let Some(float_regs) = &dump.float_registers {
                    writeln!(f, "#### 🔣 All Floating-Point Registers")?;
                    writeln!(f)?;
                    writeln!(f, "| Register | Value |")?;
                    writeln!(f, "|----------|-------|")?;
                    for (i, &val) in float_regs.iter().enumerate() {
                        writeln!(f, "| `f{}` | `0x{:016X}` |", i, val)?;
                    }
                    writeln!(f)?;
                }

                if let Some(fcsr) = dump.float_csr {
                    writeln!(f, "**Floating-Point CSR:** `fcsr = 0x{:016X}`", fcsr)?;
                    writeln!(f)?;
                }

                // Statistics
                let non_zero_int = dump
                    .int_registers
                    .iter()
                    .skip(1)
                    .filter(|&&x| x != 0)
                    .count();
                writeln!(f, "> **Statistics:** Non-zero integer registers: `{}/31`", non_zero_int)?;

                if let Some(float_regs) = &dump.float_registers {
                    let non_zero_float = float_regs.iter().filter(|&&x| x != 0).count();
                    writeln!(f, "> Non-zero floating-point registers: `{}/32`", non_zero_float)?;
                }
                writeln!(f)?;

                if i < self.register_dumps.len() - 1 {
                    writeln!(f)?;
                }
            }
        } else {
            writeln!(f, "## 📝 `{}` Valid Register Dumps", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "> ❌ **No valid register dumps**")?;
            writeln!(f)?;
        }

        // Data analysis statistics
        writeln!(f, "## 📈 Data Analysis Statistics")?;
        writeln!(f)?;
        writeln!(f, "| Statistics Item | Value |")?;
        writeln!(f, "|-----------------|-------|")?;
        writeln!(
            f,
            "| Dump Efficiency | `{:.1}%` ({}/{} dumps valid) |",
            if self.total_dumps > 0 {
                (self.register_dumps.len() as f64 / self.total_dumps as f64) * 100.0
            } else {
                0.0
            },
            self.register_dumps.len(),
            self.total_dumps
        )?;

        // Data type distribution
        let total_items = self.parsed_debug_items.len();
        if total_items > 0 {
            let marker_ratio = self
                .parsed_debug_items
                .iter()
                .filter(|item| matches!(item, DebugExecutionOutputItem::Marker(_, _)))
                .count() as f64
                / total_items as f64
                * 100.0;
            writeln!(f, "| Marker Ratio | `{:.1}%` |", marker_ratio)?;
        }
        writeln!(f)?;

        writeln!(f, "---")?;
        writeln!(
            f,
            "*Generated at: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

/// Format debug output result as readable string
pub fn format_debug_output(output: &DebugExecutionOutput) -> String {
    let mut result = String::new();

    result.push_str("Debug Format Output Summary:\n");
    result.push_str(&format!(
        "  Raw data length: {} bytes\n",
        output.raw_data_length
    ));
    result.push_str(&format!(
        "  Total register dumps: {}\n\n",
        output.total_dumps
    ));

    for (i, dump) in output.register_dumps.iter().enumerate() {
        result.push_str(&format!(
            "Register Dump #{} (at position {}):\n",
            i + 1,
            dump.position
        ));
        result.push_str(&format!("  Type: {:?}\n", dump.dump_type));

        // Display register values
        result.push_str("  Integer registers:\n");
        for j in 0..32 {
            if j % 4 == 0 && j > 0 {
                result.push('\n');
            }
            result.push_str(&format!("    x{:2}: 0x{:016X}", j, dump.int_registers[j]));
        }
        result.push('\n');

        if let Some(float_regs) = &dump.float_registers {
            result.push_str("  Float registers:\n");
            for j in 0..32 {
                if j % 4 == 0 && j > 0 {
                    result.push('\n');
                }
                result.push_str(&format!("    f{:2}: 0x{:016X}", j, float_regs[j]));
            }
            result.push('\n');
        }

        result.push('\n');
    }

    result
}
