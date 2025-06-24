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
        writeln!(f, "# 通用执行输出差异变化报告")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "通用执行输出差异无变化")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "比较对象: {} ⚡ {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        // 变化汇总表格
        writeln!(f, "## 变化汇总")?;
        writeln!(f)?;
        writeln!(f, "| 变化项目 | 状态 | 详情 |")?;
        writeln!(f, "|:---------|:----:|:-----|")?;

        let mut change_count = 0;

        if let Some(ch) = &self.register_dumps_count_changed_diff {
            change_count += 1;
            let detail = match (&ch.old, &ch.new) {
                (Some((old1, old2)), Some((new1, new2))) => {
                    format!("{}→{} vs {}→{}", old1, new1, old2, new2)
                }
                _ => format!("{:?} → {:?}", ch.old, ch.new)
            };
            writeln!(f, "| 寄存器转储数量 | 🔄 变化 | {} |", detail)?;
        }

        if let Some(ch) = &self.differing_register_dumps_changed {
            change_count += 1;
            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => "📈 增加",
                (old, new) if new < old => "减少",
                _ => "不变",
            };
            writeln!(f, "| 寄存器内容差异 | {} | {}→{} 个差异转储 |", 
                trend, ch.old.len(), ch.new.len())?;
        }

        if self.exception_dumps_diff_presence_changed.is_some() {
            change_count += 1;
            let ch = self.exception_dumps_diff_presence_changed.as_ref().unwrap();
            let status = match (ch.old, ch.new) {
                (false, true) => "✅ 新增异常差异",
                (true, false) => "❌ 消除异常差异", 
                _ => "🔄 异常差异状态变化",
            };
            writeln!(f, "| 异常转储差异 | {} | 存在状态变化 |", status)?;
        }

        if let Some(ch) = &self.output_items_status_diff {
            change_count += 1;
            writeln!(f, "| 输出项状态 | 🔄 变化 | {:?} → {:?} |", ch.old, ch.new)?;
        }

        if change_count == 0 {
            writeln!(f, "| - | ✅ 无变化 | 所有项目保持一致 |")?;
        }
        writeln!(f)?;

        // 详细变化分析
        writeln!(f, "## 详细变化分析")?;
        writeln!(f)?;

        if let Some(ch) = &self.register_dumps_count_changed_diff {
            writeln!(f, "### 寄存器转储数量差异变化")?;
            writeln!(f)?;
            writeln!(f, "| 时期 | {} 转储数 | {} 转储数 | 差异量 | 差异率 |", sim1_name, sim2_name)?;
            writeln!(f, "|:-----|:----------:|:----------:|:------:|:------:|")?;
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
                    writeln!(f, "| 变化前 | {} | {} | {} | {:.1}% |", old1, old2, old_diff, old_rate)?;
                    writeln!(f, "| 变化后 | {} | {} | {} | {:.1}% |", new1, new2, new_diff, new_rate)?;
                }
                _ => {
                    writeln!(f, "| 变化前 | {:?} | - | - | - |", ch.old)?;
                    writeln!(f, "| 变化后 | {:?} | - | - | - |", ch.new)?;
                }
            }
            writeln!(f)?;
        }

        if let Some(ch) = &self.differing_register_dumps_changed {
            writeln!(f, "### 寄存器内容差异变化")?;
            writeln!(f)?;
            writeln!(f, "| 指标 | 变化前 | 变化后 | 净变化 | 影响评估 |")?;
            writeln!(f, "|:-----|:------:|:------:|:------:|:---------|")?;

            let net_change = ch.new.len() as i64 - ch.old.len() as i64;
            let impact = match net_change {
                x if x > 5 => "⚠️ 显著增加",
                x if x > 0 => "📈 轻微增加", 
                0 => "✅ 保持稳定",
                x if x > -5 => "轻微减少",
                _ => "✅ 显著改善",
            };

            writeln!(f, "| 差异转储数量 | {} | {} | {:+} | {} |", 
                ch.old.len(), ch.new.len(), net_change, impact)?;

            let consistency = if ch.old.is_empty() && ch.new.is_empty() {
                "🎯 完全一致"
            } else if ch.old.is_empty() {
                "⚠️ 新增差异"
            } else if ch.new.is_empty() {
                "✅ 完全修复"
            } else {
                "🔄 部分差异"
            };

            writeln!(f, "| 一致性状态 | {} | {} | - | {} |", 
                if ch.old.is_empty() { "一致" } else { "有差异" },
                if ch.new.is_empty() { "一致" } else { "有差异" },
                consistency)?;
            writeln!(f)?;
        }

        if let Some(content_diff) = &self.exception_dumps_diff_content_diff {
            if !content_diff.is_empty() {
                writeln!(f, "### 异常转储差异内容变化")?;
                writeln!(f, "{}", content_diff)?;
            }
        }

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

        writeln!(f, "---")?;
        writeln!(
            f,
            "通用输出差异变化报告生成时间: {}",
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
