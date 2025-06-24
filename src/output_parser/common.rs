use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::{fmt, fs};
use std::path::Path;

use super::{
     MarkerType, RegistersDump, CoreCSRs, ExceptionDump, ExceptionCSRs,
    MARKER_EXCEPTION_CSR, MARKER_REGISTERS_INT_AND_FLOAT, MARKER_REGISTERS_INT_ONLY,
};
use crate::elf::tracer::ElfTracer;
use crate::output_parser::util;
use crate::{error::Result, output_parser::OutputParser, emulators::EmulatorType};

// --- Moved from common.rs ---
/// Program execution output parsing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonExecutionOutput {
    /// Emulator type
    pub emulator_type: EmulatorType,
    /// Raw data length
    pub raw_data_length: usize,
    /// All parsed output items
    pub output_items: Vec<OutputItem>,
    /// Register dumps (if any)
    pub register_dumps: Vec<RegistersDump>,
    /// Exception CSR dumps (if any)
    pub exception_dumps: Vec<ExceptionDump>,
}

impl fmt::Display for CommonExecutionOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# 🔍 RISC-V Common Execution Output Analysis")?;
        writeln!(f)?;
        writeln!(f, "**Emulator Type:** `{}`", self.emulator_type)?;
        writeln!(f)?;
        
        // Basic information table
        writeln!(f, "## 📊 Basic Information")?;
        writeln!(f)?;
        writeln!(f, "| Item | Value |")?;
        writeln!(f, "|------|-------|")?;
        writeln!(f, "| Raw Data Size | `{} bytes` |", self.raw_data_length)?;
        writeln!(f, "| Total Output Items | `{}` |", self.output_items.len())?;
        writeln!(f, "| Register Dump Count | `{}` |", self.register_dumps.len())?;
        writeln!(f, "| Exception Dump Count | `{}` |", self.exception_dumps.len())?;
        writeln!(f)?;

        // Output item details
        if !self.output_items.is_empty() {
            writeln!(f, "## 📋 Output Item Details")?;
            writeln!(f)?;

            // Count various types of output items
            let mut ascii_count = 0;
            let mut marker_count = 0;
            let mut register_data_count = 0;
            let mut exception_data_count = 0;
            let mut unknown_binary_count = 0;

            for item in &self.output_items {
                match item {
                    OutputItem::AsciiText(_) => ascii_count += 1,
                    OutputItem::MagicMarker { .. } => marker_count += 1,
                    OutputItem::RegisterData { .. } => register_data_count += 1,
                    OutputItem::ExceptionData { .. } => exception_data_count += 1,
                    OutputItem::UnknownBinary { .. } => unknown_binary_count += 1,
                }
            }

            writeln!(f, "### 📈 Type Statistics")?;
            writeln!(f)?;
            writeln!(f, "| Type | Count | Description |")?;
            writeln!(f, "|------|-------|-------------|")?;
            writeln!(f, "| 📝 ASCII Text Items | `{}` | Readable text output |", ascii_count)?;
            writeln!(f, "| 🔻 Magic Marker Items | `{}` | Data segment markers |", marker_count)?;
            writeln!(f, "| 📋 Register Data Items | `{}` | Register dump data |", register_data_count)?;
            writeln!(f, "| 🚨 Exception Data Items | `{}` | Exception and interrupt info |", exception_data_count)?;
            writeln!(f, "| ❓ Unknown Binary Items | `{}` | Unrecognized binary data |", unknown_binary_count)?;
            writeln!(f)?;

            // Show all output items without truncation
            writeln!(f, "### 🔍 Item Details (Complete List)")?;
            writeln!(f)?;

            for (i, item) in self.output_items.iter().enumerate() {
                match item {
                    OutputItem::AsciiText(text) => {
                        // Don't truncate text content
                        writeln!(f, "**[{}]** 📝 **ASCII Text:** `{}`", i + 1, text)?;
                    }
                    OutputItem::MagicMarker {
                        marker,
                        marker_type,
                        position,
                    } => {
                        writeln!(
                            f,
                            "**[{}]** 🔻 **Marker:** `{}` (`0x{:016X}`) @position`{}`",
                            i + 1,
                            marker_type,
                            marker,
                            position
                        )?;
                    }
                    OutputItem::RegisterData {
                        marker_type,
                        registers,
                        position,
                    } => {
                        writeln!(
                            f,
                            "**[{}]** 📋 **Registers:** `{}` ({} registers) @position`{}`",
                            i + 1,
                            marker_type,
                            registers.len(),
                            position
                        )?;
                    }
                    OutputItem::ExceptionData { position, .. } => {
                        writeln!(f, "**[{}]** 🚨 **Exception Data** @position`{}`", i + 1, position)?;
                    }
                    OutputItem::UnknownBinary { data, position } => {
                        writeln!(
                            f,
                            "**[{}]** ❓ **Unknown Data:** `{} bytes` @position`{}`",
                            i + 1,
                            data.len(),
                            position
                        )?;
                    }
                }
            }
            writeln!(f)?;
        }

        // Register dump details - show all dumps without truncation
        if !self.register_dumps.is_empty() {
            writeln!(f, "## 📋 `{}` Register Dump Details", self.emulator_type)?;
            writeln!(f)?;

            for (i, dump) in self.register_dumps.iter().enumerate() {
                writeln!(f, "### 📊 Register Dump #{} (Position: `{}`)", i + 1, dump.position)?;
                writeln!(f)?;
                writeln!(f, "**Dump Type:** `{}`", dump.dump_type)?;
                writeln!(f)?;

                // Show all integer registers
                writeln!(f, "#### 🔢 All Integer Registers (x0-x31)")?;
                writeln!(f)?;
                writeln!(f, "| Register | ABI Name | Value | Description |")?;
                writeln!(f, "|----------|----------|-------|-------------|")?;
                
                for reg_idx in 0..32 {
                    let reg_name = util::get_register_name(reg_idx);
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
                        _ => unreachable!(),
                    };

                    writeln!(
                        f,
                        "| `x{:02}` | `{:>4}` | `0x{:016X}` | {} |",
                        reg_idx, reg_name, value, description
                    )?;
                }
                writeln!(f)?;

                // Show all core CSR registers
                writeln!(f, "#### ⚙️ All Core CSR Registers")?;
                writeln!(f)?;
                writeln!(f, "| CSR Register | Value | Description |")?;
                writeln!(f, "|--------------|-------|-------------|")?;
                writeln!(f, "| `mstatus` | `0x{:016X}` | Machine status register |", dump.core_csrs.mstatus)?;
                writeln!(f, "| `misa` | `0x{:016X}` | ISA and extensions |", dump.core_csrs.misa)?;
                writeln!(f, "| `medeleg` | `0x{:016X}` | Machine exception delegation |", dump.core_csrs.medeleg)?;
                writeln!(f, "| `mideleg` | `0x{:016X}` | Machine interrupt delegation |", dump.core_csrs.mideleg)?;
                writeln!(f, "| `mie` | `0x{:016X}` | Machine interrupt enable |", dump.core_csrs.mie)?;
                writeln!(f, "| `mtvec` | `0x{:016X}` | Machine trap vector base address |", dump.core_csrs.mtvec)?;
                writeln!(f, "| `mcounteren` | `0x{:016X}` | Machine counter enable |", dump.core_csrs.mcounteren)?;
                writeln!(f, "| `mscratch` | `0x{:016X}` | Machine scratch register |", dump.core_csrs.mscratch)?;
                writeln!(f, "| `mepc` | `0x{:016X}` | Machine exception program counter |", dump.core_csrs.mepc)?;
                writeln!(f, "| `mcause` | `0x{:016X}` | Machine trap cause |", dump.core_csrs.mcause)?;
                writeln!(f, "| `mtval` | `0x{:016X}` | Machine bad address or instruction |", dump.core_csrs.mtval)?;
                writeln!(f, "| `mip` | `0x{:016X}` | Machine interrupt pending |", dump.core_csrs.mip)?;
                writeln!(f, "| `mcycle` | `0x{:016X}` | Machine cycle counter |", dump.core_csrs.mcycle)?;
                writeln!(f, "| `minstret` | `0x{:016X}` | Machine instructions retired counter |", dump.core_csrs.minstret)?;
                writeln!(f, "| `mvendorid` | `0x{:016X}` | Vendor ID |", dump.core_csrs.mvendorid)?;
                writeln!(f, "| `marchid` | `0x{:016X}` | Architecture ID |", dump.core_csrs.marchid)?;
                writeln!(f, "| `mimpid` | `0x{:016X}` | Implementation ID |", dump.core_csrs.mimpid)?;
                writeln!(f, "| `mhartid` | `0x{:016X}` | Hardware thread ID |", dump.core_csrs.mhartid)?;
                writeln!(f)?;

                // Show all floating-point registers (if present)
                if let Some(float_regs) = &dump.float_registers {
                    writeln!(f, "#### 🔣 All Floating-Point Registers (f0-f31)")?;
                    writeln!(f)?;
                    writeln!(f, "| Register | ABI Name | Value | Description |")?;
                    writeln!(f, "|----------|----------|-------|-------------|")?;
                    
                    for reg_idx in 0..32 {
                        let (reg_abi_name, description) = match reg_idx {
                            0..=7 => (format!("ft{}", reg_idx), "Temporary floating-point register"),
                            8..=9 => (format!("fs{}", reg_idx - 8), "Saved floating-point register"),
                            10..=17 => (format!("fa{}", reg_idx - 10), "Floating-point argument/return value"),
                            18..=27 => (format!("fs{}", reg_idx - 18 + 2), "Saved floating-point register"),
                            28..=31 => (format!("ft{}", reg_idx - 28 + 8), "Temporary floating-point register"),
                            _ => unreachable!(),
                        };

                        writeln!(
                            f,
                            "| `f{:02}` | `{:>4}` | `0x{:016X}` | {} |",
                            reg_idx, reg_abi_name, float_regs[reg_idx], description
                        )?;
                    }
                    writeln!(f)?;

                    if let Some(fcsr) = dump.float_csr {
                        writeln!(f, "**Floating-Point Control and Status Register:** `fcsr = 0x{:016X}`", fcsr)?;
                        writeln!(f)?;
                    }
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
        }

        // Exception dump details - show all exceptions without truncation
        if !self.exception_dumps.is_empty() {
            writeln!(f, "## 🚨 `{}` Exception Dump Details", self.emulator_type)?;
            writeln!(f)?;

            for (i, dump) in self.exception_dumps.iter().enumerate() {
                let exception_desc = util::get_exception_description(dump.csrs.mcause);
                let is_interrupt = (dump.csrs.mcause >> 63) & 1 == 1;
                let exception_type = if is_interrupt { "Interrupt" } else { "Exception" };

                writeln!(f, "### ⚡ Exception Dump #{} (Position: `{}`)", i + 1, dump.position)?;
                writeln!(f)?;
                writeln!(f, "**Exception PC:** `0x{:016X}`", dump.csrs.mepc)?;
                if let Some(trace) = &dump.inst_trace {
                    writeln!(f, "**Traced Instruction:** `{}`", trace.disassembly)?;
                    writeln!(f, "**Machine Code:** `{}`", trace.machine_code)?;
                    writeln!(f, "**Original Instruction:** `{}`", trace.original_instruction)?;
                }
                writeln!(f, "**Type:** `{}` ({})", exception_desc, exception_type)?;
                writeln!(f)?;

                writeln!(f, "#### CSR Details")?;
                writeln!(f)?;
                writeln!(f, "| CSR Register | Value | Description |")?;
                writeln!(f, "|--------------|-------|-------------|")?;
                writeln!(f, "| `mcause` | `0x{:016X}` | {} |", dump.csrs.mcause, exception_desc)?;
                writeln!(f, "| `mtval` | `0x{:016X}` | Machine bad address or instruction |", dump.csrs.mtval)?;
                writeln!(f, "| `mstatus` | `0x{:016X}` | Machine status register |", dump.csrs.mstatus)?;
                writeln!(f, "| `mtvec` | `0x{:016X}` | Machine trap vector base address |", dump.csrs.mtvec)?;
                writeln!(f, "| `mie` | `0x{:016X}` | Machine interrupt enable |", dump.csrs.mie)?;
                writeln!(f, "| `mip` | `0x{:016X}` | Machine interrupt pending |", dump.csrs.mip)?;
                writeln!(f, "| `mscratch` | `0x{:016X}` | Machine scratch register |", dump.csrs.mscratch)?;
                writeln!(f, "| `mhartid` | `0x{:016X}` | Hardware thread ID |", dump.csrs.mhartid)?;
                writeln!(f)?;

                if i < self.exception_dumps.len() - 1 {
                    writeln!(f)?;
                }
            }
        }

        // Data analysis statistics
        writeln!(f, "## 📈 Data Analysis Statistics")?;
        writeln!(f)?;

        let total_ascii_chars: usize = self
            .output_items
            .iter()
            .filter_map(|item| match item {
                OutputItem::AsciiText(text) => Some(text.len()),
                _ => None,
            })
            .sum();

        let total_binary_bytes: usize = self
            .output_items
            .iter()
            .filter_map(|item| match item {
                OutputItem::UnknownBinary { data, .. } => Some(data.len()),
                _ => None,
            })
            .sum();

        writeln!(f, "| Statistics Item | Value |")?;
        writeln!(f, "|-----------------|-------|")?;
        if total_ascii_chars > 0 {
            writeln!(f, "| 📝 Total ASCII Character Count | `{}` |", total_ascii_chars)?;
        }
        if total_binary_bytes > 0 {
            writeln!(f, "| ❓ Total Unknown Binary Data Bytes | `{}` |", total_binary_bytes)?;
        }

        // Exception type statistics
        if !self.exception_dumps.is_empty() {
            use std::collections::HashMap;
            let mut exception_types: HashMap<String, usize> = HashMap::new();

            for dump in &self.exception_dumps {
                let desc = util::get_exception_description(dump.csrs.mcause);
                *exception_types.entry(desc).or_insert(0) += 1;
            }

            let mut sorted_types: Vec<_> = exception_types.into_iter().collect();
            sorted_types.sort_by(|a, b| b.1.cmp(&a.1));

            writeln!(f)?;
            writeln!(f, "### 🚨 Exception Type Distribution")?;
            writeln!(f)?;
            writeln!(f, "| Exception Type | Occurrence Count |")?;
            writeln!(f, "|----------------|------------------|")?;
            for (exception_type, count) in sorted_types {
                writeln!(f, "| {} | `{}` |", exception_type, count)?;
            }
            writeln!(f)?;
        }

        // Register dump type statistics
        if !self.register_dumps.is_empty() {
            let int_only_count = self
                .register_dumps
                .iter()
                .filter(|d| matches!(d.dump_type, MarkerType::RegistersIntOnly))
                .count();
            let int_float_count = self
                .register_dumps
                .iter()
                .filter(|d| matches!(d.dump_type, MarkerType::RegistersIntAndFloat))
                .count();

            writeln!(f, "### 📋 Register Dump Type Distribution")?;
            writeln!(f)?;
            writeln!(f, "| Dump Type | Count |")?;
            writeln!(f, "|-----------|-------|")?;
            if int_only_count > 0 {
                writeln!(f, "| Integer Registers Only | `{}` |", int_only_count)?;
            }
            if int_float_count > 0 {
                writeln!(f, "| Integer + Floating-Point Registers | `{}` |", int_float_count)?;
            }
            writeln!(f)?;
        }

        // Data coverage analysis
        let parsed_bytes = self
            .output_items
            .iter()
            .map(|item| {
                match item {
                    OutputItem::AsciiText(text) => text.len() + 1,
                    OutputItem::MagicMarker { .. } => 8,
                    OutputItem::RegisterData { registers, .. } => registers.len() * 8,
                    OutputItem::ExceptionData { .. } => 72,
                    OutputItem::UnknownBinary { data, .. } => data.len(),
                }
            })
            .sum::<usize>();

        let coverage_ratio = if self.raw_data_length > 0 {
            (parsed_bytes as f64 / self.raw_data_length as f64) * 100.0
        } else {
            0.0
        };

        writeln!(f, "| 📊 Data Coverage Rate | `{:.1}%` ({}/{} bytes) |", coverage_ratio, parsed_bytes, self.raw_data_length)?;
        writeln!(f)?;

        writeln!(f, "---")?;
        writeln!(f, "*Generated at: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;

        Ok(())
    }
}

