use crate::emulators::EmulatorType;
use crate::output_diff::diff::{
    ExceptionListDiff, RegistersDumpDiff, compare_exception_dump_lists, compare_registers_dumps,
};
use crate::output_parser::common::CommonExecutionOutput;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonExecutionOutputDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub output_items_status: Option<String>,
    pub register_dumps_count_changed: Option<(usize, usize)>,
    pub differing_register_dumps: Vec<(usize, RegistersDumpDiff)>,
    pub exception_dumps_diff: Option<ExceptionListDiff>,
}

impl CommonExecutionOutputDiff {
    pub fn is_empty(&self) -> bool {
        self.output_items_status.is_none()
            && self.register_dumps_count_changed.is_none()
            && self.differing_register_dumps.is_empty()
            && self
                .exception_dumps_diff
                .as_ref()
                .map_or(true, |e| e.is_empty())
    }
}

impl fmt::Display for CommonExecutionOutputDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sim1_name = self.sim1_emulator_type.to_string();
        let sim2_name = self.sim2_emulator_type.to_string();

        writeln!(f, "# Common Execution Output Diff Report")?;
        writeln!(f)?;
        writeln!(f, "Comparison: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "## Diff Result")?;
            writeln!(f)?;
            writeln!(
                f,
                "No significant differences found - outputs from both simulators match exactly!"
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

        if self.register_dumps_count_changed.is_some() {
            diff_count += 1;
            writeln!(f, "| Register Dump Count | Count Mismatch |")?;
        }

        if !self.differing_register_dumps.is_empty() {
            diff_count += 1;
            writeln!(
                f,
                "| Register Content | {} register dumps have content differences |",
                self.differing_register_dumps.len()
            )?;
        }

        if self.output_items_status.is_some() {
            diff_count += 1;
            writeln!(f, "| Output Item Status | Item count or content differs |")?;
        }

        if self.exception_dumps_diff.is_some() {
            diff_count += 1;
            writeln!(f, "| Exception Dumps | Exception information differs |")?;
        }

        if diff_count == 0 {
            writeln!(f, "| - | No Differences |")?;
        }
        writeln!(f)?;

        // Detailed Diff Information
        writeln!(f, "## Detailed Diff Analysis")?;
        writeln!(f)?;

        if let Some((count1, count2)) = self.register_dumps_count_changed {
            writeln!(f, "### Register Dump Count Difference")?;
            writeln!(f)?;
            writeln!(f, "{}: {}", sim1_name, count1)?;
            writeln!(f, "{}: {}", sim2_name, count2)?;
            writeln!(f)?;
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

        if let Some(status) = &self.output_items_status {
            writeln!(f, "### Output Item Status Difference")?;
            writeln!(f)?;
            writeln!(f, "Status: {}", status)?;
            writeln!(f)?;
        }

        if let Some(ex_diff) = &self.exception_dumps_diff {
            if !ex_diff.is_empty() {
                writeln!(f, "### Exception Dump Differences")?;
                writeln!(f)?;
                // Assuming ExceptionListDiff::fmt is cleaned
                writeln!(f, "{}", ex_diff)?;
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

pub fn compare_execution_outputs(
    output1: &CommonExecutionOutput,
    output2: &CommonExecutionOutput,
) -> CommonExecutionOutputDiff {
    let mut diff = CommonExecutionOutputDiff {
        sim1_emulator_type: output1.emulator_type,
        sim2_emulator_type: output2.emulator_type,
        output_items_status: None,
        register_dumps_count_changed: None,
        differing_register_dumps: Vec::new(),
        exception_dumps_diff: None,
    };

    if output1.output_items.len() != output2.output_items.len() {
        diff.output_items_status = Some(format!(
            "{} has {} items, {} has {} items",
            output1.emulator_type,
            output1.output_items.len(),
            output2.emulator_type,
            output2.output_items.len()
        ));
    } else {
        let mut items_differ = false;
        for (item1, item2) in output1.output_items.iter().zip(output2.output_items.iter()) {
            let item1_json = serde_json::to_string(item1).unwrap_or_default();
            let item2_json = serde_json::to_string(item2).unwrap_or_default();
            if item1_json != item2_json {
                items_differ = true;
                break;
            }
        }
        if items_differ {
            diff.output_items_status = Some(format!(
                "Output items differ between {} and {}",
                output1.emulator_type, output2.emulator_type
            ));
        }
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

    let ex_list_diff = compare_exception_dump_lists(
        &output1.exception_dumps,
        &output2.exception_dumps,
        output1.emulator_type,
        output2.emulator_type,
    );
    if !ex_list_diff.is_empty() {
        diff.exception_dumps_diff = Some(ex_list_diff);
    }

    diff
}
