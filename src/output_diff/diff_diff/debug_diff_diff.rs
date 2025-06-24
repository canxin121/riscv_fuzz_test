use crate::emulators::EmulatorType;
use crate::output_diff::diff::RegistersDumpDiff;
use crate::output_diff::diff::debug_diff::DebugExecutionOutputDiff;
use crate::output_diff::diff_diff::Change;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DebugExecutionOutputDiffDiff {
    pub sim1_emulator_type: EmulatorType,
    pub sim2_emulator_type: EmulatorType,
    pub register_dumps_count_changed_diff: Option<Change<Option<(usize, usize)>>>,
    pub differing_register_dumps_changed: Option<Change<Vec<(usize, RegistersDumpDiff)>>>,
    pub total_dumps_changed_diff: Option<Change<Option<(usize, usize)>>>,
}

impl Default for DebugExecutionOutputDiffDiff {
    fn default() -> Self {
        Self {
            sim1_emulator_type: EmulatorType::Spike,
            sim2_emulator_type: EmulatorType::Rocket,
            register_dumps_count_changed_diff: None,
            differing_register_dumps_changed: None,
            total_dumps_changed_diff: None,
        }
    }
}

impl DebugExecutionOutputDiffDiff {
    pub fn is_empty(&self) -> bool {
        self.register_dumps_count_changed_diff.is_none()
            && self.differing_register_dumps_changed.is_none()
            && self.total_dumps_changed_diff.is_none()
    }

    fn get_sim1_name(&self) -> String {
        self.sim1_emulator_type.to_string()
    }

    fn get_sim2_name(&self) -> String {
        self.sim2_emulator_type.to_string()
    }
}