impl OutputParser for CommonExecutionOutput {
    fn parse_from_file<P: AsRef<Path>>(
        log_path: P,
        dump_path: P,
        emulator_type: EmulatorType,
    ) -> Result<Self>{
        parse_common_output_from_file(log_path, dump_path, emulator_type)
    }

}



/// Output item type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputItem {
    /// ASCII text output
    AsciiText(String),
    /// Magic marker
    MagicMarker {
        marker: u64,
        marker_type: MarkerType,
        position: usize,
    },
    /// Register dump data
    RegisterData {
        marker_type: MarkerType,
        registers: Vec<u64>,
        position: usize,
    },
    /// Exception CSR dump data
    ExceptionData {
        csrs: ExceptionCSRs,
        position: usize,
    },
    /// Unknown binary data
    UnknownBinary { data: Vec<u8>, position: usize },
}
/// Parse execution output from file
pub fn parse_common_output_from_file<P: AsRef<Path>>(
    log_path: P,
    dump_path: P,
    emulator_type: EmulatorType,
) -> Result<CommonExecutionOutput> {
    if !dump_path.as_ref().exists() {
        return Err(crate::error::RiscvFuzzError::Config {
            message: "ELF dump file not found".into(),
        });
    }

    let data = fs::read(log_path.as_ref())?;
    debug!(
        "📄 Reading output file: {} ({} bytes) for emulator {:?}",
        log_path.as_ref().display(),
        data.len(),
        emulator_type
    );
    let mut result = parse_common_binary_data(&data, emulator_type)?;

    // If there are exceptions, try to trace instructions from ELF dump
    if !result.exception_dumps.is_empty() {
        if dump_path.as_ref().exists() {
            debug!(
                "Found ELF dump at {}, attempting to trace exceptions.",
                dump_path.as_ref().display()
            );
            match ElfTracer::new(&dump_path) {
                Ok(tracer) => {
                    for dump in result.exception_dumps.iter_mut() {
                        dump.inst_trace = tracer.trace_pc(dump.csrs.mepc);
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to create ELF tracer from {}: {}",
                        dump_path.as_ref().display(),
                        e
                    );
                }
            }
        } else {
            return Err(crate::error::RiscvFuzzError::Config { message: "ELF dump file not found".into() });
        }
    }

    Ok(result)
}

