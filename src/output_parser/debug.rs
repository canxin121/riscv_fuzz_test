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

/// 调试输出中的单个解析项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugExecutionOutputItem {
    /// 标记
    Marker(MarkerType, usize), // MarkerType, Position
    /// 寄存器转储信息 (不含完整数据，仅元信息)
    RegisterDumpInfo(MarkerType, usize, usize), // MarkerType, RegisterCount, Position
    /// 异常信息 (不含完整数据，仅元信息)
    ExceptionInfo(ExceptionCSRs, usize), // ExceptionCSRs, Position
    /// 文本
    Text(String),
    /// 未知数据块
    Unknown(usize, usize), // Length, Position
}

impl fmt::Display for DebugExecutionOutputItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DebugExecutionOutputItem::Marker(marker_type, pos) => {
                write!(f, "标记 @{}: {:?}", pos, marker_type)
            }
            DebugExecutionOutputItem::RegisterDumpInfo(marker_type, count, pos) => {
                write!(
                    f,
                    "寄存器转储信息 @{}: {:?} ({} 个寄存器)",
                    pos, marker_type, count
                )
            }
            DebugExecutionOutputItem::ExceptionInfo(csrs, pos) => {
                write!(
                    f,
                    "异常信息 @{}: MEPC=0x{:X}, MCAUSE=0x{:X}",
                    pos, csrs.mepc, csrs.mcause
                )
            }
            DebugExecutionOutputItem::Text(text) => {
                // 移除省略，完整显示文本内容
                write!(f, "文本: \"{}\"", text.replace('\n', "\\n"))
            }
            DebugExecutionOutputItem::Unknown(len, pos) => {
                write!(f, "未知数据 @{}: {} 字节", pos, len)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugExecutionOutput {
    /// 模拟器类型
    pub emulator_type: EmulatorType,
    /// 原始数据长度
    pub raw_data_length: usize,
    /// 解析到的调试项
    pub parsed_debug_items: Vec<DebugExecutionOutputItem>,
    /// 有效的寄存器转储
    pub register_dumps: Vec<RegistersDump>,
    /// 总转储数（包括有效和无效）
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
        writeln!(f, "# 🔧 RISC-V 调试执行输出")?;
        writeln!(f)?;
        writeln!(f, "**模拟器类型:** `{}`", self.emulator_type)?;
        writeln!(f)?;

        // 基本信息表格
        writeln!(f, "## 📊 基本信息")?;
        writeln!(f)?;
        writeln!(f, "| 项目 | 数值 |")?;
        writeln!(f, "|------|------|")?;
        writeln!(f, "| 原始数据长度 | `{} 字节` |", self.raw_data_length)?;
        writeln!(
            f,
            "| 解析的调试项数量 | `{}` |",
            self.parsed_debug_items.len()
        )?;
        writeln!(
            f,
            "| 有效寄存器转储数量 | `{}` |",
            self.register_dumps.len()
        )?;
        writeln!(f, "| 遇到的总转储标记数 | `{}` |", self.total_dumps)?;
        writeln!(f)?;

        // 调试项详情
        if !self.parsed_debug_items.is_empty() {
            writeln!(f, "## 📋 `{}` 解析的调试项", self.emulator_type)?;
            writeln!(f)?;

            // 统计不同类型的调试项
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

            writeln!(f, "### 📈 调试项类型统计")?;
            writeln!(f)?;
            writeln!(f, "| 类型 | 数量 | 描述 |")?;
            writeln!(f, "|------|------|------|")?;
            writeln!(f, "| 🔻 标记项 | `{}` | 数据段标记 |", marker_count)?;
            writeln!(
                f,
                "| 📋 寄存器转储信息 | `{}` | 寄存器转储元信息 |",
                register_info_count
            )?;
            writeln!(
                f,
                "| 🚨 异常信息 | `{}` | 异常和中断信息 |",
                exception_info_count
            )?;
            writeln!(f, "| 📝 文本项 | `{}` | 可读文本输出 |", text_count)?;
            writeln!(f, "| ❓ 未知数据 | `{}` | 未识别的数据块 |", unknown_count)?;
            writeln!(f)?;

            writeln!(f, "### 🔍 调试项详情 (完整列表)")?;
            writeln!(f)?;

            // 完整显示所有调试项，不省略
            for (i, item) in self.parsed_debug_items.iter().enumerate() {
                match item {
                    DebugExecutionOutputItem::Marker(marker_type, pos) => {
                        writeln!(
                            f,
                            "**[{}]** 🔻 **标记:** `{:?}` @位置`{}`",
                            i + 1,
                            marker_type,
                            pos
                        )?;
                    }
                    DebugExecutionOutputItem::RegisterDumpInfo(marker_type, count, pos) => {
                        writeln!(
                            f,
                            "**[{}]** 📋 **寄存器转储信息:** `{:?}` ({} 个寄存器) @位置`{}`",
                            i + 1,
                            marker_type,
                            count,
                            pos
                        )?;
                    }
                    DebugExecutionOutputItem::ExceptionInfo(csrs, pos) => {
                        writeln!(
                            f,
                            "**[{}]** 🚨 **异常信息:** MEPC=`0x{:X}`, MCAUSE=`0x{:X}` @位置`{}`",
                            i + 1,
                            csrs.mepc,
                            csrs.mcause,
                            pos
                        )?;
                    }
                    DebugExecutionOutputItem::Text(text) => {
                        // 完整显示文本内容，不省略
                        writeln!(f, "**[{}]** 📝 **文本:** `{}`", i + 1, text)?;
                    }
                    DebugExecutionOutputItem::Unknown(len, pos) => {
                        writeln!(
                            f,
                            "**[{}]** ❓ **未知数据:** `{} 字节` @位置`{}`",
                            i + 1,
                            len,
                            pos
                        )?;
                    }
                }
            }
            writeln!(f)?;
        }

        // 有效寄存器转储详情
        if !self.register_dumps.is_empty() {
            writeln!(f, "## 📝 `{}` 有效寄存器转储", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "**总计:** `{} 个有效转储`", self.register_dumps.len())?;
            writeln!(f)?;

            // 完整显示所有转储，不省略
            for (i, dump) in self.register_dumps.iter().enumerate() {
                writeln!(f, "### 📊 转储 #{} (位置: `{}`)", i + 1, dump.position)?;
                writeln!(f)?;
                writeln!(f, "**转储类型:** `{:?}`", dump.dump_type)?;
                writeln!(f)?;

                // 关键寄存器概览 - 显示所有寄存器
                writeln!(f, "#### 🎯 所有整数寄存器")?;
                writeln!(f)?;
                writeln!(f, "| 寄存器 | ABI名称 | 值 | 描述 |")?;
                writeln!(f, "|--------|---------|----|----- |")?;

                for reg_idx in 0..32 {
                    let reg_name = get_register_name(reg_idx);
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
                        _ => "未知寄存器",
                    };

                    writeln!(
                        f,
                        "| `x{:02}` | `{}` | `0x{:016X}` | {} |",
                        reg_idx, reg_name, value, description
                    )?;
                }
                writeln!(f)?;

                // 核心CSR概览 - 显示所有CSR
                writeln!(f, "#### ⚙️ 所有核心CSR")?;
                writeln!(f)?;
                writeln!(f, "| CSR | 值 | 描述 |")?;
                writeln!(f, "|-----|-----|----- |")?;
                writeln!(
                    f,
                    "| `mstatus` | `0x{:016X}` | 机器状态寄存器 |",
                    dump.core_csrs.mstatus
                )?;
                writeln!(
                    f,
                    "| `misa` | `0x{:016X}` | ISA和扩展 |",
                    dump.core_csrs.misa
                )?;
                writeln!(
                    f,
                    "| `medeleg` | `0x{:016X}` | 机器异常委托 |",
                    dump.core_csrs.medeleg
                )?;
                writeln!(
                    f,
                    "| `mideleg` | `0x{:016X}` | 机器中断委托 |",
                    dump.core_csrs.mideleg
                )?;
                writeln!(
                    f,
                    "| `mie` | `0x{:016X}` | 机器中断使能 |",
                    dump.core_csrs.mie
                )?;
                writeln!(
                    f,
                    "| `mtvec` | `0x{:016X}` | 机器陷阱向量基地址 |",
                    dump.core_csrs.mtvec
                )?;
                writeln!(
                    f,
                    "| `mcounteren` | `0x{:016X}` | 机器计数器使能 |",
                    dump.core_csrs.mcounteren
                )?;
                writeln!(
                    f,
                    "| `mscratch` | `0x{:016X}` | 机器临时寄存器 |",
                    dump.core_csrs.mscratch
                )?;
                writeln!(
                    f,
                    "| `mepc` | `0x{:016X}` | 机器异常程序计数器 |",
                    dump.core_csrs.mepc
                )?;
                writeln!(
                    f,
                    "| `mcause` | `0x{:016X}` | 机器陷阱原因 |",
                    dump.core_csrs.mcause
                )?;
                writeln!(
                    f,
                    "| `mtval` | `0x{:016X}` | 机器坏地址或指令 |",
                    dump.core_csrs.mtval
                )?;
                writeln!(
                    f,
                    "| `mip` | `0x{:016X}` | 机器中断挂起 |",
                    dump.core_csrs.mip
                )?;
                writeln!(
                    f,
                    "| `mcycle` | `0x{:016X}` | 机器周期计数器 |",
                    dump.core_csrs.mcycle
                )?;
                writeln!(
                    f,
                    "| `minstret` | `0x{:016X}` | 机器指令退役计数器 |",
                    dump.core_csrs.minstret
                )?;
                writeln!(
                    f,
                    "| `mvendorid` | `0x{:016X}` | 厂商ID |",
                    dump.core_csrs.mvendorid
                )?;
                writeln!(
                    f,
                    "| `marchid` | `0x{:016X}` | 架构ID |",
                    dump.core_csrs.marchid
                )?;
                writeln!(
                    f,
                    "| `mimpid` | `0x{:016X}` | 实现ID |",
                    dump.core_csrs.mimpid
                )?;
                writeln!(
                    f,
                    "| `mhartid` | `0x{:016X}` | 硬件线程ID |",
                    dump.core_csrs.mhartid
                )?;
                writeln!(f)?;

                // 浮点寄存器详情 - 显示所有浮点寄存器
                if let Some(float_regs) = &dump.float_registers {
                    writeln!(f, "#### 🔣 所有浮点寄存器")?;
                    writeln!(f)?;
                    writeln!(f, "| 寄存器 | 值 |")?;
                    writeln!(f, "|--------|-----|")?;
                    for (i, &val) in float_regs.iter().enumerate() {
                        writeln!(f, "| `f{}` | `0x{:016X}` |", i, val)?;
                    }
                    writeln!(f)?;
                }

                if let Some(fcsr) = dump.float_csr {
                    writeln!(f, "**浮点CSR:** `fcsr = 0x{:016X}`", fcsr)?;
                    writeln!(f)?;
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
        } else {
            writeln!(f, "## 📝 `{}` 有效寄存器转储", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "> ❌ **无有效寄存器转储**")?;
            writeln!(f)?;
        }

        // 数据分析统计
        writeln!(f, "## 📈 数据分析统计")?;
        writeln!(f)?;
        writeln!(f, "| 统计项 | 数值 |")?;
        writeln!(f, "|--------|------|")?;
        writeln!(
            f,
            "| 转储效率 | `{:.1}%` ({}/{} 个转储有效) |",
            if self.total_dumps > 0 {
                (self.register_dumps.len() as f64 / self.total_dumps as f64) * 100.0
            } else {
                0.0
            },
            self.register_dumps.len(),
            self.total_dumps
        )?;

        // 数据类型分布
        let total_items = self.parsed_debug_items.len();
        if total_items > 0 {
            let marker_ratio = self
                .parsed_debug_items
                .iter()
                .filter(|item| matches!(item, DebugExecutionOutputItem::Marker(_, _)))
                .count() as f64
                / total_items as f64
                * 100.0;
            writeln!(f, "| 标记占比 | `{:.1}%` |", marker_ratio)?;
        }
        writeln!(f)?;

        writeln!(f, "---")?;
        writeln!(
            f,
            "*生成时间: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

/// 格式化debug输出结果为可读字符串
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

        // 显示寄存器值
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