impl fmt::Display for DebugExecutionOutputDiffDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# 调试执行输出差异变化报告")?;
        writeln!(f)?;

        if self.is_empty() {
            writeln!(f, "调试执行输出差异无变化")?;
            writeln!(f)?;
            return Ok(());
        }

        let sim1_name = self.get_sim1_name();
        let sim2_name = self.get_sim2_name();

        writeln!(f, "比较对象: {} vs {}", sim1_name, sim2_name)?;
        writeln!(f)?;

        writeln!(f, "## 变化汇总")?;
        writeln!(f)?;
        writeln!(f, "| 变化项目 | 数量 |")?;
        writeln!(f, "|----------|------|")?;

        let mut change_count = 0;

        if self.register_dumps_count_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| 有效寄存器转储数 | 变化 |")?;
        }

        if self.differing_register_dumps_changed.is_some() {
            change_count += 1;
            writeln!(f, "| 寄存器内容差异 | 变化 |")?;
        }

        if self.total_dumps_changed_diff.is_some() {
            change_count += 1;
            writeln!(f, "| 总转储标记数 | 变化 |")?;
        }

        if change_count == 0 {
            writeln!(f, "| - | 无变化 |")?;
        }
        writeln!(f)?;

        writeln!(f, "## 详细变化分析")?;
        writeln!(f)?;

        if let Some(ch) = &self.register_dumps_count_changed_diff {
            writeln!(f, "### 有效寄存器转储数差异变化")?;
            writeln!(f)?;
            writeln!(
                f,
                "| 时期 | {} | {} | 总差异 | 效率变化 |",
                sim1_name, sim2_name
            )?;
            writeln!(f, "|------|------------|------------|--------|----------|")?;

            match (&ch.old, &ch.new) {
                (Some((old_count1, old_count2)), Some((new_count1, new_count2))) => {
                    // 如果有总转储数据，计算效率
                    let efficiency_info = if let Some(total_ch) = &self.total_dumps_changed_diff {
                        match (&total_ch.old, &total_ch.new) {
                            (Some((old_total1, old_total2)), Some((new_total1, new_total2))) => {
                                let old_eff1 = if *old_total1 > 0 {
                                    (*old_count1 as f64 / *old_total1 as f64) * 100.0
                                } else {
                                    0.0
                                };
                                let old_eff2 = if *old_total2 > 0 {
                                    (*old_count2 as f64 / *old_total2 as f64) * 100.0
                                } else {
                                    0.0
                                };
                                let new_eff1 = if *new_total1 > 0 {
                                    (*new_count1 as f64 / *new_total1 as f64) * 100.0
                                } else {
                                    0.0
                                };
                                let new_eff2 = if *new_total2 > 0 {
                                    (*new_count2 as f64 / *new_total2 as f64) * 100.0
                                } else {
                                    0.0
                                };
                                format!(
                                    "{:.1}%→{:.1}% / {:.1}%→{:.1}%",
                                    old_eff1, new_eff1, old_eff2, new_eff2
                                )
                            }
                            _ => "无法计算".to_string(),
                        }
                    } else {
                        "无总数数据".to_string()
                    };

                    writeln!(
                        f,
                        "| 变化前 | {} 个 | {} 个 | {} 个 | {} |",
                        old_count1,
                        old_count2,
                        (*old_count2 as i64 - *old_count1 as i64).abs(),
                        efficiency_info.split('/').next().unwrap_or("N/A")
                    )?;
                    writeln!(
                        f,
                        "| 变化后 | {} 个 | {} 个 | {} 个 | {} |",
                        new_count1,
                        new_count2,
                        (*new_count2 as i64 - *new_count1 as i64).abs(),
                        efficiency_info.split('/').nth(1).unwrap_or("N/A")
                    )?;
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

            writeln!(f, "| 指标 | 变化前 | 变化后 | 净变化 | 变化趋势 |")?;
            writeln!(f, "|------|--------|--------|--------|----------|")?;

            let trend = match (ch.old.len(), ch.new.len()) {
                (old, new) if new > old => format!("增加 {} 个差异", new - old),
                (old, new) if new < old => format!("减少 {} 个差异", old - new),
                _ => "保持不变".to_string(),
            };

            writeln!(
                f,
                "| 差异转储数量 | {} 个 | {} 个 | {:+} | {} |",
                ch.old.len(),
                ch.new.len(),
                ch.new.len() as i64 - ch.old.len() as i64,
                trend
            )?;

            let status_old = if ch.old.is_empty() {
                "无差异"
            } else {
                "存在差异"
            };
            let status_new = if ch.new.is_empty() {
                "无差异"
            } else {
                "存在差异"
            };

            writeln!(
                f,
                "| 差异状态 | {} | {} | - | {} |",
                status_old,
                status_new,
                if ch.old.is_empty() && !ch.new.is_empty() {
                    "新增差异"
                } else if !ch.old.is_empty() && ch.new.is_empty() {
                    "消除差异"
                } else {
                    "状态延续"
                }
            )?;
            writeln!(f)?;

            if !ch.old.is_empty() || !ch.new.is_empty() {
                writeln!(f, "#### 差异转储索引对比")?;
                writeln!(f)?;
                writeln!(f, "| 时期 | 转储索引列表 |")?;
                writeln!(f, "|------|--------------|")?;

                if !ch.old.is_empty() {
                    let old_indices: Vec<String> =
                        ch.old.iter().map(|(idx, _)| idx.to_string()).collect();
                    writeln!(f, "| 变化前 | {} |", old_indices.join(", "))?;
                } else {
                    writeln!(f, "| 变化前 | 无差异转储 |")?;
                }

                if !ch.new.is_empty() {
                    let new_indices: Vec<String> =
                        ch.new.iter().map(|(idx, _)| idx.to_string()).collect();
                    writeln!(f, "| 变化后 | {} |", new_indices.join(", "))?;
                } else {
                    writeln!(f, "| 变化后 | 无差异转储 |")?;
                }
                writeln!(f)?;
            }
        }

        writeln!(f, "---")?;
        writeln!(
            f,
            "调试差异变化报告生成时间: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )?;

        Ok(())
    }
}

pub fn compare_debug_execution_output_diffs(
    diff1: &DebugExecutionOutputDiff,
    diff2: &DebugExecutionOutputDiff,
) -> DebugExecutionOutputDiffDiff {
    let mut ddiff = DebugExecutionOutputDiffDiff {
        sim1_emulator_type: diff1.sim1_emulator_type,
        sim2_emulator_type: diff1.sim2_emulator_type,
        ..Default::default()
    };

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

    if diff1.total_dumps_changed != diff2.total_dumps_changed {
        ddiff.total_dumps_changed_diff = Some(Change {
            old: diff1.total_dumps_changed,
            new: diff2.total_dumps_changed,
        });
    }

    ddiff
}