/// Parse binary data
pub fn parse_common_binary_data(data: &[u8], emulator_type: EmulatorType) -> Result<CommonExecutionOutput> {
    let mut result = CommonExecutionOutput {
        emulator_type,
        raw_data_length: data.len(),
        output_items: Vec::new(),
        register_dumps: Vec::new(),
        exception_dumps: Vec::new(),
    };

    if data.is_empty() {
        return Ok(result);
    }

    let mut pos = 0;
    
    while pos < data.len() {
        // Try to find printable ASCII text
        if let Some((text, consumed)) = try_parse_ascii_text(&data[pos..]) {
            if !text.is_empty() {
                debug!("📝 Found ASCII text at position {}: {:?}", pos, text);
                result.output_items.push(OutputItem::AsciiText(text));
            }
            pos += consumed;
            continue;
        }

        // Try to parse 8-byte magic marker
        if pos + 8 <= data.len() {
            let potential_marker = read_u64_le(&data[pos..pos + 8]);
            
            if let Some(marker_type) = get_marker_type(potential_marker) {
                debug!("🔍 Found known marker 0x{:016X} ({:?}) at position {}", 
                       potential_marker, marker_type, pos);
                
                result.output_items.push(OutputItem::MagicMarker {
                    marker: potential_marker,
                    marker_type: marker_type.clone(),
                    position: pos,
                });
                
                pos += 8;
                
                // Parse subsequent data based on marker type
                match marker_type {
                    MarkerType::RegistersIntOnly => {
                        if let Some((registers, core_csrs, consumed)) = parse_int_registers(&data[pos..]) {
                            let dump = RegistersDump {
                                dump_type: marker_type.clone(),
                                int_registers: registers,
                                core_csrs: core_csrs.clone(),
                                float_registers: None,
                                float_csr: None,
                                position: pos - 8,
                            };
                            result.register_dumps.push(dump.clone());
                            let mut all_data = registers.to_vec();
                            all_data.extend_from_slice(&[
                                core_csrs.mstatus, core_csrs.misa, core_csrs.medeleg, core_csrs.mideleg,
                                core_csrs.mie, core_csrs.mtvec, core_csrs.mcounteren, core_csrs.mscratch,
                                core_csrs.mepc, core_csrs.mcause, core_csrs.mtval, core_csrs.mip,
                                core_csrs.mcycle, core_csrs.minstret, core_csrs.mvendorid, core_csrs.marchid,
                                core_csrs.mimpid, core_csrs.mhartid
                            ]);
                            result.output_items.push(OutputItem::RegisterData {
                                marker_type: marker_type.clone(),
                                registers: all_data,
                                position: pos - 8,
                            });
                            pos += consumed;
                        }
                    },
                    MarkerType::RegistersIntAndFloat => {
                        if let Some((int_regs, core_csrs, float_regs, fcsr, consumed)) = parse_int_and_float_registers(&data[pos..]) {
                            let dump = RegistersDump {
                                dump_type: marker_type.clone(),
                                int_registers: int_regs,
                                core_csrs: core_csrs.clone(),
                                float_registers: Some(float_regs),
                                float_csr: Some(fcsr),
                                position: pos - 8,
                            };
                            result.register_dumps.push(dump.clone());
                            let mut all_data = int_regs.to_vec();
                            all_data.extend_from_slice(&[
                                core_csrs.mstatus, core_csrs.misa, core_csrs.medeleg, core_csrs.mideleg,
                                core_csrs.mie, core_csrs.mtvec, core_csrs.mcounteren, core_csrs.mscratch,
                                core_csrs.mepc, core_csrs.mcause, core_csrs.mtval, core_csrs.mip,
                                core_csrs.mcycle, core_csrs.minstret, core_csrs.mvendorid, core_csrs.marchid,
                                core_csrs.mimpid, core_csrs.mhartid
                            ]);
                            all_data.push(fcsr);
                            all_data.extend_from_slice(&float_regs);
                            result.output_items.push(OutputItem::RegisterData {
                                marker_type: marker_type.clone(),
                                registers: all_data,
                                position: pos - 8,
                            });
                            pos += consumed;
                        }
                    },
                    MarkerType::ExceptionCSR => {
                        if let Some((csrs, consumed)) = parse_exception_csrs(&data[pos..]) {
                            let dump = ExceptionDump {
                                csrs: csrs.clone(),
                                position: pos - 8,
                                inst_trace: None,
                            };
                            result.exception_dumps.push(dump);
                            result.output_items.push(OutputItem::ExceptionData {
                                csrs,
                                position: pos - 8,
                            });
                            pos += consumed;
                        }
                    },
                    MarkerType::Unknown(_) => {
                        // Skip unknown markers
                    }
                }
                continue;
            } else if looks_like_marker(potential_marker) {
                // Possibly unknown marker
                debug!("❓ Found potential unknown marker 0x{:016X} at position {}", 
                       potential_marker, pos);
                result.output_items.push(OutputItem::MagicMarker {
                    marker: potential_marker,
                    marker_type: MarkerType::Unknown(potential_marker),
                    position: pos,
                });
                pos += 8;
                continue;
            }
        }

        // If unrecognizable, treat as unknown binary data
        let chunk_size = std::cmp::min(8, data.len() - pos);
        let chunk = data[pos..pos + chunk_size].to_vec();
        result.output_items.push(OutputItem::UnknownBinary {
            data: chunk,
            position: pos,
        });
        pos += chunk_size;
    }

    debug!(
        "✅ HTIF parsing completed: {} items, {} register dumps, {} exception dumps",
        result.output_items.len(),
        result.register_dumps.len(),
        result.exception_dumps.len()
    );

    Ok(result)
}

