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

/// 转换统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionStats {
    /// 原始异常转储数量
    pub original_exception_count: usize,
    /// 原始寄存器转储数量
    pub original_register_count: usize,
    /// 转换是否成功
    pub conversion_successful: bool,
    /// 转换警告信息
    pub warnings: Vec<String>,
}

/// 标准化的执行输出结构
/// 包含异常转储和单个寄存器转储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardExecutionOutput {
    /// 模拟器类型
    pub emulator_type: EmulatorType,
    /// 异常转储列表
    pub exceptions: Vec<ExceptionDump>,
    /// 寄存器转储（通常只有一个）
    pub register_dump: Option<RegistersDump>,
    /// 转换过程中的统计信息
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
        writeln!(f, "# 🎯 RISC-V 标准执行输出")?;
        writeln!(f)?;
        writeln!(f, "**模拟器类型:** `{}`", self.emulator_type)?;
        writeln!(f)?;

        // 基本信息表格
        writeln!(f, "## 📊 基本信息")?;
        writeln!(f)?;
        writeln!(f, "| 项目 | 数值 |")?;
        writeln!(f, "|------|------|")?;
        writeln!(f, "| 异常数量 | `{}` |", self.exceptions.len())?;
        writeln!(
            f,
            "| 寄存器转储 | `{}` |",
            if self.register_dump.is_some() {
                "存在"
            } else {
                "无"
            }
        )?;
        writeln!(f)?;

        // 转换统计
        writeln!(f, "## 🔄 转换统计")?;
        writeln!(f)?;
        writeln!(f, "| 统计项 | 数值 | 状态 |")?;
        writeln!(f, "|--------|------|------|")?;
        writeln!(
            f,
            "| 原始异常计数 | `{}` | - |",
            self.conversion_stats.original_exception_count
        )?;
        writeln!(
            f,
            "| 原始寄存器转储计数 | `{}` | - |",
            self.conversion_stats.original_register_count
        )?;
        writeln!(
            f,
            "| 转换成功 | `{}` | {} |",
            self.conversion_stats.conversion_successful,
            if self.conversion_stats.conversion_successful {
                "✅"
            } else {
                "❌"
            }
        )?;
        writeln!(
            f,
            "| 警告数量 | `{}` | {} |",
            self.conversion_stats.warnings.len(),
            if self.conversion_stats.warnings.is_empty() {
                "✅"
            } else {
                "⚠️"
            }
        )?;
        writeln!(f)?;

        if !self.conversion_stats.warnings.is_empty() {
            writeln!(f, "### ⚠️ 转换警告 (完整列表)")?;
            writeln!(f)?;
            // 显示所有警告，不省略
            for (i, warning) in self.conversion_stats.warnings.iter().enumerate() {
                writeln!(f, "{}. `{}`", i + 1, warning)?;
            }
            writeln!(f)?;
        }

        // 异常列表
        if !self.exceptions.is_empty() {
            writeln!(f, "## 🚨 `{}` 异常列表", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "**总计:** `{} 个异常`", self.exceptions.len())?;
            writeln!(f)?;

            writeln!(f, "| # | MEPC | MCAUSE | 异常描述 | MTVAL | 位置 |")?;
            writeln!(f, "|---|------|--------|----------|-------|------|")?;

            // 显示所有异常，不省略
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
            writeln!(f, "## 🚨 `{}` 异常列表", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "> ✅ **无异常记录**")?;
            writeln!(f)?;
        }

        // 寄存器转储
        if let Some(dump) = &self.register_dump {
            writeln!(f, "## 📝 `{}` 最终寄存器转储", self.emulator_type)?;
            writeln!(f)?;
            writeln!(
                f,
                "**转储类型:** `{:?}` | **位置:** `{}`",
                dump.dump_type, dump.position
            )?;
            writeln!(f)?;

            // 核心寄存器完整列表
            writeln!(f, "### 🎯 所有整数寄存器")?;
            writeln!(f)?;
            writeln!(f, "| 寄存器 | 值 | 描述 |")?;
            writeln!(f, "|--------|-----|----- |")?;
            for i in 0..32 {
                let reg_name = get_register_name(i);
                let description = match i {
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
                    "| `{}` (x{}) | `0x{:016X}` | {} |",
                    reg_name, i, dump.int_registers[i], description
                )?;
            }
            writeln!(f)?;

            // 核心CSR完整列表
            writeln!(f, "### ⚙️ 所有核心CSR")?;
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

            if let Some(fp_regs) = &dump.float_registers {
                writeln!(f, "### 🔣 所有浮点寄存器")?;
                writeln!(f)?;
                writeln!(f, "| 寄存器 | 值 |")?;
                writeln!(f, "|--------|-----|")?;
                // 显示所有浮点寄存器
                for (i, &val) in fp_regs.iter().enumerate() {
                    writeln!(f, "| `f{}` | `0x{:016X}` |", i, val)?;
                }
                writeln!(f)?;
            }

            if let Some(fcsr) = dump.float_csr {
                writeln!(f, "**浮点CSR:** `fcsr = 0x{:016X}`", fcsr)?;
                writeln!(f)?;
            }
        } else {
            writeln!(f, "## 📝 `{}` 最终寄存器转储", self.emulator_type)?;
            writeln!(f)?;
            writeln!(f, "> ❌ **无寄存器转储**")?;
            writeln!(f)?;
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "*生成时间: {}",
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
