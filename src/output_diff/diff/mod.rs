pub mod common_diff;
pub mod debug_diff;
pub mod standard_diff;

use crate::elf::tracer::InstructionTrace;
use crate::emulators::EmulatorType; // Use the canonical EmulatorType
use crate::output_parser::{
    CoreCSRs, ExceptionCSRs, ExceptionDump, RegistersDump, util::get_exception_description,
    util::get_register_name,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// 引入必要的输出类型和 Diff 类型
use self::common_diff::CommonExecutionOutputDiff;
use self::debug_diff::DebugExecutionOutputDiff;
use self::standard_diff::StandardExecutionOutputDiff;
use crate::output_parser::common::CommonExecutionOutput;
use crate::output_parser::debug::DebugExecutionOutput;
use crate::output_parser::standard::StandardExecutionOutput;

/// 异常差异类别
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum ExceptionDiffCategory {
    /// 固定MIP差异模式
    FixedMipDifference { sim1_value: u64, sim2_value: u64 },
    /// MCAUSE差异模式
    McauseDifference { sim1_cause: u64, sim2_cause: u64 },
    /// 仅在特定模拟器中出现的异常
    OnlyInSimulator {
        simulator: EmulatorType,
        mcause: u64,
    },
    /// MTVAL差异
    MtvalDifference,
    /// 其他CSR差异
    OtherCsrDifference { csr_name: String },
    // OccurrenceCountDifference might be harder to map directly from current ExceptionListDiff
}

/// 异常差异类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExceptionDiffInfo {
    /// 异常仅在一个模拟器中存在
    OnlyInSimulator {
        simulator: EmulatorType, // Already crate::emulators::EmulatorType
        pc: u64,
        mcause: u64,
        description: String,
        instruction_trace: Option<InstructionTrace>, // Added field
                                                     // occurrence_count: usize, // Individual diffs are 1, categorization will sum them
    },
    /// CSR值差异
    CsrDifference {
        pc: u64,
        csr_name: String,
        sim1_value: u64,
        sim2_value: u64,
        sim1_description: Option<String>,
        sim2_description: Option<String>,
        instruction_trace: Option<InstructionTrace>, // Added field
    },
    // OccurrenceCountDifference is not directly produced by compare_exception_dump_lists
    // It would require a different input structure or pre-processing.
}

impl ExceptionDiffInfo {
    /// 获取异常差异的类别
    pub fn get_category(&self) -> ExceptionDiffCategory {
        match self {
            ExceptionDiffInfo::OnlyInSimulator {
                simulator, mcause, ..
            } => ExceptionDiffCategory::OnlyInSimulator {
                simulator: *simulator, // Dereference if it's a copy type, or clone
                mcause: *mcause,
            },
            ExceptionDiffInfo::CsrDifference {
                csr_name,
                sim1_value,
                sim2_value,
                ..
            } => match csr_name.as_str() {
                "mip" => ExceptionDiffCategory::FixedMipDifference {
                    sim1_value: *sim1_value,
                    sim2_value: *sim2_value,
                },
                "mcause" => ExceptionDiffCategory::McauseDifference {
                    sim1_cause: *sim1_value,
                    sim2_cause: *sim2_value,
                },
                "mtval" => ExceptionDiffCategory::MtvalDifference,
                _ => ExceptionDiffCategory::OtherCsrDifference {
                    csr_name: csr_name.clone(),
                },
            },
        }
    }

    /// 获取PC地址
    pub fn get_pc(&self) -> u64 {
        match self {
            ExceptionDiffInfo::OnlyInSimulator { pc, .. } => *pc,
            ExceptionDiffInfo::CsrDifference { pc, .. } => *pc,
        }
    }
}

/// 归类后的异常差异组
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategorizedExceptionDiffs {
    pub category: ExceptionDiffCategory,
    pub diffs_summary: Vec<String>, // Store brief descriptions of individual diffs
    pub count: usize,
    pub pc_list: Vec<u64>,
    pub pc_instruction_traces: Vec<Option<InstructionTrace>>, // Added field for instruction traces
}

impl fmt::Display for CategorizedExceptionDiffs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "### {}", format_category_title(&self.category))?;
        writeln!(f)?;
        writeln!(f, "| 属性 | 数值 |")?;
        writeln!(f, "|------|------|")?;
        writeln!(f, "| 发生次数 | {} |", self.count)?;
        writeln!(f, "| 受影响的PC | {} |", self.pc_list.len())?;
        writeln!(f)?;

        if !self.pc_list.is_empty() {
            writeln!(f, "#### PC地址与指令映射")?;
            writeln!(f)?;
            writeln!(f, "| # | PC地址 | 反汇编指令 | 原始汇编指令|")?;
            writeln!(f, "|---|--------|------------|------------|")?;
            for (i, pc) in self.pc_list.iter().enumerate() {
                let trace_info = if i < self.pc_instruction_traces.len() {
                    self.pc_instruction_traces[i].as_ref()
                } else {
                    None
                };

                match trace_info {
                    Some(trace) => {
                        writeln!(
                            f,
                            "| {} | 0x{:016X} | {} | {} |",
                            i + 1,
                            pc,
                            trace.disassembly,
                            trace.original_instruction
                        )?;
                    }
                    None => {
                        writeln!(f, "| {} | 0x{:016X} | - | - |", i + 1, pc)?;
                    }
                }
            }
            writeln!(f)?;
        }

        if !self.diffs_summary.is_empty() {
            writeln!(f, "#### 差异示例")?;
            writeln!(f)?;
            for (i, summary) in self.diffs_summary.iter().enumerate() {
                writeln!(f, "{}. {}", i + 1, summary)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "#### 描述")?;
        writeln!(f)?;
        writeln!(f, "{}", format_category_description(&self.category))?;

        Ok(())
    }
}

