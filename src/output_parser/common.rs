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
/// 程序执行输出的解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonExecutionOutput {
    /// 模拟器类型
    pub emulator_type: EmulatorType,
    /// 原始数据长度
    pub raw_data_length: usize,
    /// 解析到的所有输出项
    pub output_items: Vec<OutputItem>,
    /// 寄存器转储（如果有）
    pub register_dumps: Vec<RegistersDump>,
    /// 异常CSR转储（如果有）
    pub exception_dumps: Vec<ExceptionDump>,
}

impl fmt::Display for CommonExecutionOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# 🔍 RISC-V 通用执行输出解析结果")?;
        writeln!(f)?;
        writeln!(f, "**模拟器类型:** `{}`", self.emulator_type)?;
        writeln!(f)?;
        
        // 基本信息表格
        writeln!(f, "## 📊 基本信息")?;
        writeln!(f)?;
        writeln!(f, "| 项目 | 数值 |")?;
        writeln!(f, "|------|------|")?;
        writeln!(f, "| 原始数据大小 | `{} 字节` |", self.raw_data_length)?;
        writeln!(f, "| 输出项总数 | `{}` |", self.output_items.len())?;
        writeln!(f, "| 寄存器转储数量 | `{}` |", self.register_dumps.len())?;
        writeln!(f, "| 异常转储数量 | `{}` |", self.exception_dumps.len())?;
        writeln!(f)?;

        // 输出项详情
        if !self.output_items.is_empty() {
            writeln!(f, "## 📋 输出项详情")?;
            writeln!(f)?;

            // 统计各种类型的输出项
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

            writeln!(f, "### 📈 类型统计")?;
            writeln!(f)?;
            writeln!(f, "| 类型 | 数量 | 描述 |")?;
            writeln!(f, "|------|------|------|")?;
            writeln!(f, "| 📝 ASCII文本项 | `{}` | 可读文本输出 |", ascii_count)?;
            writeln!(f, "| 🔻 魔数标记项 | `{}` | 数据段标记 |", marker_count)?;
            writeln!(f, "| 📋 寄存器数据项 | `{}` | 寄存器转储数据 |", register_data_count)?;
            writeln!(f, "| 🚨 异常数据项 | `{}` | 异常和中断信息 |", exception_data_count)?;
            writeln!(f, "| ❓ 未知二进制项 | `{}` | 未识别的二进制数据 |", unknown_binary_count)?;
            writeln!(f)?;

            // 显示所有输出项，不省略
            writeln!(f, "### 🔍 项目详情 (完整列表)")?;
            writeln!(f)?;

            for (i, item) in self.output_items.iter().enumerate() {
                match item {
                    OutputItem::AsciiText(text) => {
                        // 不省略文本内容
                        writeln!(f, "**[{}]** 📝 **ASCII文本:** `{}`", i + 1, text)?;
                    }
                    OutputItem::MagicMarker {
                        marker,
                        marker_type,
                        position,
                    } => {
                        writeln!(
                            f,
                            "**[{}]** 🔻 **标记:** `{}` (`0x{:016X}`) @位置`{}`",
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
                            "**[{}]** 📋 **寄存器:** `{}` ({} 个寄存器) @位置`{}`",
                            i + 1,
                            marker_type,
                            registers.len(),
                            position
                        )?;
                    }
                    OutputItem::ExceptionData { position, .. } => {
                        writeln!(f, "**[{}]** 🚨 **异常数据** @位置`{}`", i + 1, position)?;
                    }
                    OutputItem::UnknownBinary { data, position } => {
                        writeln!(
                            f,
                            "**[{}]** ❓ **未知数据:** `{} 字节` @位置`{}`",
                            i + 1,
                            data.len(),
                            position
                        )?;
                    }
                }
            }
            writeln!(f)?;
        }

        // 寄存器转储详情 - 显示所有转储，不省略
        if !self.register_dumps.is_empty() {
            writeln!(f, "## 📋 `{}` 寄存器转储详情", self.emulator_type)?;
            writeln!(f)?;

            for (i, dump) in self.register_dumps.iter().enumerate() {
                writeln!(f, "### 📊 寄存器转储 #{} (位置: `{}`)", i + 1, dump.position)?;
                writeln!(f)?;
                writeln!(f, "**转储类型:** `{}`", dump.dump_type)?;
                writeln!(f)?;

                // 显示所有整数寄存器
                writeln!(f, "#### 🔢 所有整数寄存器 (x0-x31)")?;
                writeln!(f)?;
                writeln!(f, "| 寄存器 | ABI名称 | 值 | 描述 |")?;
                writeln!(f, "|--------|---------|----|----- |")?;
                
                for reg_idx in 0..32 {
                    let reg_name = util::get_register_name(reg_idx);
                    let value = dump.int_registers[reg_idx];

                    let description = match reg_idx {
                        0 => "零寄存器",
                        1 => "返回地址",
                        2 => "栈指针",
                        3 => "全局指针",
                        4 => "线程指针",
                        5..=7 => "临时寄存器",
                        8 => "帧指针/保存寄存器",
                        9 => "保存寄存器",
                        10..=11 => "函数参数/返回值",
                        12..=17 => "函数参数",
                        18..=27 => "保存寄存器",
                        28..=31 => "临时寄存器",
                        _ => unreachable!(),
                    };

                    writeln!(
                        f,
                        "| `x{:02}` | `{:>4}` | `0x{:016X}` | {} |",
                        reg_idx, reg_name, value, description
                    )?;
                }
                writeln!(f)?;

                // 显示所有核心CSR寄存器
                writeln!(f, "#### ⚙️ 所有核心CSR寄存器")?;
                writeln!(f)?;
                writeln!(f, "| CSR寄存器 | 值 | 描述 |")?;
                writeln!(f, "|-----------|----|----- |")?;
                writeln!(f, "| `mstatus` | `0x{:016X}` | 机器状态寄存器 |", dump.core_csrs.mstatus)?;
                writeln!(f, "| `misa` | `0x{:016X}` | ISA和扩展 |", dump.core_csrs.misa)?;
                writeln!(f, "| `medeleg` | `0x{:016X}` | 机器异常委托 |", dump.core_csrs.medeleg)?;
                writeln!(f, "| `mideleg` | `0x{:016X}` | 机器中断委托 |", dump.core_csrs.mideleg)?;
                writeln!(f, "| `mie` | `0x{:016X}` | 机器中断使能 |", dump.core_csrs.mie)?;
                writeln!(f, "| `mtvec` | `0x{:016X}` | 机器陷阱向量基地址 |", dump.core_csrs.mtvec)?;
                writeln!(f, "| `mcounteren` | `0x{:016X}` | 机器计数器使能 |", dump.core_csrs.mcounteren)?;
                writeln!(f, "| `mscratch` | `0x{:016X}` | 机器临时寄存器 |", dump.core_csrs.mscratch)?;
                writeln!(f, "| `mepc` | `0x{:016X}` | 机器异常程序计数器 |", dump.core_csrs.mepc)?;
                writeln!(f, "| `mcause` | `0x{:016X}` | 机器陷阱原因 |", dump.core_csrs.mcause)?;
                writeln!(f, "| `mtval` | `0x{:016X}` | 机器坏地址或指令 |", dump.core_csrs.mtval)?;
                writeln!(f, "| `mip` | `0x{:016X}` | 机器中断挂起 |", dump.core_csrs.mip)?;
                writeln!(f, "| `mcycle` | `0x{:016X}` | 机器周期计数器 |", dump.core_csrs.mcycle)?;
                writeln!(f, "| `minstret` | `0x{:016X}` | 机器指令退役计数器 |", dump.core_csrs.minstret)?;
                writeln!(f, "| `mvendorid` | `0x{:016X}` | 厂商ID |", dump.core_csrs.mvendorid)?;
                writeln!(f, "| `marchid` | `0x{:016X}` | 架构ID |", dump.core_csrs.marchid)?;
                writeln!(f, "| `mimpid` | `0x{:016X}` | 实现ID |", dump.core_csrs.mimpid)?;
                writeln!(f, "| `mhartid` | `0x{:016X}` | 硬件线程ID |", dump.core_csrs.mhartid)?;
                writeln!(f)?;

                // 显示所有浮点寄存器（如果存在）
                if let Some(float_regs) = &dump.float_registers {
                    writeln!(f, "#### 🔣 所有浮点寄存器 (f0-f31)")?;
                    writeln!(f)?;
                    writeln!(f, "| 寄存器 | ABI名称 | 值 | 描述 |")?;
                    writeln!(f, "|--------|---------|----|----- |")?;
                    
                    for reg_idx in 0..32 {
                        let (reg_abi_name, description) = match reg_idx {
                            0..=7 => (format!("ft{}", reg_idx), "临时浮点寄存器"),
                            8..=9 => (format!("fs{}", reg_idx - 8), "保存浮点寄存器"),
                            10..=17 => (format!("fa{}", reg_idx - 10), "浮点参数/返回值"),
                            18..=27 => (format!("fs{}", reg_idx - 18 + 2), "保存浮点寄存器"),
                            28..=31 => (format!("ft{}", reg_idx - 28 + 8), "临时浮点寄存器"),
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
                        writeln!(f, "**浮点控制和状态寄存器:** `fcsr = 0x{:016X}`", fcsr)?;
                        writeln!(f)?;
                    }
                }

                // 统计信息
                let non_zero_int = dump
                    .int_registers
                    .iter()
                    .skip(1)
                    .filter(|&&x| x != 0)
                    .count();
                writeln!(f, "> **统计信息:** 非零整数寄存器: `{}/31`", non_zero_int)?;

                if let Some(float_regs) = &dump.float_registers {
                    let non_zero_float = float_regs.iter().filter(|&&x| x != 0).count();
                    writeln!(f, "> 非零浮点寄存器: `{}/32`", non_zero_float)?;
                }
                writeln!(f)?;

                if i < self.register_dumps.len() - 1 {
                    writeln!(f)?;
                }
            }
        }

        // 异常转储详情 - 显示所有异常，不省略
        if !self.exception_dumps.is_empty() {
            writeln!(f, "## 🚨 `{}` 异常转储详情", self.emulator_type)?;
            writeln!(f)?;

            for (i, dump) in self.exception_dumps.iter().enumerate() {
                let exception_desc = util::get_exception_description(dump.csrs.mcause);
                let is_interrupt = (dump.csrs.mcause >> 63) & 1 == 1;
                let exception_type = if is_interrupt { "中断" } else { "异常" };

                writeln!(f, "### ⚡ 异常转储 #{} (位置: `{}`)", i + 1, dump.position)?;
                writeln!(f)?;
                writeln!(f, "**异常PC:** `0x{:016X}`", dump.csrs.mepc)?;
                if let Some(trace) = &dump.inst_trace {
                    writeln!(f, "**溯源指令:** `{}`", trace.disassembly)?;
                    writeln!(f, "**机器码:** `{}`", trace.machine_code)?;
                    writeln!(f, "**原始指令:** `{}`", trace.original_instruction)?;
                }
                writeln!(f, "**类型:** `{}` ({})", exception_desc, exception_type)?;
                writeln!(f)?;

                writeln!(f, "#### CSR详情")?;
                writeln!(f)?;
                writeln!(f, "| CSR寄存器 | 值 | 描述 |")?;
                writeln!(f, "|-----------|----|----- |")?;
                writeln!(f, "| `mcause` | `0x{:016X}` | {} |", dump.csrs.mcause, exception_desc)?;
                writeln!(f, "| `mtval` | `0x{:016X}` | 机器坏地址或指令 |", dump.csrs.mtval)?;
                writeln!(f, "| `mstatus` | `0x{:016X}` | 机器状态寄存器 |", dump.csrs.mstatus)?;
                writeln!(f, "| `mtvec` | `0x{:016X}` | 机器陷阱向量基地址 |", dump.csrs.mtvec)?;
                writeln!(f, "| `mie` | `0x{:016X}` | 机器中断使能 |", dump.csrs.mie)?;
                writeln!(f, "| `mip` | `0x{:016X}` | 机器中断挂起 |", dump.csrs.mip)?;
                writeln!(f, "| `mscratch` | `0x{:016X}` | 机器临时寄存器 |", dump.csrs.mscratch)?;
                writeln!(f, "| `mhartid` | `0x{:016X}` | 硬件线程ID |", dump.csrs.mhartid)?;
                writeln!(f)?;

                if i < self.exception_dumps.len() - 1 {
                    writeln!(f)?;
                }
            }
        }

        // 数据分析统计（保持不变）
        writeln!(f, "## 📈 数据分析统计")?;
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

        writeln!(f, "| 统计项 | 数值 |")?;
        writeln!(f, "|--------|------|")?;
        if total_ascii_chars > 0 {
            writeln!(f, "| 📝 ASCII文本总字符数 | `{}` |", total_ascii_chars)?;
        }
        if total_binary_bytes > 0 {
            writeln!(f, "| ❓ 未知二进制数据总字节数 | `{}` |", total_binary_bytes)?;
        }

        // 异常类型统计
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
            writeln!(f, "### 🚨 异常类型分布")?;
            writeln!(f)?;
            writeln!(f, "| 异常类型 | 出现次数 |")?;
            writeln!(f, "|----------|----------|")?;
            for (exception_type, count) in sorted_types {
                writeln!(f, "| {} | `{}` |", exception_type, count)?;
            }
            writeln!(f)?;
        }

        // 寄存器转储类型统计
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

            writeln!(f, "### 📋 寄存器转储类型分布")?;
            writeln!(f)?;
            writeln!(f, "| 转储类型 | 数量 |")?;
            writeln!(f, "|----------|------|")?;
            if int_only_count > 0 {
                writeln!(f, "| 仅整数寄存器 | `{}` |", int_only_count)?;
            }
            if int_float_count > 0 {
                writeln!(f, "| 整数+浮点寄存器 | `{}` |", int_float_count)?;
            }
            writeln!(f)?;
        }

        // 数据覆盖率分析
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

        writeln!(f, "| 📊 数据覆盖率 | `{:.1}%` ({}/{} 字节) |", coverage_ratio, parsed_bytes, self.raw_data_length)?;
        writeln!(f)?;

        writeln!(f, "---")?;
        writeln!(f, "*生成时间: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;

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



/// 输出项类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputItem {
    /// ASCII文本输出
    AsciiText(String),
    /// 魔数标记
    MagicMarker {
        marker: u64,
        marker_type: MarkerType,
        position: usize,
    },
    /// 寄存器转储数据
    RegisterData {
        marker_type: MarkerType,
        registers: Vec<u64>,
        position: usize,
    },
    /// 异常CSR转储数据
    ExceptionData {
        csrs: ExceptionCSRs,
        position: usize,
    },
    /// 未知二进制数据
    UnknownBinary { data: Vec<u8>, position: usize },
}
/// 从文件解析执行输出
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

    // 如果有异常，尝试从ELF dump中溯源指令
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

