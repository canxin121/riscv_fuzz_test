use crate::emulators::EmulatorType;
use crate::output_diff::diff::ExceptionDiffCategory;
use crate::output_diff::diff::ExceptionListDiff;

/// 检查 ExceptionListDiff 是否包含仅在 Rocket 模拟器中出现的非法指令 (mcause=2)。
pub fn has_rocket_only_illegal_instructions(diff: &ExceptionListDiff) -> bool {
    diff.categorized_summary.iter().any(|cat_diff| {
        matches!(
            cat_diff.category,
            ExceptionDiffCategory::OnlyInSimulator {
                simulator: EmulatorType::Rocket,
                mcause: 2,
            }
        )
    })
}

/// 获取所有仅在 Rocket 模拟器中出现的非法指令 (mcause=2) 的原始指令字符串。
pub fn get_rocket_illegal_instruction_originals(diff: &ExceptionListDiff) -> Vec<String> {
    let mut originals = Vec::new();
    for cat_diff in &diff.categorized_summary {
        if matches!(
            cat_diff.category,
            ExceptionDiffCategory::OnlyInSimulator {
                simulator: EmulatorType::Rocket,
                mcause: 2,
            }
        ) {
            for trace_opt in &cat_diff.pc_instruction_traces {
                if let Some(trace) = trace_opt {
                    originals.push(trace.original_instruction.clone());
                }
            }
        }
    }

    originals.sort_unstable();
    originals.dedup();
    originals
}
