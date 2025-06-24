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
        writeln!(f, "# Conversion Statistics Diff Change Report")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "No changes in conversion statistics differences")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.sim1_emulator_type.to_string();
        let sim2_name = self.sim2_emulator_type.to_string();

        writeln!(f, "Comparison: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        writeln!(f, "## Change Summary")?;
        writeln!(f)?;
        writeln!(f, "| Change Item | Change Status |")?;
        writeln!(f, "|:------------|:-------------:|")?;

        let mut change_count = 0;

        if self.original_exception_count_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| Original Exception Count | Changed |")?;
        }

        if self.original_register_count_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| Original Register Count | Changed |")?;
        }

        if self.conversion_successful_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| Conversion Success Status | Changed |")?;
        }

        if self.warnings_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| Warning Information | Changed |")?;
        }

        if change_count == 0 {
            writeln!(f, "| Total | No specific item changes |")?;
        }
        writeln!(f)?;

        writeln!(f, "## Detailed Change Analysis")?;
        writeln!(f)?;

        if let Some(ch) = &self.original_exception_count_changed_diff {
            writeln!(f, "### Original Exception Count Changes")?;
            writeln!(f)?;
            writeln!(f, "| Period | {} Count | {} Count |", sim1_name, sim2_name)?;
            writeln!(f, "|:-------|:--------:|:--------:|")?;

            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| Before | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| After | {} | {} |", new_s1, new_s2)?;
                }
                (None, Some((new_s1, new_s2))) => {
                    writeln!(f, "| Before | N/A | N/A |")?;
                    writeln!(f, "| After | {} | {} |", new_s1, new_s2)?;
                }
                (Some((old_s1, old_s2)), None) => {
                    writeln!(f, "| Before | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| After | N/A | N/A |")?;
                }
                (None, None) => {
                    writeln!(f, "| Before | N/A | N/A |")?;
                    writeln!(f, "| After | N/A | N/A |")?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.original_register_count_changed_diff {
            writeln!(f, "### Original Register Count Changes")?;
            writeln!(f)?;
            writeln!(f, "| Period | {} Count | {} Count |", sim1_name, sim2_name)?;
            writeln!(f, "|:-------|:--------:|:--------:|")?;

            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| Before | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| After | {} | {} |", new_s1, new_s2)?;
                }
                (None, Some((new_s1, new_s2))) => {
                    writeln!(f, "| Before | N/A | N/A |")?;
                    writeln!(f, "| After | {} | {} |", new_s1, new_s2)?;
                }
                (Some((old_s1, old_s2)), None) => {
                    writeln!(f, "| Before | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| After | N/A | N/A |")?;
                }
                (None, None) => {
                    writeln!(f, "| Before | N/A | N/A |")?;
                    writeln!(f, "| After | N/A | N/A |")?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.conversion_successful_changed_diff {
            writeln!(f, "### Conversion Success Status Changes")?;
            writeln!(f)?;
            writeln!(f, "| Period | {} Status | {} Status |", sim1_name, sim2_name)?;
            writeln!(f, "|:-------|:---------:|:---------:|")?;

            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(
                        f,
                        "| Before | {} | {} |",
                        if *old_s1 { "Success" } else { "Failed" },
                        if *old_s2 { "Success" } else { "Failed" }
                    )?;
                    writeln!(
                        f,
                        "| After | {} | {} |",
                        if *new_s1 { "Success" } else { "Failed" },
                        if *new_s2 { "Success" } else { "Failed" }
                    )?;
                }
                (None, Some((new_s1, new_s2))) => {
                    writeln!(f, "| Before | N/A | N/A |")?;
                    writeln!(
                        f,
                        "| After | {} | {} |",
                        if *new_s1 { "Success" } else { "Failed" },
                        if *new_s2 { "Success" } else { "Failed" }
                    )?;
                }
                (Some((old_s1, old_s2)), None) => {
                    writeln!(
                        f,
                        "| Before | {} | {} |",
                        if *old_s1 { "Success" } else { "Failed" },
                        if *old_s2 { "Success" } else { "Failed" }
                    )?;
                    writeln!(f, "| After | N/A | N/A |")?;
                }
                (None, None) => {
                    writeln!(f, "| Before | N/A | N/A |")?;
                    writeln!(f, "| After | N/A | N/A |")?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.warnings_changed_diff {
            writeln!(f, "### Warning Information Changes")?;
            writeln!(f)?;

            match (&ch.old, &ch.new) {
                (Some((old_w1, old_w2)), Some((new_w1, new_w2))) => {
                    writeln!(f, "#### {} Warnings (Before)", sim1_name)?;
                    for warn in old_w1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} Warnings (Before)", sim2_name)?;
                    for warn in old_w2 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} Warnings (After)", sim1_name)?;
                    for warn in new_w1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} Warnings (After)", sim2_name)?;
                    for warn in new_w2 {
                        writeln!(f, "- {}", warn)?;
                    }
                }
                (Some((old_warnings1, old_warnings2)), None) => {
                    writeln!(f, "#### {} Warnings (Before)", sim1_name)?;
                    for warn in old_warnings1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} Warnings (Before)", sim2_name)?;
                    for warn in old_warnings2 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} Warnings (After)", sim1_name)?;
                    writeln!(f, "- No data")?;
                    writeln!(f, "#### {} Warnings (After)", sim2_name)?;
                    writeln!(f, "- No data")?;
                }
                (None, Some((new_warnings1, new_warnings2))) => {
                    writeln!(f, "#### {} Warnings (Before)", sim1_name)?;
                    writeln!(f, "- No data")?;
                    writeln!(f, "#### {} Warnings (Before)", sim2_name)?;
                    writeln!(f, "- No data")?;
                    writeln!(f, "#### {} Warnings (After)", sim1_name)?;
                    for warn in new_warnings1 {
                        writeln!(f, "- {}", warn)?;
                    }
                    writeln!(f, "#### {} Warnings (After)", sim2_name)?;
                    for warn in new_warnings2 {
                        writeln!(f, "- {}", warn)?;
                    }
                }
                _ => {
                    writeln!(f, "Before: {:?}", ch.old)?;
                    writeln!(f, "After: {:?}", ch.new)?;
                }
            }
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "Report generated at: {}",
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
        writeln!(f, "## Register Dump Diff Change Report")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "No changes in register dump differences")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "Comparison: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        // Create change summary table
        writeln!(f, "### Change Summary")?;
        writeln!(f)?;
        writeln!(
            f,
            "| Register Type | Before Diff Count | After Diff Count | Net Change | Change Trend |"
        )?;
        writeln!(
            f,
            "|:--------------|:-----------------:|:----------------:|:----------:|:------------:|"
        )?;

        if let Some(ch) = &self.int_registers_diff_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| Integer Registers | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.core_csrs_diff_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| Core CSRs | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.float_registers_diff_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| Float Registers | {} | {} | {:+} | {} |",
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
                (0, 1) => "📈 New",
                (1, 0) => "📉 Resolved",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| Float CSRs | {} | {} | {:+} | {} |",
                old_count,
                new_count,
                new_count - old_count,
                trend
            )?;
        }
        writeln!(f)?;

        if let Some(ch) = &self.float_registers_status_changed_diff {
            writeln!(f, "### Float Register Status Changes")?;
            writeln!(f)?;
            writeln!(f, "| Period | {} Status | {} Status |", sim1_name, sim2_name)?;
            writeln!(f, "|:-------|:----------:|:----------:|")?;
            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| Before | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| After | {} | {} |", new_s1, new_s2)?;
                }
                _ => {
                    writeln!(f, "| Before | {:?} | - |", ch.old)?;
                    writeln!(f, "| After | {:?} | - |", ch.new)?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.float_csr_status_changed_diff {
            writeln!(f, "### Float CSR Status Changes")?;
            writeln!(f)?;
            writeln!(f, "| Period | {} Status | {} Status |", sim1_name, sim2_name)?;
            writeln!(f, "|:-------|:----------:|:----------:|")?;
            match (&ch.old, &ch.new) {
                (Some((old_s1, old_s2)), Some((new_s1, new_s2))) => {
                    writeln!(f, "| Before | {} | {} |", old_s1, old_s2)?;
                    writeln!(f, "| After | {} | {} |", new_s1, new_s2)?;
                }
                _ => {
                    writeln!(f, "| Before | {:?} | - |", ch.old)?;
                    writeln!(f, "| After | {:?} | - |", ch.new)?;
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
        writeln!(f, "## Exception List Diff Change Report")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "No changes in exception list differences")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "Comparison: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        // Create change summary table
        writeln!(f, "### Change Summary")?;
        writeln!(f)?;
        writeln!(
            f,
            "| Exception Type | Before Count | After Count | Net Change | Change Trend |"
        )?;
        writeln!(
            f,
            "|:---------------|:------------:|:-----------:|:----------:|:------------:|"
        )?;

        if let Some(ch) = &self.list1_only_exceptions_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| {} Only Exceptions | {} | {} | {:+} | {} |",
                sim1_name,
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.list2_only_exceptions_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| {} Only Exceptions | {} | {} | {:+} | {} |",
                sim2_name,
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.paired_exceptions_diffs_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| Paired Exception Diffs | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }

        if let Some(ch) = &self.categorized_summary_changed {
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(
                f,
                "| Categorized Summary | {} | {} | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;
        }
        writeln!(f)?;

        // Detailed analysis - only show when there are significant changes
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
            writeln!(f, "### Detailed Change Analysis")?;
            writeln!(f)?;

            if let Some(ch) = &self.categorized_summary_changed {
                if ch.old.len() != ch.new.len() {
                    writeln!(f, "#### Categorized Summary Category Details")?;
                    writeln!(f)?;
                    writeln!(f, "| Period | Category Count | Category Overview |")?;
                    writeln!(f, "|:-------|:--------------:|:------------------|")?;
                    writeln!(
                        f,
                        "| Before | {} | {} |",
                        ch.old.len(),
                        if ch.old.len() <= 3 {
                            "Few categories"
                        } else {
                            "Multiple category differences"
                        }
                    )?;
                    writeln!(
                        f,
                        "| After | {} | {} |",
                        ch.new.len(),
                        if ch.new.len() <= 3 {
                            "Few categories"
                        } else {
                            "Multiple category differences"
                        }
                    )?;
                    writeln!(f)?;
                }
            }
        }

        if let Some(ch) = &self.sim1_emulator_type_changed {
            writeln!(f, "### {} Simulator Type Change", sim1_name)?;
            writeln!(f, "Before: {}, After: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        if let Some(ch) = &self.sim2_emulator_type_changed {
            writeln!(f, "### {} Simulator Type Change", sim2_name)?;
            writeln!(f, "Before: {}, After: {}", ch.old, ch.new)?;
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
        writeln!(f, "# Standard Execution Output Diff Change Report")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "No changes in standard execution output differences")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "Comparison: {} ⚡ {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        writeln!(f, "## Change Details")?;
        writeln!(f)?;

        // Simulator type changes
        if let Some(ch) = &self.sim1_emulator_type_changed_diff {
            writeln!(f, "### {} Simulator Type Change", sim1_name)?;
            writeln!(f, "Before: {}, After: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        if let Some(ch) = &self.sim2_emulator_type_changed_diff {
            writeln!(f, "### {} Simulator Type Change", sim2_name)?;
            writeln!(f, "Before: {}, After: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // Register dump status difference changes
        if let Some(ch) = &self.register_dump_status_diff {
            writeln!(f, "### Register Dump Status Changes")?;
            writeln!(f, "Before: {:?}, After: {:?}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // Register dump presence status changes
        if let Some(ch) = &self.register_dump_diff_presence_changed {
            writeln!(f, "### Register Dump Presence Status Changes")?;
            writeln!(f, "Before: {}, After: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // Register dump content changes
        if let Some(content_diff) = &self.register_dump_diff_content_diff {
            writeln!(f, "### Register Dump Content Changes")?;
            writeln!(f, "{}", content_diff)?;
            writeln!(f)?;
        }

        // Exception difference presence status changes
        if let Some(ch) = &self.exceptions_diff_presence_changed {
            writeln!(f, "### Exception Difference Presence Status Changes")?;
            writeln!(f, "Before: {}, After: {}", ch.old, ch.new)?;
            writeln!(f)?;
        }

        // Exception difference content changes
        if let Some(content_diff) = &self.exceptions_diff_content_diff {
            writeln!(f, "### Exception Difference Content Changes")?;
            writeln!(f, "{}", content_diff)?;
            writeln!(f)?;
        }

        // Conversion statistics content changes
        if let Some(content_diff) = &self.conversion_stats_diff_content_diff {
            writeln!(f, "### Conversion Statistics Content Changes")?;
            writeln!(f, "{}", content_diff)?;
            writeln!(f)?;
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "Report generated at: {}",
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