pub fn format_category_title(category: &ExceptionDiffCategory) -> String {
    match category {
        ExceptionDiffCategory::FixedMipDifference {
            sim1_value,
            sim2_value,
        } => {
            format!(
                "固定 MIP 差异 (值1=0x{:X}, 值2=0x{:X})",
                sim1_value, sim2_value
            )
        }
        ExceptionDiffCategory::McauseDifference {
            sim1_cause,
            sim2_cause,
        } => {
            let sim1_desc = get_exception_description(*sim1_cause);
            let sim2_desc = get_exception_description(*sim2_cause);
            format!("MCAUSE 差异 (原因1: {} vs 原因2: {})", sim1_desc, sim2_desc)
        }
        ExceptionDiffCategory::OnlyInSimulator { simulator, mcause } => {
            let desc = get_exception_description(*mcause);
            format!(
                "仅在 {} 中出现 (mcause: 0x{:X} - {})",
                simulator, mcause, desc
            )
        }
        ExceptionDiffCategory::MtvalDifference => "MTVAL 值差异".to_string(),
        ExceptionDiffCategory::OtherCsrDifference { csr_name } => {
            format!("其他 CSR ({}) 差异", csr_name)
        }
    }
}

pub fn format_category_name(category: &ExceptionDiffCategory) -> String {
    match category {
        ExceptionDiffCategory::FixedMipDifference { .. } => "固定MIP差异".to_string(),
        ExceptionDiffCategory::McauseDifference { .. } => "MCAUSE差异".to_string(),
        ExceptionDiffCategory::OnlyInSimulator { simulator, .. } => {
            format!("仅在{}中出现的异常", simulator)
        }
        ExceptionDiffCategory::MtvalDifference => "MTVAL差异".to_string(),
        ExceptionDiffCategory::OtherCsrDifference { csr_name } => {
            format!("{}差异", csr_name)
        }
    }
}

fn format_category_description(category: &ExceptionDiffCategory) -> String {
    match category {
        ExceptionDiffCategory::FixedMipDifference { .. } => {
            "描述: MIP 寄存器值在模拟器之间存在固定差异。\n".to_string()
        }
        ExceptionDiffCategory::McauseDifference { .. } => {
            "描述: 相同的操作导致了不同的异常原因。\n".to_string()
        }
        ExceptionDiffCategory::OnlyInSimulator { simulator, .. } => {
            format!(
                "描述: 异常仅在 {} 中触发，另一个模拟器在此点继续执行或没有异常。\n",
                simulator
            )
        }
        ExceptionDiffCategory::MtvalDifference => {
            "描述: MTVAL 寄存器值不同，可能是由于地址计算或陷入条件不同。\n".to_string()
        }
        ExceptionDiffCategory::OtherCsrDifference { csr_name } => {
            format!("描述: 其他 CSR ({}) 寄存器存在差异。\n", csr_name)
        }
    }
}

/// Analyzes and categorizes a list of raw exception differences.
pub fn analyze_and_categorize_exception_diffs(
    raw_diffs: Vec<ExceptionDiffInfo>,
    // sim1_type: EmulatorType, // No longer needed if ExceptionDiffInfo stores the correct type
    // sim2_type: EmulatorType, // No longer needed
) -> Vec<CategorizedExceptionDiffs> {
    let mut category_map: HashMap<ExceptionDiffCategory, Vec<ExceptionDiffInfo>> = HashMap::new();

    for diff in raw_diffs {
        let category = diff.get_category();
        category_map.entry(category).or_default().push(diff);
    }

    let mut categorized_diffs: Vec<CategorizedExceptionDiffs> = category_map
        .into_iter()
        .map(|(category, diff_list)| {
            let count = diff_list.len();
            let mut pc_list: Vec<u64> = diff_list.iter().map(|d| d.get_pc()).collect();
            pc_list.sort_unstable();
            pc_list.dedup(); // Keep only unique PCs for the summary

            // Collect instruction traces for each unique PC
            let mut pc_instruction_traces: Vec<Option<InstructionTrace>> = Vec::new();
            for pc in &pc_list {
                // Find the first diff with this PC and get its instruction trace
                let trace = diff_list
                    .iter()
                    .find(|d| d.get_pc() == *pc)
                    .and_then(|d| match d {
                        ExceptionDiffInfo::OnlyInSimulator {
                            instruction_trace, ..
                        } => instruction_trace.clone(),
                        ExceptionDiffInfo::CsrDifference {
                            instruction_trace, ..
                        } => instruction_trace.clone(),
                    });
                pc_instruction_traces.push(trace);
            }

            // Create brief summaries for a few example diffs (optional)
            let diffs_summary = diff_list
                .iter()
                .map(|d| match d {
                    ExceptionDiffInfo::OnlyInSimulator {
                        pc,
                        mcause,
                        instruction_trace,
                        ..
                    } => {
                        let trace_info = instruction_trace.as_ref().map_or_else(
                            || "".to_string(),
                            |trace| format!(" ({})", trace.disassembly),
                        );
                        format!("PC: 0x{:X}{}, Mcause: 0x{:X}", pc, trace_info, mcause)
                    }
                    ExceptionDiffInfo::CsrDifference {
                        pc,
                        csr_name,
                        sim1_value,
                        sim2_value,
                        instruction_trace,
                        ..
                    } => {
                        let trace_info = instruction_trace.as_ref().map_or_else(
                            || "".to_string(),
                            |trace| format!(" ({})", trace.disassembly),
                        );
                        format!(
                            "PC: 0x{:X}{}, CSR: {}, Sim1: 0x{:X}, Sim2: 0x{:X}",
                            pc, trace_info, csr_name, sim1_value, sim2_value
                        )
                    }
                })
                .collect();

            CategorizedExceptionDiffs {
                category,
                diffs_summary,
                count,
                pc_list,
                pc_instruction_traces,
            }
        })
        .collect();

    categorized_diffs.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| format_category_name(&a.category).cmp(&format_category_name(&b.category)))
    });
    categorized_diffs
}

