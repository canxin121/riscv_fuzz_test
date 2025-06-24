use crate::emulators::EmulatorType;
use crate::output_diff::diff::RegistersDumpDiff;
use crate::output_diff::diff::common_diff::CommonExecutionOutputDiff;
use crate::output_diff::diff_diff::Change;
use crate::output_diff::diff_diff::standard_diff_diff::{
    ExceptionListDiffDiff, compare_exception_list_diffs,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommonExecutionOutputDiffDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub sim1_emulator_type_changed_diff: Option<Change<EmulatorType>>,
    pub sim2_emulator_type_changed_diff: Option<Change<EmulatorType>>,
    pub output_items_status_diff: Option<Change<Option<String>>>,
    pub register_dumps_count_changed_diff: Option<Change<Option<(usize, usize)>>>,
    pub differing_register_dumps_changed: Option<Change<Vec<(usize, RegistersDumpDiff)>>>,
    pub exception_dumps_diff_presence_changed: Option<Change<bool>>,
    pub exception_dumps_diff_content_diff: Option<ExceptionListDiffDiff>,
}

impl Default for CommonExecutionOutputDiffDiff {
    fn default() -> Self {
        Self {
            sim1_emulator_type: EmulatorType::Spike,
            sim2_emulator_type: EmulatorType::Rocket,
            sim1_emulator_type_changed_diff: None,
            sim2_emulator_type_changed_diff: None,
            output_items_status_diff: None,
            register_dumps_count_changed_diff: None,
            differing_register_dumps_changed: None,
            exception_dumps_diff_presence_changed: None,
            exception_dumps_diff_content_diff: None,
        }
    }
}

impl CommonExecutionOutputDiffDiff {
    pub fn is_empty(&self) -> bool {
        self.sim1_emulator_type_changed_diff.is_none()
            && self.sim2_emulator_type_changed_diff.is_none()
            && self.output_items_status_diff.is_none()
            && self.register_dumps_count_changed_diff.is_none()
            && self.differing_register_dumps_changed.is_none()
            && self.exception_dumps_diff_presence_changed.is_none()
            && self
                .exception_dumps_diff_content_diff
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

impl fmt::Display for CommonExecutionOutputDiffDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# Common Execution Output Diff Change Report")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "No changes in common execution output differences")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "Comparison: {} ⚡ {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        // Change summary table
        writeln!(f, "## Change Summary")?;
        writeln!(f)?;
        writeln!(f, "| Change Item | Status | Details |")?;
        writeln!(f, "|:------------|:------:|:--------|")?;

        let mut change_count = 0;

        if let Some(ch) = &self.register_dumps_count_changed_diff {
            change_count += 1;
            let detail = match (&ch.old, &ch.new) {
                (Some((old1, old2)), Some((new1, new2))) => {
                    format!("{}→{} vs {}→{}", old1, new1, old2, new2)
                }
                _ => format!("{:?} → {:?}", ch.old, ch.new)
            };
            writeln!(f, "| Register Dump Count | 🔄 Changed | {} |", detail)?;
        }

        if let Some(ch) = &self.differing_register_dumps_changed {
            change_count += 1;
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 Increased",
                (old, new) if new < old => "📉 Decreased",
                _ => "⏸️ Unchanged",
            };
            writeln!(f, "| Register Content Differences | {} | {}→{} differing dumps |", 
                trend, ch.old.len(), ch.new.len())?;
        }

        if self.exception_dumps_diff_presence_changed.is_some() {
            change_count += 1;
            let ch = self.exception_dumps_diff_presence_changed.as_ref().unwrap();
            let status = match (ch.old, ch.new) {
                (false, true) => "✅ New Exception Differences",
                (true, false) => "❌ Exception Differences Resolved", 
                _ => "🔄 Exception Difference Status Changed",
            };
            writeln!(f, "| Exception Dump Differences | {} | Presence status changed |", status)?;
        }

        if let Some(ch) = &self.output_items_status_diff {
            change_count += 1;
            writeln!(f, "| Output Item Status | 🔄 Changed | {:?} → {:?} |", ch.old, ch.new)?;
        }

        if change_count == 0 {
            writeln!(f, "| - | ✅ No Changes | All items remain consistent |")?;
        }
        writeln!(f)?;

        // Detailed change analysis
        writeln!(f, "## Detailed Change Analysis")?;
        writeln!(f)?;

