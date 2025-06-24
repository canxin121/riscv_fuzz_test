use crate::utils::get_regs_in_inst;
use std::collections::HashSet;

pub fn extract_minimal_instructions_for_regs(
    insts: Vec<String>,
    target_regs: Vec<String>,
) -> Vec<String> {
    if insts.is_empty() || target_regs.is_empty() {
        return Vec::new();
    }

    // 初始化感兴趣的寄存器集合
    let mut interested_regs: HashSet<String> = target_regs.into_iter().collect();
    let mut result_instructions = Vec::new();

    // 从后往前遍历指令流
    for inst in insts.iter().rev() {
        // 获取当前指令中涉及的所有寄存器
        let regs_in_inst = get_regs_in_inst(inst);

        // 检查当前指令是否涉及感兴趣的寄存器
        let has_interested_reg = regs_in_inst.iter().any(|reg| interested_regs.contains(reg));

        if has_interested_reg {
            // 保留这条指令
            result_instructions.push(inst.clone());

            // 将这条指令中的所有寄存器加入感兴趣集合
            // 因为它们可能影响我们关心的寄存器
            for reg in regs_in_inst {
                interested_regs.insert(reg);
            }
        }
    }

    // 反转结果以恢复原始执行顺序
    result_instructions.reverse();
    result_instructions
}