/// Represents the differences between two `RegistersDump` instances.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistersDumpDiff {
    pub emulator_type1: EmulatorType,
    pub emulator_type2: EmulatorType,
    pub int_registers_diff: Vec<(usize, String, u64, u64)>, // index, name, val1, val2
    pub core_csrs_diff: Vec<(String, u64, u64)>,            // csr_name, val1, val2
    pub float_registers_status_changed: Option<(String, String)>, // e.g. (Some, None)
    pub float_registers_diff: Vec<(usize, u64, u64)>,       // index, val1, val2
    pub float_csr_status_changed: Option<(String, String)>, // e.g. (Some, None)
    pub float_csr_diff: Option<(u64, u64)>,
}

impl RegistersDumpDiff {
    /// Checks if there are any differences.
    pub fn is_empty(&self) -> bool {
        self.int_registers_diff.is_empty()
            && self.core_csrs_diff.is_empty()
            && self.float_registers_status_changed.is_none()
            && self.float_registers_diff.is_empty()
            && self.float_csr_status_changed.is_none()
            && self.float_csr_diff.is_none()
    }

    /// 检查是否存在整数或浮点寄存器差异
    pub fn has_register_differences(&self) -> bool {
        !self.int_registers_diff.is_empty() || !self.float_registers_diff.is_empty()
    }

    /// 提取存在差异的寄存器名称
    pub fn extract_differing_registers(&self) -> Vec<String> {
        let mut differing_regs = Vec::new();

        // 添加整数寄存器差异
        for (idx, _name, _val1, _val2) in &self.int_registers_diff {
            differing_regs.push(format!("x{}", idx));
        }

        // 添加浮点寄存器差异
        for (idx, _val1, _val2) in &self.float_registers_diff {
            differing_regs.push(format!("f{}", idx));
        }

        differing_regs
    }
}