        if let Some(ch) = &self.register_dumps_count_changed_diff {
            writeln!(f, "### Register Dump Count Difference Changes")?;
            writeln!(f)?;
            writeln!(f, "| Period | {} Dump Count | {} Dump Count | Difference | Difference Rate |", sim1_name, sim2_name)?;
            writeln!(f, "|:-------|:-------------:|:-------------:|:----------:|:---------------:|")?;
            match (&ch.old, &ch.new) {
                (Some((old1, old2)), Some((new1, new2))) => {
                    let old_diff = (*old2 as i64 - *old1 as i64).abs();
                    let new_diff = (*new2 as i64 - *new1 as i64).abs();
                    let old_rate = if *old1.max(old2) > 0 { 
                        (old_diff as f64 / *old1.max(old2) as f64) * 100.0 
                    } else { 0.0 };
                    let new_rate = if *new1.max(new2) > 0 { 
                        (new_diff as f64 / *new1.max(new2) as f64) * 100.0 
                    } else { 0.0 };
                    writeln!(f, "| Before | {} | {} | {} | {:.1}% |", old1, old2, old_diff, old_rate)?;
                    writeln!(f, "| After | {} | {} | {} | {:.1}% |", new1, new2, new_diff, new_rate)?;
                }
                _ => {
                    writeln!(f, "| Before | {:?} | - | - | - |", ch.old)?;
                    writeln!(f, "| After | {:?} | - | - | - |", ch.new)?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.differing_register_dumps_changed {
            writeln!(f, "### Register Content Difference Changes")?;
            writeln!(f)?;
            writeln!(f, "| Metric | Before | After | Net Change | Impact Assessment |")?;
            writeln!(f, "|:-------|:------:|:-----:|:----------:|:------------------|")?;

            let net_change = ch.new.len() as i64 - ch.old.len() as i64;
            let impact = match net_change {
                x if x > 5 => "⚠️ Significant Increase",
                x if x > 0 => "📈 Slight Increase", 
                0 => "✅ Stable",
                x if x > -5 => "📉 Slight Decrease",
                _ => "✅ Significant Improvement",
            };

            writeln!(f, "| Differing Dump Count | {} | {} | {:+} | {} |", 
                ch.old.len(), ch.new.len(), net_change, impact)?;

            let consistency = if ch.old.is_empty() && ch.new.is_empty() {
                "🎯 Fully Consistent"
            } else if ch.old.is_empty() {
                "⚠️ New Differences"
            } else if ch.new.is_empty() {
                "✅ Fully Fixed"
            } else {
                "🔄 Partial Differences"
            };

            writeln!(f, "| Consistency Status | {} | {} | - | {} |", 
                if ch.old.is_empty() { "Consistent" } else { "Has Differences" },
                if ch.new.is_empty() { "Consistent" } else { "Has Differences" },
                consistency)?;
            writeln!(f)?;

            if !ch.old.is_empty() || !ch.new.is_empty() {
                writeln!(f, "#### Differing Dump Index Comparison")?;
                writeln!(f)?;
                writeln!(f, "| Period | Dump Index List |")?;
                writeln!(f, "|--------|-----------------|")?;

                if !ch.old.is_empty() {
                    let old_indices: Vec<String> =
                        ch.old.iter().map(|(idx, _)| (idx + 1).to_string()).collect();
                    writeln!(f, "| Before | {} |", old_indices.join(", "))?;
                } else {
                    writeln!(f, "| Before | No differing dumps |")?;
                }

                if !ch.new.is_empty() {
                    let new_indices: Vec<String> =
                        ch.new.iter().map(|(idx, _)| (idx + 1).to_string()).collect();
                    writeln!(f, "| After | {} |", new_indices.join(", "))?;
                } else {
                    writeln!(f, "| After | No differing dumps |")?;
                }
                writeln!(f)?;
            }
        }

        if let Some(content_diff) = &self.exception_dumps_diff_content_diff {
            if !content_diff.is_empty() {
                writeln!(f, "### Exception Dump Difference Content Changes")?;
                writeln!(f, "{}", content_diff)?;
            }
        }

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

        writeln!(f, "---")?;
        writeln!(
            f,
            "Common output diff change report generated at: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

pub fn compare_common_execution_output_diffs(
    diff1: &CommonExecutionOutputDiff,
    diff2: &CommonExecutionOutputDiff,
) -> CommonExecutionOutputDiffDiff {
    let mut ddiff = CommonExecutionOutputDiffDiff {
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

    if diff1.output_items_status != diff2.output_items_status {
        ddiff.output_items_status_diff = Some(Change {
            old: diff1.output_items_status.clone(),
            new: diff2.output_items_status.clone(),
        });
    }
    if diff1.register_dumps_count_changed != diff2.register_dumps_count_changed {
        ddiff.register_dumps_count_changed_diff = Some(Change {
            old: diff1.register_dumps_count_changed,
            new: diff2.register_dumps_count_changed,
        });
    }
    if diff1.differing_register_dumps != diff2.differing_register_dumps {
        ddiff.differing_register_dumps_changed = Some(Change {
            old: diff1.differing_register_dumps.clone(),
            new: diff2.differing_register_dumps.clone(),
        });
    }

    let ex_dumps_diff1_present = diff1.exception_dumps_diff.is_some();
    let ex_dumps_diff2_present = diff2.exception_dumps_diff.is_some();
    if ex_dumps_diff1_present != ex_dumps_diff2_present {
        ddiff.exception_dumps_diff_presence_changed = Some(Change {
            old: ex_dumps_diff1_present,
            new: ex_dumps_diff2_present,
        });
    }
    if let (Some(ex1), Some(ex2)) = (&diff1.exception_dumps_diff, &diff2.exception_dumps_diff) {
        let content_ddiff = compare_exception_list_diffs(ex1, ex2);
        if !content_ddiff.is_empty() {
            ddiff.exception_dumps_diff_content_diff = Some(content_ddiff);
        }
    }

    ddiff
}
