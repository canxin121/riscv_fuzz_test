use crate::emulators::EmulatorType;
use crate::output_diff::diff::standard_diff::{ConversionStatsDiff, StandardExecutionOutputDiff};
use crate::output_diff::diff::{
    CategorizedExceptionDiffs, ExceptionListDiff, PairedExceptionDiff, RegistersDumpDiff,
};
use crate::output_diff::diff_diff::Change;
use crate::output_parser::ExceptionDump;
use serde::{Deserialize, Serialize};
use std::fmt;

// --- ConversionStatsDiffDiff ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversionStatsDiffDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub original_exception_count_changed_diff: Option<Change<Option<(usize, usize)>>>,
    pub original_register_count_changed_diff: Option<Change<Option<(usize, usize)>>>,
    pub conversion_successful_changed_diff: Option<Change<Option<(bool, bool)>>>,
    pub warnings_changed_diff: Option<Change<Option<(Vec<String>, Vec<String>)>>>,
}

impl Default for ConversionStatsDiffDiff {
    fn default() -> Self {
        Self {
            sim1_emulator_type: EmulatorType::Spike,
            sim2_emulator_type: EmulatorType::Rocket,
            original_exception_count_changed_diff: None,
            original_register_count_changed_diff: None,
            conversion_successful_changed_diff: None,
            warnings_changed_diff: None,
        }
    }
}

impl ConversionStatsDiffDiff {
    pub fn is_empty(&self) -> bool {
        self.original_exception_count_changed_diff.is_none()
            && self.original_register_count_changed_diff.is_none()
            && self.conversion_successful_changed_diff.is_none()
            && self.warnings_changed_diff.is_none()
    }
}