impl fmt::Display for RegistersDumpDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# 寄存器转储差异")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "未发现差异")?;
            writeln!(f)?;
            return Ok(());
        }

        // 差异汇总
        let mut diff_sections = Vec::new();
        if !self.int_registers_diff.is_empty() {
            diff_sections.push("整数寄存器");
        }
        if !self.core_csrs_diff.is_empty() {
            diff_sections.push("核心CSR");
        }
        if self.float_registers_status_changed.is_some() {
            diff_sections.push("浮点寄存器状态");
        }
        if !self.float_registers_diff.is_empty() {
            diff_sections.push("浮点寄存器");
        }
        if self.float_csr_status_changed.is_some() || self.float_csr_diff.is_some() {
            diff_sections.push("浮点CSR");
        }

        writeln!(f, "发现差异: {}", diff_sections.join(", "))?;
        writeln!(f)?;

        if !self.int_registers_diff.is_empty() {
            writeln!(f, "## 整数寄存器差异")?;
            writeln!(f)?;
            writeln!(f, "差异数量: {} / 32", self.int_registers_diff.len())?;
            writeln!(f)?;
            writeln!(
                f,
                "| 寄存器 | ABI名称 | {} | {} |",
                self.emulator_type1, self.emulator_type2
            )?;
            writeln!(f, "|--------|---------|------|------|")?;
            for (idx, name, val1, val2) in &self.int_registers_diff {
                writeln!(
                    f,
                    "| x{:02} | {} | 0x{:016X} | 0x{:016X} |",
                    idx, name, val1, val2,
                )?;
            }
            writeln!(f)?;
        }

        if !self.core_csrs_diff.is_empty() {
            writeln!(f, "## 核心CSR差异")?;
            writeln!(f)?;
            writeln!(f, "差异数量: {}", self.core_csrs_diff.len())?;
            writeln!(f)?;
            writeln!(
                f,
                "| CSR | {} | {} |",
                self.emulator_type1, self.emulator_type2
            )?;
            writeln!(f, "|-----|------|------|")?;
            for (name, val1, val2) in &self.core_csrs_diff {
                writeln!(f, "| {} | 0x{:016X} | 0x{:016X} |", name, val1, val2)?;
            }
            writeln!(f)?;
        }

        if let Some((status1, status2)) = &self.float_registers_status_changed {
            writeln!(f, "## 浮点寄存器状态差异")?;
            writeln!(f)?;
            writeln!(
                f,
                "| 项目 | {} | {} |",
                self.emulator_type1, self.emulator_type2
            )?;
            writeln!(f, "|------|--------|--------|")?;
            writeln!(f, "| 浮点寄存器 | {} | {} |", status1, status2)?;
            writeln!(f)?;
        }

        if !self.float_registers_diff.is_empty() {
            writeln!(f, "## 浮点寄存器差异")?;
            writeln!(f)?;
            writeln!(
                f,
                "差异数量: {} / 32 个浮点寄存器",
                self.float_registers_diff.len()
            )?;
            writeln!(f)?;
            writeln!(
                f,
                "| 寄存器 | {} | {} |",
                self.emulator_type1, self.emulator_type2
            )?;
            writeln!(f, "|--------|------|------|")?;
            for (idx, val1, val2) in &self.float_registers_diff {
                writeln!(f, "| f{:02} | 0x{:016X} | 0x{:016X} |", idx, val1, val2,)?;
            }
            writeln!(f)?;
        }

        if let Some((status1, status2)) = &self.float_csr_status_changed {
            writeln!(f, "## 浮点CSR状态差异")?;
            writeln!(f)?;
            writeln!(
                f,
                "| 项目 | {} | {} |",
                self.emulator_type1, self.emulator_type2
            )?;
            writeln!(f, "|------|--------|--------|")?;
            writeln!(f, "| 浮点CSR | {} | {} |", status1, status2)?;
            writeln!(f)?;
        }

        if let Some((val1, val2)) = self.float_csr_diff {
            writeln!(f, "## 浮点CSR差异")?;
            writeln!(f)?;
            writeln!(
                f,
                "| CSR | {} | {} |",
                self.emulator_type1, self.emulator_type2
            )?;
            writeln!(f, "|-----|------|------|")?;
            writeln!(f, "| fcsr | 0x{:016X} | 0x{:016X} |", val1, val2,)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

/// Compares two `RegistersDump` instances.
pub fn compare_registers_dumps(
    dump1: &RegistersDump,
    dump2: &RegistersDump,
    sim1_type: EmulatorType,
    sim2_type: EmulatorType,
) -> RegistersDumpDiff {
    let mut diff = RegistersDumpDiff {
        emulator_type1: sim1_type,
        emulator_type2: sim2_type,
        int_registers_diff: Vec::new(),
        core_csrs_diff: Vec::new(),
        float_registers_status_changed: None,
        float_registers_diff: Vec::new(),
        float_csr_status_changed: None,
        float_csr_diff: None,
    };

    for i in 0..32 {
        if dump1.int_registers[i] != dump2.int_registers[i] {
            diff.int_registers_diff.push((
                i,
                get_register_name(i).to_string(),
                dump1.int_registers[i],
                dump2.int_registers[i],
            ));
        }
    }

    compare_core_csrs(&dump1.core_csrs, &dump2.core_csrs, &mut diff.core_csrs_diff);

    match (&dump1.float_registers, &dump2.float_registers) {
        (Some(fr1), Some(fr2)) => {
            for i in 0..32 {
                if fr1[i] != fr2[i] {
                    diff.float_registers_diff.push((i, fr1[i], fr2[i]));
                }
            }
        }
        (Some(_), None) => {
            diff.float_registers_status_changed =
                Some(("Present".to_string(), "Absent".to_string()));
        }
        (None, Some(_)) => {
            diff.float_registers_status_changed =
                Some(("Absent".to_string(), "Present".to_string()));
        }
        (None, None) => {}
    }

    match (dump1.float_csr, dump2.float_csr) {
        (Some(fcsr1), Some(fcsr2)) => {
            if fcsr1 != fcsr2 {
                diff.float_csr_diff = Some((fcsr1, fcsr2));
            }
        }
        (Some(_), None) => {
            diff.float_csr_status_changed = Some(("Present".to_string(), "Absent".to_string()));
        }
        (None, Some(_)) => {
            diff.float_csr_status_changed = Some(("Absent".to_string(), "Present".to_string()));
        }
        (None, None) => {}
    }

    diff
}

fn compare_core_csrs(csrs1: &CoreCSRs, csrs2: &CoreCSRs, diff_list: &mut Vec<(String, u64, u64)>) {
    if csrs1.mstatus != csrs2.mstatus {
        diff_list.push(("mstatus".to_string(), csrs1.mstatus, csrs2.mstatus));
    }
    if csrs1.misa != csrs2.misa {
        diff_list.push(("misa".to_string(), csrs1.misa, csrs2.misa));
    }
    if csrs1.medeleg != csrs2.medeleg {
        diff_list.push(("medeleg".to_string(), csrs1.medeleg, csrs2.medeleg));
    }
    if csrs1.mideleg != csrs2.mideleg {
        diff_list.push(("mideleg".to_string(), csrs1.mideleg, csrs2.mideleg));
    }
    if csrs1.mie != csrs2.mie {
        diff_list.push(("mie".to_string(), csrs1.mie, csrs2.mie));
    }
    if csrs1.mtvec != csrs2.mtvec {
        diff_list.push(("mtvec".to_string(), csrs1.mtvec, csrs2.mtvec));
    }
    if csrs1.mcounteren != csrs2.mcounteren {
        diff_list.push(("mcounteren".to_string(), csrs1.mcounteren, csrs2.mcounteren));
    }
    if csrs1.mscratch != csrs2.mscratch {
        diff_list.push(("mscratch".to_string(), csrs1.mscratch, csrs2.mscratch));
    }
    if csrs1.mepc != csrs2.mepc {
        diff_list.push(("mepc".to_string(), csrs1.mepc, csrs2.mepc));
    }
    if csrs1.mcause != csrs2.mcause {
        diff_list.push(("mcause".to_string(), csrs1.mcause, csrs2.mcause));
    }
    if csrs1.mtval != csrs2.mtval {
        diff_list.push(("mtval".to_string(), csrs1.mtval, csrs2.mtval));
    }
    if csrs1.mip != csrs2.mip {
        diff_list.push(("mip".to_string(), csrs1.mip, csrs2.mip));
    }
    if csrs1.mcycle != csrs2.mcycle {
        diff_list.push(("mcycle".to_string(), csrs1.mcycle, csrs2.mcycle));
    }
    if csrs1.minstret != csrs2.minstret {
        diff_list.push(("minstret".to_string(), csrs1.minstret, csrs2.minstret));
    }
    if csrs1.mvendorid != csrs2.mvendorid {
        diff_list.push(("mvendorid".to_string(), csrs1.mvendorid, csrs2.mvendorid));
    }
    if csrs1.marchid != csrs2.marchid {
        diff_list.push(("marchid".to_string(), csrs1.marchid, csrs2.marchid));
    }
    if csrs1.mimpid != csrs2.mimpid {
        diff_list.push(("mimpid".to_string(), csrs1.mimpid, csrs2.mimpid));
    }
    if csrs1.mhartid != csrs2.mhartid {
        diff_list.push(("mhartid".to_string(), csrs1.mhartid, csrs2.mhartid));
    }
}

/// Represents differences between two paired `ExceptionDump` instances.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PairedExceptionDiff {
    pub exception1: ExceptionDump,                 // Cloned from list1
    pub exception2: ExceptionDump,                 // Cloned from list2 (the matched one)
    pub csrs_differences: Vec<(String, u64, u64)>, // field_name, val_from_ex1, val_from_ex2
}

