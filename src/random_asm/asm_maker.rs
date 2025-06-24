use crate::{
    elf::template::generate_standard_asm,
    random_asm::inst_generator::{
        GenerationOrder, InstructionsGenerator, remove_special_instructions,
    },
};
use rand::prelude::*;
use riscv_instruction::separated_instructions::{RV64Extensions, RiscvInstruction};
use std::collections::HashMap;

/// 生成随机指令
pub fn generate_instructions(
    instruction_counts: &HashMap<RV64Extensions, usize>,
    generation_order: GenerationOrder,
    rng: &mut ThreadRng,
) -> Vec<RiscvInstruction> {
    let mut generator = InstructionsGenerator::new_rv64();

    // 设置每个扩展的指令数量
    for (&extension, &count) in instruction_counts {
        generator = generator.with(extension, count);
    }

    // 设置生成顺序
    generator = generator.order(generation_order);

    // 生成指令
    let mut instructions = generator.generate_with_rng(rng);

    // 过滤掉可能导致控制流跳转的指令
    instructions = remove_special_instructions(instructions);

    instructions
}

/// 将指令列表格式化为汇编代码字符串
fn format_instructions(insts: &[RiscvInstruction]) -> String {
    insts
        .iter()
        .map(|inst| format!("    {}", inst))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 根据指令列表生成标准RISC-V汇编模板
pub fn generate_standard_asm_from_insts(insts: &[RiscvInstruction]) -> String {
    let user_code = format_instructions(insts);
    generate_standard_asm(&user_code)
}