impl fmt::Display for ConversionStatsDiffDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# 转换统计差异变化报告")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "转换统计差异无变化")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.sim1_emulator_type.to_string();
        let sim2_name = self.sim2_emulator_type.to_string();

        writeln!(f, "比较对象: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        writeln!(f, "## 变化汇总")?;
        writeln!(f)?;
        writeln!(f, "| 变化项目 | 变化状态 |")?;
        writeln!(f, "|:---------|:---------:|")?;

        let mut change_count = 0;

        if self.original_exception_count_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| 原始异常数量 | 变化 |")?;
        }

        if self.original_register_count_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| 原始寄存器数量 | 变化 |")?;
        }

        if self.conversion_successful_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| 转换成功状态 | 变化 |")?;
        }

        if self.warnings_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| 警告信息 | 变化 |")?;
        }

        if change_count == 0 {
            writeln!(f, "| 总计 | 无具体项目变化 |")?;
        }
        writeln!(f)?;

        writeln!(f, "## 详细变化分析")?;
        writeln!(f)?;

        if let Some(ch) = &self.original_exception_count_changed_diff {
            writeln!(f, "### 原始异常数量变化")?;
            writeln!(f)?;
            writeln!(f, "| 时期 | {} 数量 | {} 数量 |", sim1_name, sim2_name)?;
            writeln!(f, "|:-----|:------------:|:------------:|")?;

            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| 变化前 | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| 变化后 | {} | {} |", new_s1, new_s2)?;
                }
                (None, Some((new_s1, new_s2))) => {
                    writeln!(f, "| 变化前 | N/A | N/A |")?;
                    writeln!(f, "| 变化后 | {} | {} |", new_s1, new_s2)?;
                }
                (Some((old_s1, old_s2)), None) => {
                    writeln!(f, "| 变化前 | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| 变化后 | N/A | N/A |")?;
                }
                (None, None) => {
                    writeln!(f, "| 变化前 | N/A | N/A |")?;
                    writeln!(f, "| 变化后 | N/A | N/A |")?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.original_register_count_changed_diff {
            writeln!(f, "### 原始寄存器数量变化")?;
            writeln!(f)?;
            writeln!(f, "| 时期 | {} 数量 | {} 数量 |", sim1_name, sim2_name)?;
            writeln!(f, "|:-----|:------------:|:------------:|")?;

            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| 变化前 | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| 变化后 | {} | {} |", new_s1, new_s2)?;
                }
                (None, Some((new_s1, new_s2))) => {
                    writeln!(f, "| 变化前 | N/A | N/A |")?;
                    writeln!(f, "| 变化后 | {} | {} |", new_s1, new_s2)?;
                }
                (Some((old_s1, old_s2)), None) => {
                    writeln!(f, "| 变化前 | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| 变化后 | N/A | N/A |")?;
                }
                (None, None) => {
                    writeln!(f, "| 变化前 | N/A | N/A |")?;
                    writeln!(f, "| 变化后 | N/A | N/A |")?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.conversion_successful_changed_diff {
            writeln!(f, "### 转换成功状态变化")?;
            writeln!(f)?;
            writeln!(f, "| 时期 | {} 状态 | {} 状态 |", sim1_name, sim2_name)?;
            writeln!(f, "|:-----|:-------------:|:-------------:|")?;

            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(
                        f,
                        "| 变化前 | {} | {} |",
                        if *old_s1 { "成功" } else { "失败" },
                        if *old_s2 { "成功" } else { "失败" }
                    )?;
                    writeln!(
                        f,
                        "| 变化后 | {} | {} |",
                        if *new_s1 { "成功" } else { "失败" },
                        if *new_s2 { "成功" } else { "失败" }
                    )?;
                }
                (None, Some((new_s1, new_s2))) => {
                    writeln!(f, "| 变化前 | N/A | N/A |")?;
                    writeln!(
                        f,
                        "| 变化后 | {} | {} |",
                        if *new_s1 { "成功" } else { "失败" },
                        if *new_s2 { "成功" } else { "失败" }
                    )?;
                }
                (Some((old_s1, old_s2)), None) => {
                    writeln!(
                        f,
                        "| 变化前 | {} | {} |",
                        if *old_s1 { "成功" } else { "失败" },
                        if *old_s2 { "成功" } else { "失败" }
                    )?;
                    writeln!(f, "| 变化后 | N/A | N/A |")?;
                }
                (None, None) => {
                    writeln!(f, "| 变化前 | N/A | N/A |")?;
                    writeln!(f, "| 变化后 | N/A | N/A |")?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.warnings_changed_diff {
            writeln!(f, "### 警告信息变化")?;
            writeln!(f)?;

            match (&ch.old, &ch.new) {
                (Some((old_w1, old_w2)), Some((new_w1, new_w2))) => {
                    writeln!(f, "#### {} 警告 (变化前)", sim1_name)?;
                    for warn in old_w1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} 警告 (变化前)", sim2_name)?;
                    for warn in old_w2 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} 警告 (变化后)", sim1_name)?;
                    for warn in new_w1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} 警告 (变化后)", sim2_name)?;
                    for warn in new_w2 {
                        writeln!(f, "- {}", warn)?;
                    }
                }
                (Some((old_warnings1, old_warnings2)), None) => {
                    writeln!(f, "#### {} 警告 (变化前)", sim1_name)?;
                    for warn in old_warnings1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} 警告 (变化前)", sim2_name)?;
                    for warn in old_warnings2 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} 警告 (变化后)", sim1_name)?;
                    writeln!(f, "- 无数据")?;
                    writeln!(f, "#### {} 警告 (变化后)", sim2_name)?;
                    writeln!(f, "- 无数据")?;
                }
                (None, Some((new_warnings1, new_warnings2))) => {
                    writeln!(f, "#### {} 警告 (变化前)", sim1_name)?;
                    writeln!(f, "- 无数据")?;
                    writeln!(f, "#### {} 警告 (变化前)", sim2_name)?;
                    writeln!(f, "- 无数据")?;
                    writeln!(f, "#### {} 警告 (变化后)", sim1_name)?;
                    for warn in new_warnings1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} 警告 (变化后)", sim2_name)?;
                    for warn in new_warnings2 {
                        writeln!(f, "- {}", warn)?;
                    }
                }
                _ => {
                    writeln!(f, "变化前: {:?}", ch.old)?;
                    writeln!(f, "变化后: {:?}", ch.new)?;
                }
            }
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "报告生成时间: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