impl PairedExceptionDiff {
    pub fn format_with_simulator_names(&self, sim1_name: &str, sim2_name: &str) -> String {
        let mut result = String::new();

        result.push_str(&format!(
            "  配对异常差异 (基于 MEPC 0x{:016X} 匹配):\n",
            self.exception1.csrs.mepc
        ));

        let desc1 = get_exception_description(self.exception1.csrs.mcause);
        let desc2 = get_exception_description(self.exception2.csrs.mcause);

        result.push_str(&format!(
            "    {}异常: 位置={}, MCAUSE=0x{:016X} ({})\n",
            sim1_name, self.exception1.position, self.exception1.csrs.mcause, desc1
        ));

        result.push_str(&format!(
            "    {}异常: 位置={}, MCAUSE=0x{:016X} ({})\n",
            sim2_name, self.exception2.position, self.exception2.csrs.mcause, desc2
        ));

        if !self.csrs_differences.is_empty() {
            result.push_str("    CSR 字段差异:\n");
            for (name, val1, val2) in &self.csrs_differences {
                let val1_desc = if name == "mcause" {
                    format!(" ({})", get_exception_description(*val1))
                } else {
                    "".to_string()
                };
                let val2_desc = if name == "mcause" {
                    format!(" ({})", get_exception_description(*val2))
                } else {
                    "".to_string()
                };
                result.push_str(&format!(
                    "      {}: {}=0x{:016X}{} vs {}=0x{:016X}{}\n",
                    name, sim1_name, val1, val1_desc, sim2_name, val2, val2_desc
                ));
            }
        } else {
            result.push_str("    无字段差异\n");
        }

        result
    }
}

/// Represents the differences between two lists of `ExceptionDump`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionListDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub list1_only_exceptions: Vec<ExceptionDump>,
    pub list2_only_exceptions: Vec<ExceptionDump>,
    pub paired_exceptions_diffs: Vec<PairedExceptionDiff>,
    pub categorized_summary: Vec<CategorizedExceptionDiffs>,
}

impl ExceptionListDiff {
    pub fn is_empty(&self) -> bool {
        // An ExceptionListDiff is empty if:
        // 1. No exceptions exist only in one simulator
        // 2. All paired exceptions have no CSR differences and no position differences
        // 3. No categorized differences exist
        self.list1_only_exceptions.is_empty()
            && self.list2_only_exceptions.is_empty()
            && self
                .paired_exceptions_diffs
                .iter()
                .all(|p| p.csrs_differences.is_empty())
            && self.categorized_summary.is_empty()
    }
}

