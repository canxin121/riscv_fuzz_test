use crate::emulators::EmulatorType;
use crate::output_diff::diff::{
    ExceptionListDiff, RegistersDumpDiff, compare_exception_dump_lists, compare_registers_dumps,
};
use crate::output_parser::standard::{ConversionStats, StandardExecutionOutput};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionStatsDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub original_exception_count_changed: Option<(usize, usize)>,
    pub original_register_count_changed: Option<(usize, usize)>,
    pub conversion_successful_changed: Option<(bool, bool)>,
    pub warnings_changed: Option<(Vec<String>, Vec<String>)>,
}

impl ConversionStatsDiff {
    pub fn is_empty(&self) -> bool {
        self.original_exception_count_changed.is_none()
            && self.original_register_count_changed.is_none()
            && self.conversion_successful_changed.is_none()
            && self.warnings_changed.is_none()
    }
}

impl fmt::Display for ConversionStatsDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# Conversion Statistics Diff")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "No differences found in conversion statistics")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.sim1_emulator_type.to_string();
        let sim2_name = self.sim2_emulator_type.to_string();

        writeln!(f, "| Statistics | {} | {} | Difference |", sim1_name, sim2_name)?;
        writeln!(f, "|------------|------------|------------|------------|")?;

        if let Some((v1, v2)) = self.original_exception_count_changed {
            writeln!(
                f,
                "| Original Exception Count | {} | {} | {} |",
                v1,
                v2,
                (v2 as i64 - v1 as i64).abs()
            )?;
        }

        if let Some((v1, v2)) = self.original_register_count_changed {
            writeln!(
                f,
                "| Original Register Count | {} | {} | {} |",
                v1,
                v2,
                (v2 as i64 - v1 as i64).abs()
            )?;
        }

        if let Some((v1, v2)) = self.conversion_successful_changed {
            let status1 = if v1 { "Success" } else { "Failed" };
            let status2 = if v2 { "Success" } else { "Failed" };
            writeln!(
                f,
                "| Conversion Success Status | {} | {} | {} |",
                status1,
                status2,
                if v1 != v2 { "Different" } else { "Same" }
            )?;
        }

        if let Some((v1, v2)) = &self.warnings_changed {
            writeln!(
                f,
                "| Warning Count | {} | {} | {} |",
                v1.len(),
                v2.len(),
                (v2.len() as i64 - v1.len() as i64).abs()
            )?;
        }
        writeln!(f)?;

        // Warning details
        if let Some((v1, v2)) = &self.warnings_changed {
            if !v1.is_empty() || !v2.is_empty() {
                writeln!(f, "## Warning Details")?;
                writeln!(f)?;

                if !v1.is_empty() {
                    writeln!(f, "### {} Warnings", sim1_name)?;
                    writeln!(f)?;
                    for (i, warning) in v1.iter().enumerate() {
                        writeln!(f, "{}. {}", i + 1, warning)?;
                    }
                    writeln!(f)?;
                }

                if !v2.is_empty() {
                    writeln!(f, "### {} Warnings", sim2_name)?;
                    writeln!(f)?;
                    for (i, warning) in v2.iter().enumerate() {
                        writeln!(f, "{}. {}", i + 1, warning)?;
                    }
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardExecutionOutputDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub exceptions_diff: Option<ExceptionListDiff>,
    pub register_dump_status: Option<String>,
    pub register_dump_diff: Option<RegistersDumpDiff>,
    pub conversion_stats_diff: Option<ConversionStatsDiff>,
}

impl StandardExecutionOutputDiff {
    pub fn is_empty(&self) -> bool {
        self.exceptions_diff.as_ref().map_or(true, |e| e.is_empty())
            && self.register_dump_status.is_none()
            && self
                .register_dump_diff
                .as_ref()
                .map_or(true, |r| r.is_empty())
            && self
                .conversion_stats_diff
                .as_ref()
                .map_or(true, |c| c.is_empty())
    }
}

impl fmt::Display for StandardExecutionOutputDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sim1_name = self.sim1_emulator_type.to_string();
        let sim2_name = self.sim2_emulator_type.to_string();

        writeln!(f, "# Standard Execution Output Diff Report")?;
        writeln!(f)?;
        writeln!(f, "Comparison: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "## Diff Result")?;
            writeln!(f)?;
            writeln!(f, "No significant differences found - standard outputs from both simulators match exactly!")?;
            writeln!(f)?;
            return Ok(());
        }

        writeln!(f, "## Detected Differences")?;
        writeln!(f)?;

        // Difference summary table
        let mut diff_count = 0;
        writeln!(f, "| Diff Type | Count |")?;
        writeln!(f, "|----------|------|")?;

        if self.register_dump_status.is_some() || self.register_dump_diff.is_some() {
            diff_count += 1;
            writeln!(f, "| Register Dump | Dump status or content differs |")?;
        }

        if self.exceptions_diff.is_some() {
            diff_count += 1;
            writeln!(f, "| Exception Diff | Exception information differs |")?;
        }

        if self.conversion_stats_diff.is_some() {
            diff_count += 1;
            writeln!(f, "| Conversion Stats | Conversion process statistics differ |")?;
        }

        if diff_count == 0 {
            writeln!(f, "| - | No Differences |")?;
        }
        writeln!(f)?;

        // Detailed difference information
        writeln!(f, "## Detailed Diff Analysis")?;
        writeln!(f)?;

        if let Some(status) = &self.register_dump_status {
            writeln!(f, "### Register Dump Status Difference")?;
            writeln!(f)?;
            let updated_status = status
                .replace(
                    &format!("Present in {}, Absent in {}", sim1_name, sim2_name),
                    &format!("Present in {}, Absent in {}", sim1_name, sim2_name),
                )
                .replace(
                    &format!("Absent in {}, Present in {}", sim1_name, sim2_name),
                    &format!("Absent in {}, Present in {}", sim1_name, sim2_name),
                );
            writeln!(f, "Status: {}", updated_status)?;
            writeln!(f)?;
        }

        if let Some(reg_diff) = &self.register_dump_diff {
            if !reg_diff.is_empty() {
                writeln!(f, "### Register Dump Content Differences")?;
                writeln!(f)?;
                writeln!(f, "{}", reg_diff)?;
                writeln!(f)?;
            }
        }

        if let Some(ex_diff) = &self.exceptions_diff {
            if !ex_diff.is_empty() {
                writeln!(f, "### Exception Difference Details")?;
                writeln!(f)?;
                writeln!(f, "{}", ex_diff)?;
                writeln!(f)?;
            }
        }

        if let Some(stats_diff) = &self.conversion_stats_diff {
            if !stats_diff.is_empty() {
                writeln!(f, "### Conversion Statistics Difference Details")?;
                writeln!(f)?;
                writeln!(f, "{}", stats_diff)?;
                writeln!(f)?;
            }
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "Standard diff report generated at: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

fn compare_conversion_stats(
    stats1: &ConversionStats,
    stats2: &ConversionStats,
    sim1_type: EmulatorType,
    sim2_type: EmulatorType,
) -> ConversionStatsDiff {
    let mut diff = ConversionStatsDiff {
        sim1_emulator_type: sim1_type,
        sim2_emulator_type: sim2_type,
        original_exception_count_changed: None,
        original_register_count_changed: None,
        conversion_successful_changed: None,
        warnings_changed: None,
    };

    if stats1.original_exception_count != stats2.original_exception_count {
        diff.original_exception_count_changed = Some((
            stats1.original_exception_count,
            stats2.original_exception_count,
        ));
    }
    if stats1.original_register_count != stats2.original_register_count {
        diff.original_register_count_changed = Some((
            stats1.original_register_count,
            stats2.original_register_count,
        ));
    }
    if stats1.conversion_successful != stats2.conversion_successful {
        diff.conversion_successful_changed =
            Some((stats1.conversion_successful, stats2.conversion_successful));
    }
    if stats1.warnings != stats2.warnings {
        diff.warnings_changed = Some((stats1.warnings.clone(), stats2.warnings.clone()));
    }
    diff
}

pub fn compare_standard_execution_outputs(
    output1: &StandardExecutionOutput,
    output2: &StandardExecutionOutput,
) -> StandardExecutionOutputDiff {
    let mut diff = StandardExecutionOutputDiff {
        sim1_emulator_type: output1.emulator_type,
        sim2_emulator_type: output2.emulator_type,
        exceptions_diff: None,
        register_dump_status: None,
        register_dump_diff: None,
        conversion_stats_diff: None,
    };

    let ex_list_diff = compare_exception_dump_lists(
        &output1.exceptions,
        &output2.exceptions,
        output1.emulator_type,
        output2.emulator_type,
    );
    if !ex_list_diff.is_empty() {
        diff.exceptions_diff = Some(ex_list_diff);
    }

    match (&output1.register_dump, &output2.register_dump) {
        (Some(rd1), Some(rd2)) => {
            let reg_d_diff =
                compare_registers_dumps(rd1, rd2, output1.emulator_type, output2.emulator_type);
            if !reg_d_diff.is_empty() {
                diff.register_dump_diff = Some(reg_d_diff);
            }
        }
        (Some(_), None) => {
            diff.register_dump_status = Some(format!(
                "Present in {}, Absent in {}",
                output1.emulator_type, output2.emulator_type
            ));
        }
        (None, Some(_)) => {
            diff.register_dump_status = Some(format!(
                "Absent in {}, Present in {}",
                output1.emulator_type, output2.emulator_type
            ));
        }
        (None, None) => {}
    }

    let stats_d = compare_conversion_stats(
        &output1.conversion_stats,
        &output2.conversion_stats,
        output1.emulator_type,
        output2.emulator_type,
    );
    if !stats_d.is_empty() {
        diff.conversion_stats_diff = Some(stats_d);
    }

    diff
}