/// Get marker type
fn get_marker_type(marker: u64) -> Option<MarkerType> {
    match marker {
        MARKER_REGISTERS_INT_ONLY => Some(MarkerType::RegistersIntOnly),
        MARKER_REGISTERS_INT_AND_FLOAT => Some(MarkerType::RegistersIntAndFloat),
        MARKER_EXCEPTION_CSR => Some(MarkerType::ExceptionCSR),
        _ => None,
    }
}

/// Try to parse ASCII text
fn try_parse_ascii_text(data: &[u8]) -> Option<(String, usize)> {
    let mut text_end = 0;
    let mut has_printable = false;
    
    for (i, &byte) in data.iter().enumerate() {
        if byte == 0 {
            // Found null terminator, end text
            text_end = i + 1;
            break;
        } else if byte.is_ascii() && (byte.is_ascii_graphic() || byte.is_ascii_whitespace()) {
            has_printable = true;
            text_end = i + 1;
        } else if byte < 32 && byte != b'\n' && byte != b'\r' && byte != b'\t' {
            // Found control character (except common newlines), end text
            break;
        } else if byte > 127 {
            // Found non-ASCII character, end text
            break;
        } else {
            text_end = i + 1;
        }
    }
    
    if text_end > 0 && has_printable {
        let text_bytes = &data[..text_end];
        // Remove trailing null bytes
        let text_bytes = if text_bytes.last() == Some(&0) {
            &text_bytes[..text_bytes.len() - 1]
        } else {
            text_bytes
        };
        
        if let Ok(text) = String::from_utf8(text_bytes.to_vec()) {
            return Some((text, text_end));
        }
    }
    
    None
}

