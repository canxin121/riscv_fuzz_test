pub mod common_diff_diff;
pub mod debug_diff_diff;
pub mod standard_diff_diff;

use serde::{Deserialize, Serialize};
use std::fmt;

// 引入必要的 Diff 类型
use crate::output_diff::diff::common_diff::CommonExecutionOutputDiff;
use crate::output_diff::diff::debug_diff::DebugExecutionOutputDiff;
use crate::output_diff::diff::standard_diff::StandardExecutionOutputDiff;

// Helper struct to represent a change from an old value to a new value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change<T> {
    pub old: T,
    pub new: T,
}

impl<T: fmt::Debug> fmt::Display for Change<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {:?}", self.old, self.new)
    }
}

// Trait for diff types that can be diff_diffed
pub trait DiffDiffable {
    type DiffDiffOutput;
    fn diff_diff(&self, other: &Self) -> Self::DiffDiffOutput;
}

impl DiffDiffable for StandardExecutionOutputDiff {
    type DiffDiffOutput = standard_diff_diff::StandardExecutionOutputDiffDiff;
    fn diff_diff(&self, other: &Self) -> Self::DiffDiffOutput {
        standard_diff_diff::compare_standard_execution_output_diffs(self, other)
    }
}

impl DiffDiffable for DebugExecutionOutputDiff {
    type DiffDiffOutput = debug_diff_diff::DebugExecutionOutputDiffDiff;
    fn diff_diff(&self, other: &Self) -> Self::DiffDiffOutput {
        debug_diff_diff::compare_debug_execution_output_diffs(self, other)
    }
}

impl DiffDiffable for CommonExecutionOutputDiff {
    type DiffDiffOutput = common_diff_diff::CommonExecutionOutputDiffDiff;
    fn diff_diff(&self, other: &Self) -> Self::DiffDiffOutput {
        common_diff_diff::compare_common_execution_output_diffs(self, other)
    }
}

/// Generic function to compare two diffs.
pub fn compare_output_diffs<T: DiffDiffable>(diff1: &T, diff2: &T) -> T::DiffDiffOutput {
    diff1.diff_diff(diff2)
}

// Re-export main comparison functions and DiffDiff structs
pub use common_diff_diff::{CommonExecutionOutputDiffDiff, compare_common_execution_output_diffs};
pub use debug_diff_diff::{DebugExecutionOutputDiffDiff, compare_debug_execution_output_diffs};
pub use standard_diff_diff::{
    ConversionStatsDiffDiff, ExceptionListDiffDiff, RegistersDumpDiffDiff,
    StandardExecutionOutputDiffDiff, compare_conversion_stats_diffs, compare_exception_list_diffs,
    compare_registers_dump_diffs, compare_standard_execution_output_diffs,
};
