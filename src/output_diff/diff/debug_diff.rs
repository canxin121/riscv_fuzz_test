use crate::emulators::EmulatorType;
use crate::output_diff::diff::{RegistersDumpDiff, compare_registers_dumps};
use crate::output_parser::debug::DebugExecutionOutput;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugExecutionOutputDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub register_dumps_count_changed: Option<(usize, usize)>,
    pub differing_register_dumps: Vec<(usize, RegistersDumpDiff)>,
    pub total_dumps_changed: Option<(usize, usize)>,
}

impl DebugExecutionOutputDiff {
    pub fn is_empty(&self) -> bool {
        self.register_dumps_count_changed.is_none()
            && self.differing_register_dumps.is_empty()
            && self.total_dumps_changed.is_none()
    }
}

impl fmt::Display for DebugExecutionOutputDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sim1_name = self.sim1_emulator_type.to_string();
        let sim2_name = self.sim2_emulator_type.to_string();

        writeln!(f, "# Debug Execution Output Diff Report")?;
        writeln!(f)?;
        writeln!(f, "Comparison: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "## Diff Result")?;
            writeln!(f)?;
            writeln!(
                f,
                "No significant differences found - debug outputs from both simulators match exactly!"
            )?;
            writeln!(f)?;
            return Ok(());
        }

        writeln!(f, "## Differences Detected")?;
        writeln!(f)?;

        // Diff Summary Table
        let mut diff_count = 0;
        writeln!(f, "| Diff Type | Count |")?;
        writeln!(f, "|----------|------|")?;

        if let Some((count1, count2)) = self.register_dumps_count_changed {
            diff_count += 1;
            writeln!(
                f,
                "| Valid Register Dumps | {}: {}, {}: {} |",
                sim1_name, count1, sim2_name, count2
            )?;
        }

        if !self.differing_register_dumps.is_empty() {
            diff_count += 1;
            writeln!(
                f,
                "| Register Content | {} dumps have content differences |",
                self.differing_register_dumps.len()
            )?;
        }

        if let Some((total1, total2)) = self.total_dumps_changed {
            diff_count += 1;
            writeln!(
                f,
                "| Total Dump Markers | {}: {}, {}: {} |",
                sim1_name, total1, sim2_name, total2
            )?;
        }

        if diff_count == 0 {
            writeln!(f, "| - | No Differences |")?;
        }
        writeln!(f)?;

        // Detailed Diff Information
        writeln!(f, "## Detailed Diff Analysis")?;
        writeln!(f)?;

        if let Some((count1, count2)) = self.register_dumps_count_changed {
            writeln!(f, "### Valid Register Dump Count Difference")?;
            writeln!(f)?;
            writeln!(f, "{}: {}", sim1_name, count1)?;
            writeln!(f, "{}: {}", sim2_name, count2)?;
            writeln!(f)?;

            if let Some((total1, total2)) = self.total_dumps_changed {
                let efficiency1 = if total1 > 0 {
                    (count1 as f64 / total1 as f64) * 100.0
                } else {
                    0.0
                };
                let efficiency2 = if total2 > 0 {
                    (count2 as f64 / total2 as f64) * 100.0
                } else {
                    0.0
                };
                writeln!(f, "#### Dump Efficiency Comparison")?;
                writeln!(f)?;
                writeln!(
                    f,
                    "| Simulator | Valid Dumps | Total Markers | Efficiency |"
                )?;
                writeln!(f, "|--------|----------|--------|------|")?;
                writeln!(
                    f,
                    "| {} | {} | {} | {:.1}% |",
                    sim1_name, count1, total1, efficiency1
                )?;
                writeln!(
                    f,
                    "| {} | {} | {} | {:.1}% |",
                    sim2_name, count2, total2, efficiency2
                )?;
                writeln!(f)?;
            }
        }

        if !self.differing_register_dumps.is_empty() {
            writeln!(f, "### Register Content Differences")?;
            writeln!(f)?;
            writeln!(
                f,
                "Found {} dumps with differences:",
                self.differing_register_dumps.len()
            )?;
            writeln!(f)?;

            for (i, (index, reg_diff)) in self.differing_register_dumps.iter().enumerate() {
                writeln!(f, "#### Dump Index {} (#{} in sequence)", index, i + 1)?;
                writeln!(f)?;
                // Assuming RegistersDumpDiff::fmt is cleaned
                writeln!(f, "{}", reg_diff)?;
                writeln!(f)?;
            }
        }

        if let Some((total1, total2)) = self.total_dumps_changed {
            writeln!(f, "### Total Dump Marker Count Difference")?;
            writeln!(f)?;
            writeln!(f, "{}: {}", sim1_name, total1)?;
            writeln!(f, "{}: {}", sim2_name, total2)?;
            writeln!(f, "Difference: {}", (total2 as i64 - total1 as i64).abs())?;
            writeln!(f)?;
        }

        Ok(())
    }
}

pub fn compare_debug_execution_outputs(
    output1: &DebugExecutionOutput,
    output2: &DebugExecutionOutput,
) -> DebugExecutionOutputDiff {
    let mut diff = DebugExecutionOutputDiff {
        sim1_emulator_type: output1.emulator_type,
        sim2_emulator_type: output2.emulator_type,
        register_dumps_count_changed: None,
        differing_register_dumps: Vec::new(),
        total_dumps_changed: None,
    };

    if output1.total_dumps != output2.total_dumps {
        diff.total_dumps_changed = Some((output1.total_dumps, output2.total_dumps));
    }

    if output1.register_dumps.len() != output2.register_dumps.len() {
        diff.register_dumps_count_changed =
            Some((output1.register_dumps.len(), output2.register_dumps.len()));
    } else {
        for (i, (rd1, rd2)) in output1
            .register_dumps
            .iter()
            .zip(output2.register_dumps.iter())
            .enumerate()
        {
            let reg_dump_diff =
                compare_registers_dumps(rd1, rd2, output1.emulator_type, output2.emulator_type);
            if !reg_dump_diff.is_empty() {
                diff.differing_register_dumps.push((i, reg_dump_diff));
            }
        }
    }
    diff
}