/// Parse 32 integer registers (256 bytes)
fn parse_int_registers(data: &[u8]) -> Option<([u64; 32], CoreCSRs, usize)> {
    if data.len() < 400 {
        return None;
    }
    
    let mut registers = [0u64; 32];
    for i in 0..32 {
        let offset = i * 8;
        registers[i] = read_u64_le(&data[offset..offset + 8]);
    }
    
    // Parse core CSRs (starting from offset 256)
    let core_csrs = CoreCSRs {
        mstatus: read_u64_le(&data[256..264]),
        misa: read_u64_le(&data[264..272]),
        medeleg: read_u64_le(&data[272..280]),
        mideleg: read_u64_le(&data[280..288]),
        mie: read_u64_le(&data[288..296]),
        mtvec: read_u64_le(&data[296..304]),
        mcounteren: read_u64_le(&data[304..312]),
        mscratch: read_u64_le(&data[312..320]),
        mepc: read_u64_le(&data[320..328]),
        mcause: read_u64_le(&data[328..336]),
        mtval: read_u64_le(&data[336..344]),
        mip: read_u64_le(&data[344..352]),
        mcycle: read_u64_le(&data[352..360]),
        minstret: read_u64_le(&data[360..368]),
        mvendorid: read_u64_le(&data[368..376]),
        marchid: read_u64_le(&data[376..384]),
        mimpid: read_u64_le(&data[384..392]),
        mhartid: read_u64_le(&data[392..400]),
    };
    
    debug!("📋 Parsed 32 integer registers + core CSRs");
    Some((registers, core_csrs, 400))
}