pub fn compare_conversion_stats_diffs(
    diff1: &ConversionStatsDiff,
    diff2: &ConversionStatsDiff,
) -> ConversionStatsDiffDiff {
    let mut ddiff = ConversionStatsDiffDiff {
        sim1_emulator_type: diff1.sim1_emulator_type, // 使用 diff1 中的类型
        sim2_emulator_type: diff1.sim2_emulator_type, // 使用 diff1 中的类型
        ..Default::default()
    };
    if diff1.original_exception_count_changed != diff2.original_exception_count_changed {
        ddiff.original_exception_count_changed_diff = Some(Change {
            old: diff1.original_exception_count_changed,
            new: diff2.original_exception_count_changed,
        });
    }
    if diff1.original_register_count_changed != diff2.original_register_count_changed {
        ddiff.original_register_count_changed_diff = Some(Change {
            old: diff1.original_register_count_changed,
            new: diff2.original_register_count_changed,
        });
    }
    if diff1.conversion_successful_changed != diff2.conversion_successful_changed {
        ddiff.conversion_successful_changed_diff = Some(Change {
            old: diff1.conversion_successful_changed,
            new: diff2.conversion_successful_changed,
        });
    }
    if diff1.warnings_changed != diff2.warnings_changed {
        ddiff.warnings_changed_diff = Some(Change {
            old: diff1.warnings_changed.clone(),
            new: diff2.warnings_changed.clone(),
        });
    }
    ddiff
}

// --- RegistersDumpDiffDiff ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistersDumpDiffDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub int_registers_diff_changed: Option<Change<Vec<(usize, String, u64, u64)>>>,
    pub core_csrs_diff_changed: Option<Change<Vec<(String, u64, u64)>>>,
    pub float_registers_status_changed_diff: Option<Change<Option<(String, String)>>>,
    pub float_registers_diff_changed: Option<Change<Vec<(usize, u64, u64)>>>,
    pub float_csr_status_changed_diff: Option<Change<Option<(String, String)>>>,
    pub float_csr_diff_changed: Option<Change<Option<(u64, u64)>>>,
}

impl Default for RegistersDumpDiffDiff {
    fn default() -> Self {
        Self {
            sim1_emulator_type: EmulatorType::Spike,
            sim2_emulator_type: EmulatorType::Rocket,
            int_registers_diff_changed: None,
            core_csrs_diff_changed: None,
            float_registers_status_changed_diff: None,
            float_registers_diff_changed: None,
            float_csr_status_changed_diff: None,
            float_csr_diff_changed: None,
        }
    }
}

impl RegistersDumpDiffDiff {
    pub fn is_empty(&self) -> bool {
        self.int_registers_diff_changed.is_none()
            && self.core_csrs_diff_changed.is_none()
            && self.float_registers_status_changed_diff.is_none()
            && self.float_registers_diff_changed.is_none()
            && self.float_csr_status_changed_diff.is_none()
            && self.float_csr_diff_changed.is_none()
    }

    fn get_sim1_name(&self) -> String {
        self.sim1_emulator_type.to_string()
    }

    fn get_sim2_name(&self) -> String {
        self.sim2_emulator_type.to_string()
    }
}