/// 解析二进制数据
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
        // 尝试找到可打印的ASCII文本
        if let Some((text, consumed)) = try_parse_ascii_text(&data[pos..]) {
            if !text.is_empty() {
                debug!("📝 Found ASCII text at position {}: {:?}", pos, text);
                result.output_items.push(OutputItem::AsciiText(text));
            }
            pos += consumed;
            continue;
        }

        // 尝试解析8字节的魔数标记
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
                
                // 根据标记类型解析后续数据
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
                        // 对于未知标记，跳过
                    }
                }
                continue;
            } else if looks_like_marker(potential_marker) {
                // 可能是未知的标记
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

        // 如果无法识别，作为未知二进制数据处理
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

/// 获取标记类型
fn get_marker_type(marker: u64) -> Option<MarkerType> {
    match marker {
        MARKER_REGISTERS_INT_ONLY => Some(MarkerType::RegistersIntOnly),
        MARKER_REGISTERS_INT_AND_FLOAT => Some(MarkerType::RegistersIntAndFloat),
        MARKER_EXCEPTION_CSR => Some(MarkerType::ExceptionCSR),
        _ => None,
    }
}

/// 尝试解析ASCII文本
fn try_parse_ascii_text(data: &[u8]) -> Option<(String, usize)> {
    let mut text_end = 0;
    let mut has_printable = false;
    
    for (i, &byte) in data.iter().enumerate() {
        if byte == 0 {
            // 遇到null终止符，结束文本
            text_end = i + 1;
            break;
        } else if byte.is_ascii() && (byte.is_ascii_graphic() || byte.is_ascii_whitespace()) {
            has_printable = true;
            text_end = i + 1;
        } else if byte < 32 && byte != b'\n' && byte != b'\r' && byte != b'\t' {
            // 遇到控制字符（除了常见的换行符），结束文本
            break;
        } else if byte > 127 {
            // 遇到非ASCII字符，结束文本
            break;
        } else {
            text_end = i + 1;
        }
    }
    
    if text_end > 0 && has_printable {
        let text_bytes = &data[..text_end];
        // 移除尾部的null字节
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

/// 解析32个整数寄存器 (256字节)
fn parse_int_registers(data: &[u8]) -> Option<([u64; 32], CoreCSRs, usize)> {
    if data.len() < 400 {
        return None;
    }
    
    let mut registers = [0u64; 32];
    for i in 0..32 {
        let offset = i * 8;
        registers[i] = read_u64_le(&data[offset..offset + 8]);
    }
    
    // 解析核心CSRs (从偏移256开始)
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

/// 解析32个整数寄存器 + 核心CSRs + 浮点寄存器 + 浮点CSR (664字节)
fn parse_int_and_float_registers(data: &[u8]) -> Option<([u64; 32], CoreCSRs, [u64; 32], u64, usize)> {
    if data.len() < 664 {
        return None;
    }
    
    let mut int_registers = [0u64; 32];
    for i in 0..32 {
        let offset = i * 8;
        int_registers[i] = read_u64_le(&data[offset..offset + 8]);
    }
    
    // 解析核心CSRs (从偏移256开始)
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
    
    // 解析浮点CSR (偏移400)
    let fcsr = read_u64_le(&data[400..408]);
    
    // 解析浮点寄存器 (从偏移408开始)
    let mut float_registers = [0u64; 32];
    for i in 0..32 {
        let offset = 408 + i * 8;
        float_registers[i] = read_u64_le(&data[offset..offset + 8]);
    }
    
    debug!("📋 Parsed 32 integer + core CSRs + 32 float registers + fcsr");
    Some((int_registers, core_csrs, float_registers, fcsr, 664))
}

/// 解析异常CSR (72字节)
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

/// 判断是否看起来像魔数标记
fn looks_like_marker(value: u64) -> bool {
    // 简单启发式：检查是否有重复的字节模式或特殊值
    let bytes = value.to_le_bytes();
    let unique_bytes: std::collections::HashSet<u8> = bytes.iter().cloned().collect();
    
    // 如果只有1-3个不同的字节值，可能是标记
    unique_bytes.len() <= 3 || 
    // 或者包含常见的魔数模式
    value & 0xFFFFFFFF == 0xDEADBEEF ||
    value & 0xFFFFFFFF == 0xCAFEBABE ||
    value & 0xFFFFFFFF == 0xFEEDFACE ||
    value & 0xFFFFFFFF == 0xBADC0DE
}

/// 小端序读取64位整数
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