/// Parse 32 integer registers + core CSRs + floating-point registers + floating-point CSR (664 bytes)
fn parse_int_and_float_registers(data: &[u8]) -> Option<([u64; 32], CoreCSRs, [u64; 32], u64, usize)> {
    if data.len() < 664 {
        return None;
    }
    
    let mut int_registers = [0u64; 32];
    for i in 0..32 {
        let offset = i * 8;
        int_registers[i] = read_u64_le(&data[offset..offset + 8]);
    }
    
    // Parse core CSRs (starting from offset 256)
    let core_csrs = CoreCSRs {
        mstatus: read_u64_le(&data[256..264]),
        misa: read_u64_le(&data[264..272]),
        medeleg: read_u64_le(&data[272..280]),
        mideleg: read_u64_le(&data[280..288]),
        mie: read_u64_le(&data[288..296]),
        mtvec: read_u64_le(&data[296..304]),
        mcounteren: read_u64_le(&data[304..312]),
        mscratch: read_u64_le(&data[312..320]),
        mepc: read_u64_le(&data[320..328]),
        mcause: read_u64_le(&data[328..336]),
        mtval: read_u64_le(&data[336..344]),
        mip: read_u64_le(&data[344..352]),
        mcycle: read_u64_le(&data[352..360]),
        minstret: read_u64_le(&data[360..368]),
        mvendorid: read_u64_le(&data[368..376]),
        marchid: read_u64_le(&data[376..384]),
        mimpid: read_u64_le(&data[384..392]),
        mhartid: read_u64_le(&data[392..400]),
    };
    
    // Parse floating-point CSR (offset 400)
    let fcsr = read_u64_le(&data[400..408]);
    
    // Parse floating-point registers (starting from offset 408)
    let mut float_registers = [0u64; 32];
    for i in 0..32 {
        let offset = 408 + i * 8;
        float_registers[i] = read_u64_le(&data[offset..offset + 8]);
    }
    
    debug!("📋 Parsed 32 integer + core CSRs + 32 float registers + fcsr");
    Some((int_registers, core_csrs, float_registers, fcsr, 664))
}