impl fmt::Display for ExceptionListDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sim1_name = self.sim1_emulator_type.to_string();
        let sim2_name = self.sim2_emulator_type.to_string();

        writeln!(f, "# 异常列表差异报告")?;
        writeln!(f)?;
        writeln!(f, "比较对象: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        let mut significant_diff_found = false;

        // 差异汇总
        let only_sim1_count = self.list1_only_exceptions.len();
        let only_sim2_count = self.list2_only_exceptions.len();
        let paired_diffs_count = self
            .paired_exceptions_diffs
            .iter()
            .filter(|p| !p.csrs_differences.is_empty())
            .count();
        let total_paired = self.paired_exceptions_diffs.len();

        writeln!(f, "## 差异汇总")?;
        writeln!(f)?;
        writeln!(f, "| 类别 | 数量 |")?;
        writeln!(f, "|------|------|")?;
        writeln!(f, "| 仅在 {} 中的异常 | {} |", sim1_name, only_sim1_count)?;
        writeln!(f, "| 仅在 {} 中的异常 | {} |", sim2_name, only_sim2_count)?;
        writeln!(f, "| 匹配异常对 (总数) | {} |", total_paired)?;
        writeln!(f, "| 匹配异常对 (有差异) | {} |", paired_diffs_count)?;
        writeln!(f, "| 分类差异数 | {} |", self.categorized_summary.len())?;
        writeln!(f)?;

        if !self.list1_only_exceptions.is_empty() {
            significant_diff_found = true;
            writeln!(f, "## 仅在 {} 中存在的异常", sim1_name)?;
            writeln!(f)?;
            writeln!(f, "总计: {}", self.list1_only_exceptions.len())?;
            writeln!(f)?;

            writeln!(
                f,
                "| # | MEPC | 反汇编指令 | 原始汇编指令 | MCAUSE | 异常描述 | MTVAL | 位置 |"
            )?;
            writeln!(
                f,
                "|---|------|------------|--------------|--------|----------|-------|------|"
            )?;

            for (i, ex) in self.list1_only_exceptions.iter().enumerate() {
                let desc = get_exception_description(ex.csrs.mcause);
                let (disassembly, original_instruction) = if let Some(trace) = &ex.inst_trace {
                    (
                        trace.disassembly.as_str(),
                        trace.original_instruction.as_str(),
                    )
                } else {
                    ("-", "-")
                };
                writeln!(
                    f,
                    "| {} | 0x{:016X} | {} | {} | 0x{:016X} | {} | 0x{:016X} | {} |",
                    i + 1,
                    ex.csrs.mepc,
                    disassembly,
                    original_instruction,
                    ex.csrs.mcause,
                    desc,
                    ex.csrs.mtval,
                    ex.position
                )?;
            }
            writeln!(f)?;
        }

        if !self.list2_only_exceptions.is_empty() {
            significant_diff_found = true;
            writeln!(f, "## 仅在 {} 中存在的异常", sim2_name)?;
            writeln!(f)?;
            writeln!(f, "总计: {}", self.list2_only_exceptions.len())?;
            writeln!(f)?;

            writeln!(
                f,
                "| # | MEPC | 反汇编指令 | 原始汇编指令 | MCAUSE | 异常描述 | MTVAL | 位置 |"
            )?;
            writeln!(
                f,
                "|---|------|------------|--------------|--------|----------|-------|------|"
            )?;

            for (i, ex) in self.list2_only_exceptions.iter().enumerate() {
                let desc = get_exception_description(ex.csrs.mcause);
                let (disassembly, original_instruction) = if let Some(trace) = &ex.inst_trace {
                    (
                        trace.disassembly.as_str(),
                        trace.original_instruction.as_str(),
                    )
                } else {
                    ("-", "-")
                };
                writeln!(
                    f,
                    "| {} | 0x{:016X} | {} | {} | 0x{:016X} | {} | 0x{:016X} | {} |",
                    i + 1,
                    ex.csrs.mepc,
                    disassembly,
                    original_instruction,
                    ex.csrs.mcause,
                    desc,
                    ex.csrs.mtval,
                    ex.position
                )?;
            }
            writeln!(f)?;
        }

        // 过滤真正有差异的配对异常
        let paired_diffs_with_actual_differences: Vec<&PairedExceptionDiff> = self
            .paired_exceptions_diffs
            .iter()
            .filter(|p| !p.csrs_differences.is_empty())
            .collect();

        if !paired_diffs_with_actual_differences.is_empty() {
            significant_diff_found = true;
            writeln!(f, "## 匹配异常的差异详情")?;
            writeln!(f)?;
            writeln!(
                f,
                "有差异的配对: {} / {} 对",
                paired_diffs_with_actual_differences.len(),
                self.paired_exceptions_diffs.len()
            )?;
            writeln!(f)?;

            for (i, pair_diff) in paired_diffs_with_actual_differences.iter().enumerate() {
                writeln!(
                    f,
                    "### 配对 {} - MEPC: 0x{:016X}",
                    i + 1,
                    pair_diff.exception1.csrs.mepc
                )?;
                writeln!(f)?;

                if let Some(trace) = &pair_diff.exception1.inst_trace {
                    writeln!(f, "#### 触发指令")?;
                    writeln!(f)?;
                    writeln!(f, "| PC地址 | 反汇编指令 | 原始汇编指令 |")?;
                    writeln!(f, "|--------|------------|--------------|")?;
                    writeln!(
                        f,
                        "| 0x{:016X} | {} | {} |",
                        pair_diff.exception1.csrs.mepc,
                        trace.disassembly,
                        trace.original_instruction
                    )?;
                    writeln!(f)?;
                }

                // 异常基本信息对比表
                let desc1 = get_exception_description(pair_diff.exception1.csrs.mcause);
                let desc2 = get_exception_description(pair_diff.exception2.csrs.mcause);

                writeln!(f, "| 项目 | {} | {} |", sim1_name, sim2_name)?;
                writeln!(f, "|------|------------|------------|")?;
                writeln!(
                    f,
                    "| 位置 | {} | {} |",
                    pair_diff.exception1.position, pair_diff.exception2.position
                )?;
                writeln!(
                    f,
                    "| MCAUSE | 0x{:016X} | 0x{:016X} |",
                    pair_diff.exception1.csrs.mcause, pair_diff.exception2.csrs.mcause
                )?;
                writeln!(f, "| 异常描述 | {} | {} |", desc1, desc2)?;
                writeln!(f)?;

                if !pair_diff.csrs_differences.is_empty() {
                    writeln!(f, "#### CSR字段差异")?;
                    writeln!(f)?;
                    writeln!(f, "| CSR字段 | {} | {} | 差异说明 |", sim1_name, sim2_name)?;
                    writeln!(f, "|---------|------------|------------|----------|")?;

                    for (name, val1, val2) in &pair_diff.csrs_differences {
                        let diff_desc = if name == "mcause" {
                            format!(
                                "{} vs {}",
                                get_exception_description(*val1),
                                get_exception_description(*val2)
                            )
                        } else {
                            "数值不同".to_string()
                        };
                        writeln!(
                            f,
                            "| {} | 0x{:016X} | 0x{:016X} | {} |",
                            name, val1, val2, diff_desc
                        )?;
                    }
                    writeln!(f)?;
                } else {
                    writeln!(f, "CSR字段无差异")?;
                    writeln!(f)?;
                }
            }
        } else if !self.paired_exceptions_diffs.is_empty() {
            writeln!(f, "## 匹配异常状态")?;
            writeln!(f)?;
            writeln!(
                f,
                "{} 对 匹配异常，无差异",
                self.paired_exceptions_diffs.len()
            )?;
            writeln!(f)?;
        }

        if !self.categorized_summary.is_empty() {
            significant_diff_found = true;
            writeln!(f, "## 分类异常差异摘要")?;
            writeln!(f)?;
            let total_categorized: usize = self.categorized_summary.iter().map(|s| s.count).sum();
            writeln!(f, "总差异数: {}", total_categorized)?;
            writeln!(f)?;

            for (i, cat_sum) in self.categorized_summary.iter().enumerate() {
                writeln!(f, "### 类别 {}", i + 1)?;
                writeln!(f)?;
                writeln!(f, "类别: {}", format_category_title(&cat_sum.category))?;
                writeln!(f, "发生次数: {}", cat_sum.count)?;
                writeln!(f, "受影响的PC: {} 个", cat_sum.pc_list.len())?;
                writeln!(f)?;

                if !cat_sum.pc_list.is_empty() {
                    writeln!(f, "#### PC地址与指令列表")?;
                    writeln!(f)?;
                    writeln!(f, "| # | PC地址 | 反汇编指令 | 原始汇编指令 |")?;
                    writeln!(f, "|---|--------|------------|--------------|")?;
                    for (j, pc) in cat_sum.pc_list.iter().enumerate() {
                        let trace_info = if j < cat_sum.pc_instruction_traces.len() {
                            cat_sum.pc_instruction_traces[j].as_ref()
                        } else {
                            None
                        };

                        match trace_info {
                            Some(trace) => {
                                writeln!(
                                    f,
                                    "| {} | 0x{:016X} | {} | {} |",
                                    j + 1,
                                    pc,
                                    trace.disassembly,
                                    trace.original_instruction
                                )?;
                            }
                            None => {
                                writeln!(f, "| {} | 0x{:016X} | - | - |", j + 1, pc)?;
                            }
                        }
                    }
                    writeln!(f)?;
                }

                writeln!(f, "#### 描述")?;
                writeln!(f)?;
                writeln!(f, "{}", format_category_description(&cat_sum.category))?;
                writeln!(f)?;

                if i < self.categorized_summary.len() - 1 {
                    writeln!(f)?;
                }
            }
        }

        if !significant_diff_found {
            writeln!(f, "## 差异结果")?;
            writeln!(f)?;
            writeln!(f, "异常列表完全匹配，无差异！")?;
            writeln!(f)?;
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "异常差异报告生成时间: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

/// Compares two lists of `ExceptionDump`.
/// Matching is done based on mepc only - this is the ONLY criteria for exception identity.
/// All other fields (mcause, mtval, etc.) can differ and will be recorded as differences.
/// Assumes list1 is from sim1_type and list2 from sim2_type for categorization purposes.
pub fn compare_exception_dump_lists(
    list1: &[ExceptionDump],
    list2: &[ExceptionDump],
    sim1_type: EmulatorType,
    sim2_type: EmulatorType,
) -> ExceptionListDiff {
    let mut list1_only_exceptions = Vec::new();
    let mut paired_exceptions_diffs = Vec::new();
    let mut raw_diffs_for_categorization = Vec::<ExceptionDiffInfo>::new();

    // Key: mepc, Value: list of indices in list2
    let mut list2_map: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, ex2) in list2.iter().enumerate() {
        list2_map.entry(ex2.csrs.mepc).or_default().push(i);
    }

    let mut list2_matched_indices: Vec<bool> = vec![false; list2.len()];

    // Process list1 exceptions
    for ex1 in list1.iter() {
        let mepc = ex1.csrs.mepc;

        if let Some(indices_in_list2) = list2_map.get_mut(&mepc) {
            // Find the first unmatched exception in list2 with same mepc
            if let Some(idx_in_list2_vec) = indices_in_list2
                .iter()
                .position(|&idx2| !list2_matched_indices[idx2])
            {
                let list2_idx = indices_in_list2[idx_in_list2_vec];
                list2_matched_indices[list2_idx] = true;

                let ex2 = &list2[list2_idx];

                // Compare all CSR fields for differences
                let mut csrs_diffs_for_paired = Vec::new();
                compare_exception_csrs(&ex1.csrs, &ex2.csrs, &mut csrs_diffs_for_paired);

                // Always create a paired diff entry (even if no differences)
                // This represents that we found matching exceptions by mepc
                paired_exceptions_diffs.push(PairedExceptionDiff {
                    exception1: ex1.clone(),
                    exception2: ex2.clone(),
                    csrs_differences: csrs_diffs_for_paired.clone(),
                });

                // Add CSR differences to categorization (but NOT the fact that they matched)
                for (csr_name, val1, val2) in csrs_diffs_for_paired {
                    let instruction_trace = ex1.inst_trace.clone();
                    raw_diffs_for_categorization.push(ExceptionDiffInfo::CsrDifference {
                        pc: mepc,
                        csr_name: csr_name.clone(),
                        sim1_value: val1,
                        sim2_value: val2,
                        sim1_description: if csr_name == "mcause" {
                            Some(get_exception_description(val1))
                        } else {
                            None
                        },
                        sim2_description: if csr_name == "mcause" {
                            Some(get_exception_description(val2))
                        } else {
                            None
                        },
                        instruction_trace,
                    });
                }
            } else {
                // All exceptions with this mepc in list2 are already matched
                // This exception from list1 has no counterpart in list2
                list1_only_exceptions.push(ex1.clone());
                let instruction_trace = ex1.inst_trace.clone();
                raw_diffs_for_categorization.push(ExceptionDiffInfo::OnlyInSimulator {
                    simulator: sim1_type,
                    pc: mepc,
                    mcause: ex1.csrs.mcause,
                    description: get_exception_description(ex1.csrs.mcause),
                    instruction_trace,
                });
            }
        } else {
            // No exception in list2 has this mepc
            list1_only_exceptions.push(ex1.clone());
            let instruction_trace = ex1.inst_trace.clone();
            raw_diffs_for_categorization.push(ExceptionDiffInfo::OnlyInSimulator {
                simulator: sim1_type,
                pc: mepc,
                mcause: ex1.csrs.mcause,
                description: get_exception_description(ex1.csrs.mcause),
                instruction_trace,
            });
        }
    }

    // Process unmatched exceptions from list2
    let list2_only_exceptions: Vec<ExceptionDump> = list2
        .iter()
        .enumerate()
        .filter_map(|(i, ex2)| {
            if !list2_matched_indices[i] {
                let instruction_trace = ex2.inst_trace.clone();
                raw_diffs_for_categorization.push(ExceptionDiffInfo::OnlyInSimulator {
                    simulator: sim2_type,
                    pc: ex2.csrs.mepc,
                    mcause: ex2.csrs.mcause,
                    description: get_exception_description(ex2.csrs.mcause),
                    instruction_trace,
                });
                Some(ex2.clone())
            } else {
                None
            }
        })
        .collect();

    let categorized_summary = if !raw_diffs_for_categorization.is_empty() {
        analyze_and_categorize_exception_diffs(raw_diffs_for_categorization)
    } else {
        Vec::new()
    };

    ExceptionListDiff {
        sim1_emulator_type: sim1_type,
        sim2_emulator_type: sim2_type,
        list1_only_exceptions,
        list2_only_exceptions,
        paired_exceptions_diffs,
        categorized_summary,
    }
}

fn compare_exception_csrs(
    csrs1: &ExceptionCSRs,
    csrs2: &ExceptionCSRs,
    diff_list: &mut Vec<(String, u64, u64)>,
) {
    // NOTE: We compare ALL CSR fields, including mepc, even though mepc should be same
    // This is defensive programming in case there are floating point precision issues
    if csrs1.mstatus != csrs2.mstatus {
        diff_list.push(("mstatus".to_string(), csrs1.mstatus, csrs2.mstatus));
    }
    if csrs1.mcause != csrs2.mcause {
        diff_list.push(("mcause".to_string(), csrs1.mcause, csrs2.mcause));
    }
    if csrs1.mepc != csrs2.mepc {
        diff_list.push(("mepc".to_string(), csrs1.mepc, csrs2.mepc));
    }
    if csrs1.mtval != csrs2.mtval {
        diff_list.push(("mtval".to_string(), csrs1.mtval, csrs2.mtval));
    }
    if csrs1.mie != csrs2.mie {
        diff_list.push(("mie".to_string(), csrs1.mie, csrs2.mie));
    }
    if csrs1.mip != csrs2.mip {
        diff_list.push(("mip".to_string(), csrs1.mip, csrs2.mip));
    }
    if csrs1.mtvec != csrs2.mtvec {
        diff_list.push(("mtvec".to_string(), csrs1.mtvec, csrs2.mtvec));
    }
    if csrs1.mscratch != csrs2.mscratch {
        diff_list.push(("mscratch".to_string(), csrs1.mscratch, csrs2.mscratch));
    }
    if csrs1.mhartid != csrs2.mhartid {
        diff_list.push(("mhartid".to_string(), csrs1.mhartid, csrs2.mhartid));
    }
}

// Trait for types that can be diffed
pub trait Diffable {
    type DiffOutput;
    fn diff(&self, other: &Self) -> Self::DiffOutput;
}

impl Diffable for StandardExecutionOutput {
    type DiffOutput = StandardExecutionOutputDiff;
    fn diff(&self, other: &Self) -> Self::DiffOutput {
        standard_diff::compare_standard_execution_outputs(self, other)
    }
}

impl Diffable for DebugExecutionOutput {
    type DiffOutput = DebugExecutionOutputDiff;
    fn diff(&self, other: &Self) -> Self::DiffOutput {
        debug_diff::compare_debug_execution_outputs(self, other)
    }
}

impl Diffable for CommonExecutionOutput {
    type DiffOutput = CommonExecutionOutputDiff;
    fn diff(&self, other: &Self) -> Self::DiffOutput {
        common_diff::compare_execution_outputs(self, other)
    }
}

/// Generic function to compare two diffable outputs.
pub fn compare_outputs<T: Diffable>(output1: &T, output2: &T) -> T::DiffOutput {
    output1.diff(output2)
}