impl fmt::Display for RegistersDumpDiffDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "## 寄存器转储差异变化报告")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "寄存器转储差异无变化")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "比较对象: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        // 创建变化汇总表格
        writeln!(f, "### 变化汇总")?;
        writeln!(f)?;
        writeln!(
            f,
            "| 寄存器类型 | 变化前差异数 | 变化后差异数 | 净变化 | 变化趋势 |"
        )?;
        writeln!(
            f,
            "|:-----------|:------------:|:------------:|:------:|:--------:|"
        )?;

        if let Some(ch) = &self.int_registers_diff_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(
                f,
                "| 整数寄存器 | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.core_csrs_diff_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(
                f,
                "| 核心CSR | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.float_registers_diff_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(
                f,
                "| 浮点寄存器 | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.float_csr_diff_changed {
            let (old_count, new_count) = match (&ch.old, &ch.new) {
                (Some(_), Some(_)) => (1, 1),
                (Some(_), None) => (1, 0),
                (None, Some(_)) => (0, 1),
                (None, None) => (0, 0),
            };
            let trend = match (old_count, new_count) {
                (0, 1) => "📈 新增",
                (1, 0) => "消除",
                _ => "不变",
            };
            writeln!(
                f,
                "| 浮点CSR | {} | {} | {:+} | {} |",
                old_count,
                new_count,
                new_count - old_count,
                trend
            )?;
        }
        writeln!(f)?;

        if let Some(ch) = &self.float_registers_status_changed_diff {
            writeln!(f, "### 浮点寄存器状态变化")?;
            writeln!(f)?;
            writeln!(f, "| 时期 | {} 状态 | {} 状态 |", sim1_name, sim2_name)?;
            writeln!(f, "|:-----|:--------:|:--------:|")?;
            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| 变化前 | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| 变化后 | {} | {} |", new_s1, new_s2)?;
                }
                _ => {
                    writeln!(f, "| 变化前 | {:?} | - |", ch.old)?;
                    writeln!(f, "| 变化后 | {:?} | - |", ch.new)?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.float_csr_status_changed_diff {
            writeln!(f, "### 浮点CSR状态变化")?;
            writeln!(f)?;
            writeln!(f, "| 时期 | {} 状态 | {} 状态 |", sim1_name, sim2_name)?;
            writeln!(f, "|:-----|:--------:|:--------:|")?;
            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| 变化前 | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| 变化后 | {} | {} |", new_s1, new_s2)?;
                }
                _ => {
                    writeln!(f, "| 变化前 | {:?} | - |", ch.old)?;
                    writeln!(f, "| 变化后 | {:?} | - |", ch.new)?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

// --- ExceptionListDiffDiff ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExceptionListDiffDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub sim1_emulator_type_changed: Option<Change<EmulatorType>>,
    pub sim2_emulator_type_changed: Option<Change<EmulatorType>>,
    pub list1_only_exceptions_changed: Option<Change<Vec<ExceptionDump>>>,
    pub list2_only_exceptions_changed: Option<Change<Vec<ExceptionDump>>>,
    pub paired_exceptions_diffs_changed: Option<Change<Vec<PairedExceptionDiff>>>,
    pub categorized_summary_changed: Option<Change<Vec<CategorizedExceptionDiffs>>>,
}

impl Default for ExceptionListDiffDiff {
    fn default() -> Self {
        Self {
            sim1_emulator_type: EmulatorType::Spike,
            sim2_emulator_type: EmulatorType::Rocket,
            sim1_emulator_type_changed: None,
            sim2_emulator_type_changed: None,
            list1_only_exceptions_changed: None,
            list2_only_exceptions_changed: None,
            paired_exceptions_diffs_changed: None,
            categorized_summary_changed: None,
        }
    }
}

impl ExceptionListDiffDiff {
    pub fn is_empty(&self) -> bool {
        self.sim1_emulator_type_changed.is_none()
            && self.sim2_emulator_type_changed.is_none()
            && self.list1_only_exceptions_changed.is_none()
            && self.list2_only_exceptions_changed.is_none()
            && self.paired_exceptions_diffs_changed.is_none()
            && self.categorized_summary_changed.is_none()
    }

    fn get_sim1_name(&self) -> String {
        self.sim1_emulator_type.to_string()
    }

    fn get_sim2_name(&self) -> String {
        self.sim2_emulator_type.to_string()
    }
}

impl fmt::Display for ExceptionListDiffDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "## 异常列表差异变化报告")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "异常列表差异无变化")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "比较对象: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        // 创建变化汇总表格
        writeln!(f, "### 变化汇总")?;
        writeln!(f)?;
        writeln!(
            f,
            "| 异常类型 | 变化前数量 | 变化后数量 | 净变化 | 变化趋势 |"
        )?;
        writeln!(
            f,
            "|:---------|:----------:|:----------:|:------:|:--------:|"
        )?;

        if let Some(ch) = &self.list1_only_exceptions_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(
                f,
                "| 仅 {} 异常 | {} | {} | {:+} | {} |",
                sim1_name,
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.list2_only_exceptions_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(
                f,
                "| 仅 {} 异常 | {} | {} | {:+} | {} |",
                sim2_name,
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.paired_exceptions_diffs_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(
                f,
                "| 配对异常差异 | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.categorized_summary_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(
                f,
                "| 分类摘要 | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }
        writeln!(f)?;

        // 详细分析 - 只有在有显著变化时才显示
        let has_significant_changes = self
            .list1_only_exceptions_changed
            .as_ref()
            .map_or(false, |ch| ch.old.len() != ch.new.len())
            || self
                .list2_only_exceptions_changed
                .as_ref()
                .map_or(false, |ch| ch.old.len() != ch.new.len())
            || self
                .paired_exceptions_diffs_changed
                .as_ref()
                .map_or(false, |ch| ch.old.len() != ch.new.len())
            || self
                .categorized_summary_changed
                .as_ref()
                .map_or(false, |ch| ch.old.len() != ch.new.len());

        if has_significant_changes {
            writeln!(f, "### 详细变化分析")?;
            writeln!(f)?;

            if let Some(ch) = &self.categorized_summary_changed {
                if ch.old.len() != ch.new.len() {
                    writeln!(f, "#### 分类摘要类别详情")?;
                    writeln!(f)?;
                    writeln!(f, "| 时期 | 类别数量 | 类别概览 |")?;
                    writeln!(f, "|:-----|:--------:|:---------|")?;
                    writeln!(
                        f,
                        "| 变化前 | {} | {} |",
                        ch.old.len(),
                        if ch.old.len() <= 3 {
                            "少量类别"
                        } else {
                            "多类别差异"
                        }
                    )?;
                    writeln!(
                        f,
                        "| 变化后 | {} | {} |",
                        ch.new.len(),
                        if ch.new.len() <= 3 {
                            "少量类别"
                        } else {
                            "多类别差异"
                        }
                    )?;
                    writeln!(f)?;
                }
            }
        }

        if let Some(ch) = &self.sim1_emulator_type_changed {
            writeln!(f, "### {} 模拟器类型变化", sim1_name)?;
            writeln!(f, "变化前: {}, 变化后: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        if let Some(ch) = &self.sim2_emulator_type_changed {
            writeln!(f, "### {} 模拟器类型变化", sim2_name)?;
            writeln!(f, "变化前: {}, 变化后: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        Ok(())
    }
}

pub fn compare_exception_list_diffs(
    diff1: &ExceptionListDiff,
    diff2: &ExceptionListDiff,
) -> ExceptionListDiffDiff {
    let mut ddiff = ExceptionListDiffDiff {
        sim1_emulator_type: diff1.sim1_emulator_type,
        sim2_emulator_type: diff1.sim2_emulator_type,
        ..Default::default()
    };
    if diff1.sim1_emulator_type != diff2.sim1_emulator_type {
        ddiff.sim1_emulator_type_changed = Some(Change {
            old: diff1.sim1_emulator_type,
            new: diff2.sim1_emulator_type,
        });
    }
    if diff1.sim2_emulator_type != diff2.sim2_emulator_type {
        ddiff.sim2_emulator_type_changed = Some(Change {
            old: diff1.sim2_emulator_type,
            new: diff2.sim2_emulator_type,
        });
    }
    if diff1.list1_only_exceptions != diff2.list1_only_exceptions {
        ddiff.list1_only_exceptions_changed = Some(Change {
            old: diff1.list1_only_exceptions.clone(),
            new: diff2.list1_only_exceptions.clone(),
        });
    }
    if diff1.list2_only_exceptions != diff2.list2_only_exceptions {
        ddiff.list2_only_exceptions_changed = Some(Change {
            old: diff1.list2_only_exceptions.clone(),
            new: diff2.list2_only_exceptions.clone(),
        });
    }
    if diff1.paired_exceptions_diffs != diff2.paired_exceptions_diffs {
        ddiff.paired_exceptions_diffs_changed = Some(Change {
            old: diff1.paired_exceptions_diffs.clone(),
            new: diff2.paired_exceptions_diffs.clone(),
        });
    }
    if diff1.categorized_summary != diff2.categorized_summary {
        ddiff.categorized_summary_changed = Some(Change {
            old: diff1.categorized_summary.clone(),
            new: diff2.categorized_summary.clone(),
        });
    }
    ddiff
}

// --- StandardExecutionOutputDiffDiff ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StandardExecutionOutputDiffDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub sim1_emulator_type_changed_diff: Option<Change<EmulatorType>>,
    pub sim2_emulator_type_changed_diff: Option<Change<EmulatorType>>,
    pub exceptions_diff_presence_changed: Option<Change<bool>>,
    pub exceptions_diff_content_diff: Option<ExceptionListDiffDiff>,
    pub register_dump_status_diff: Option<Change<Option<String>>>,
    pub register_dump_diff_presence_changed: Option<Change<bool>>,
    pub register_dump_diff_content_diff: Option<RegistersDumpDiffDiff>,
    pub conversion_stats_diff_content_diff: Option<ConversionStatsDiffDiff>,
}

impl Default for StandardExecutionOutputDiffDiff {
    fn default() -> Self {
        Self {
            sim1_emulator_type: EmulatorType::Spike,
            sim2_emulator_type: EmulatorType::Rocket,
            sim1_emulator_type_changed_diff: None,
            sim2_emulator_type_changed_diff: None,
            exceptions_diff_presence_changed: None,
            exceptions_diff_content_diff: None,
            register_dump_status_diff: None,
            register_dump_diff_presence_changed: None,
            register_dump_diff_content_diff: None,
            conversion_stats_diff_content_diff: None,
        }
    }
}

impl fmt::Display for StandardExecutionOutputDiffDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# 标准执行输出差异变化报告")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "标准执行输出差异无变化")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "比较对象: {} ⚡ {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        writeln!(f, "## 变化详情")?;
        writeln!(f)?;

        // 模拟器类型变化
        if let Some(ch) = &self.sim1_emulator_type_changed_diff {
            writeln!(f, "### {} 模拟器类型变化", sim1_name)?;
            writeln!(f, "变化前: {}, 变化后: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        if let Some(ch) = &self.sim2_emulator_type_changed_diff {
            writeln!(f, "### {} 模拟器类型变化", sim2_name)?;
            writeln!(f, "变化前: {}, 变化后: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // 寄存器转储状态差异变化
        if let Some(ch) = &self.register_dump_status_diff {
            writeln!(f, "### 寄存器转储状态变化")?;
            writeln!(f, "变化前: {:?}, 变化后: {:?}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // 寄存器转储存在状态变化
        if let Some(ch) = &self.register_dump_diff_presence_changed {
            writeln!(f, "### 寄存器转储存在状态变化")?;
            writeln!(f, "变化前: {}, 变化后: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // 寄存器转储内容变化
        if let Some(content_diff) = &self.register_dump_diff_content_diff {
            writeln!(f, "### 寄存器转储内容变化")?;
            writeln!(f, "{}", content_diff)?;
            writeln!(f)?;
        }

        // 异常差异存在状态变化
        if let Some(ch) = &self.exceptions_diff_presence_changed {
            writeln!(f, "### 异常差异存在状态变化")?;
            writeln!(f, "变化前: {}, 变化后: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // 异常差异内容变化
        if let Some(content_diff) = &self.exceptions_diff_content_diff {
            writeln!(f, "### 异常差异内容变化")?;
            writeln!(f, "{}", content_diff)?;
            writeln!(f)?;
        }

        // 转换统计内容变化
        if let Some(content_diff) = &self.conversion_stats_diff_content_diff {
            writeln!(f, "### 转换统计内容变化")?;
            writeln!(f, "{}", content_diff)?;
            writeln!(f)?;
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "报告生成时间: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

pub fn compare_standard_execution_output_diffs(
    diff1: &StandardExecutionOutputDiff,
    diff2: &StandardExecutionOutputDiff,
) -> StandardExecutionOutputDiffDiff {
    let mut ddiff = StandardExecutionOutputDiffDiff {
        sim1_emulator_type: diff1.sim1_emulator_type,
        sim2_emulator_type: diff1.sim2_emulator_type,
        ..Default::default()
    };

    if diff1.sim1_emulator_type != diff2.sim1_emulator_type {
        ddiff.sim1_emulator_type_changed_diff = Some(Change {
            old: diff1.sim1_emulator_type,
            new: diff2.sim1_emulator_type,
        });
    }
    if diff1.sim2_emulator_type != diff2.sim2_emulator_type {
        ddiff.sim2_emulator_type_changed_diff = Some(Change {
            old: diff1.sim2_emulator_type,
            new: diff2.sim2_emulator_type,
        });
    }

    let ex_diff1_present = diff1.exceptions_diff.is_some();
    let ex_diff2_present = diff2.exceptions_diff.is_some();
    if ex_diff1_present != ex_diff2_present {
        ddiff.exceptions_diff_presence_changed = Some(Change {
            old: ex_diff1_present,
            new: ex_diff2_present,
        });
    }
    if let (Some(ex1), Some(ex2)) = (&diff1.exceptions_diff, &diff2.exceptions_diff) {
        let mut content_ddiff = compare_exception_list_diffs(ex1, ex2);
        content_ddiff.sim1_emulator_type = ddiff.sim1_emulator_type;
        content_ddiff.sim2_emulator_type = ddiff.sim2_emulator_type;
        if !content_ddiff.is_empty() {
            ddiff.exceptions_diff_content_diff = Some(content_ddiff);
        }
    }

    if diff1.register_dump_status != diff2.register_dump_status {
        ddiff.register_dump_status_diff = Some(Change {
            old: diff1.register_dump_status.clone(),
            new: diff2.register_dump_status.clone(),
        });
    }

    let reg_dump_diff1_present = diff1.register_dump_diff.is_some();
    let reg_dump_diff2_present = diff2.register_dump_diff.is_some();
    if reg_dump_diff1_present != reg_dump_diff2_present {
        ddiff.register_dump_diff_presence_changed = Some(Change {
            old: reg_dump_diff1_present,
            new: reg_dump_diff2_present,
        });
    }
    if let (Some(rd1), Some(rd2)) = (&diff1.register_dump_diff, &diff2.register_dump_diff) {
        let mut content_ddiff = compare_registers_dump_diffs(rd1, rd2);
        content_ddiff.sim1_emulator_type = ddiff.sim1_emulator_type;
        content_ddiff.sim2_emulator_type = ddiff.sim2_emulator_type;
        if !content_ddiff.is_empty() {
            ddiff.register_dump_diff_content_diff = Some(content_ddiff);
        }
    }

    if let (Some(cs1), Some(cs2)) = (&diff1.conversion_stats_diff, &diff2.conversion_stats_diff) {
        let mut content_ddiff = compare_conversion_stats_diffs(cs1, cs2);
        content_ddiff.sim1_emulator_type = cs1.sim1_emulator_type;
        content_ddiff.sim2_emulator_type = cs1.sim2_emulator_type;
        if !content_ddiff.is_empty() {
            ddiff.conversion_stats_diff_content_diff = Some(content_ddiff);
        }
    }

    ddiff
}

pub fn compare_registers_dump_diffs(
    diff1: &RegistersDumpDiff,
    diff2: &RegistersDumpDiff,
) -> RegistersDumpDiffDiff {
    let mut ddiff = RegistersDumpDiffDiff::default();

    if diff1.int_registers_diff != diff2.int_registers_diff {
        ddiff.int_registers_diff_changed = Some(Change {
            old: diff1.int_registers_diff.clone(),
            new: diff2.int_registers_diff.clone(),
        });
    }
    if diff1.core_csrs_diff != diff2.core_csrs_diff {
        ddiff.core_csrs_diff_changed = Some(Change {
            old: diff1.core_csrs_diff.clone(),
            new: diff2.core_csrs_diff.clone(),
        });
    }
    if diff1.float_registers_status_changed != diff2.float_registers_status_changed {
        ddiff.float_registers_status_changed_diff = Some(Change {
            old: diff1.float_registers_status_changed.clone(),
            new: diff2.float_registers_status_changed.clone(),
        });
    }
    if diff1.float_registers_diff != diff2.float_registers_diff {
        ddiff.float_registers_diff_changed = Some(Change {
            old: diff1.float_registers_diff.clone(),
            new: diff2.float_registers_diff.clone(),
        });
    }
    if diff1.float_csr_status_changed != diff2.float_csr_status_changed {
        ddiff.float_csr_status_changed_diff = Some(Change {
            old: diff1.float_csr_status_changed.clone(),
            new: diff2.float_csr_status_changed.clone(),
        });
    }
    if diff1.float_csr_diff != diff2.float_csr_diff {
        ddiff.float_csr_diff_changed = Some(Change {
            old: diff1.float_csr_diff,
            new: diff2.float_csr_diff,
        });
    }
    ddiff
}

impl StandardExecutionOutputDiffDiff {
    pub fn is_empty(&self) -> bool {
        self.sim1_emulator_type_changed_diff.is_none()
            && self.sim2_emulator_type_changed_diff.is_none()
            && self.exceptions_diff_presence_changed.is_none()
            && self
                .exceptions_diff_content_diff
                .as_ref()
                .map_or(true, |d| d.is_empty())
            && self.register_dump_status_diff.is_none()
            && self.register_dump_diff_presence_changed.is_none()
            && self
                .register_dump_diff_content_diff
                .as_ref()
                .map_or(true, |d| d.is_empty())
            && self
                .conversion_stats_diff_content_diff
                .as_ref()
                .map_or(true, |d| d.is_empty())
    }

    fn get_sim1_name(&self) -> String {
        self.sim1_emulator_type.to_string()
    }

    fn get_sim2_name(&self) -> String {
        self.sim2_emulator_type.to_string()
    }
}