/// Parse exception CSRs (72 bytes)
fn parse_exception_csrs(data: &[u8]) -> Option<(ExceptionCSRs, usize)> {
    if data.len() < 72 {
        return None;
    }
    
    let csrs = ExceptionCSRs {
        mstatus: read_u64_le(&data[0..8]),
        mcause: read_u64_le(&data[8..16]),
        mepc: read_u64_le(&data[16..24]),
        mtval: read_u64_le(&data[24..32]),
        mie: read_u64_le(&data[32..40]),
        mip: read_u64_le(&data[40..48]),
        mtvec: read_u64_le(&data[48..56]),
        mscratch: read_u64_le(&data[56..64]),
        mhartid: read_u64_le(&data[64..72]),
    };
    
    debug!("🚨 Parsed exception CSRs: mcause=0x{:016X}, mepc=0x{:016X}", 
           csrs.mcause, csrs.mepc);
    
    Some((csrs, 72))
}

/// Check if it looks like a magic marker
fn looks_like_marker(value: u64) -> bool {
    // Simple heuristic: check for repeated byte patterns or special values
    let bytes = value.to_le_bytes();
    let unique_bytes: std::collections::HashSet<u8> = bytes.iter().cloned().collect();
    
    // If only 1-3 different byte values, might be a marker
    unique_bytes.len() <= 3 || 
    // Or contains common magic patterns
    value & 0xFFFFFFFF == 0xDEADBEEF ||
    value & 0xFFFFFFFF == 0xCAFEBABE ||
    value & 0xFFFFFFFF == 0xFEEDFACE ||
    value & 0xFFFFFFFF == 0xBADC0DE
}

/// Little-endian read 64-bit integer
fn read_u64_le(bytes: &[u8]) -> u64 {
    if bytes.len() < 8 {
        warn!("read_u64_le called with less than 8 bytes ({} bytes)", bytes.len());
        let mut temp_bytes = [0u8; 8];
        let len_to_copy = std::cmp::min(bytes.len(), 8);
        temp_bytes[..len_to_copy].copy_from_slice(&bytes[..len_to_copy]);
        return u64::from_le_bytes(temp_bytes);
    }
    u64::from_le_bytes(bytes[0..8].try_into().unwrap())
}
