use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 指令信息结构体
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InstructionTrace {
    /// 程序计数器地址
    pub pc: u64,
    /// ELF dump中的反汇编指令文本（汇编后）
    pub disassembly: String,
    /// 机器码
    pub machine_code: String,
    /// 原始指令文本
    pub original_instruction: String,
}

/// 一个持有已解析的ELF dump以进行高效PC查找的跟踪器。
pub struct ElfTracer {
    instructions: HashMap<u64, (String, String, String)>,
}

impl ElfTracer {
    /// 通过加载和解析ELF dump文件来创建一个新的ElfTracer。
    pub fn new<P: AsRef<Path>>(elf_dump_path: P) -> std::io::Result<Self> {
        let path = elf_dump_path.as_ref();
        debug!("Loading and parsing ELF dump file: {}", path.display());
        let dump_content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read ELF dump file {}: {}", path.display(), e);
                return Err(e);
            }
        };

        let lines: Vec<&str> = dump_content.lines().collect();
        let mut instructions = HashMap::new();

        for (i, line) in lines.iter().enumerate() {
            if let Some((pc, instruction_text, machine_code)) = parse_elf_instruction_line(line) {
                // 检查前一行是否是原始指令（双行格式）
                let original_instruction = if i > 0 {
                    let prev_line = lines[i - 1].trim();
                    // 如果前一行不包含冒号且不为空，则是原始指令
                    if !prev_line.is_empty() && !prev_line.contains(':') {
                        prev_line.to_string()
                    } else {
                        instruction_text.clone()
                    }
                } else {
                    instruction_text.clone()
                };
                instructions.insert(pc, (instruction_text, machine_code, original_instruction));
            }
        }

        debug!(
            "✓ Parsed {} instructions from ELF dump {}",
            instructions.len(),
            path.display()
        );

        Ok(Self { instructions })
    }

    /// 将单个程序计数器追溯到其源指令。
    pub fn trace_pc(&self, pc: u64) -> Option<InstructionTrace> {
        self.instructions
            .get(&pc)
            .map(|(disassembly, machine_code, original_instruction)| {
                debug!(
                    "✓ Found PC 0x{:X} in cached ELF dump: {} (machine code: {})",
                    pc, disassembly, machine_code
                );
                debug!("  Original instruction: {}", original_instruction);
                InstructionTrace {
                    pc,
                    disassembly: disassembly.clone(),
                    machine_code: machine_code.clone(),
                    original_instruction: original_instruction.clone(),
                }
            })
    }

    /// 高效地追溯多个PC。
    pub fn trace_multiple_pcs(&self, pcs: &[u64]) -> Vec<Option<InstructionTrace>> {
        let results = pcs.iter().map(|&pc| self.trace_pc(pc)).collect();
        debug!("✅ Batch PC trace completed for {} PCs", pcs.len());
        results
    }
}

/// 解析ELF指令行，提取PC、反汇编文本和机器码
fn parse_elf_instruction_line(line: &str) -> Option<(u64, String, String)> {
    let trimmed = line.trim();

    // 查找冒号分隔符
    if let Some(colon_pos) = trimmed.find(':') {
        let addr_str = trimmed[..colon_pos].trim();

        // 验证地址
        if let Ok(pc) = u64::from_str_radix(addr_str, 16) {
            let after_colon = trimmed[colon_pos + 1..].trim();

            // 分割机器码和指令部分
            let parts: Vec<&str> = after_colon.split_whitespace().collect();
            if parts.len() >= 2 {
                let machine_code = parts[0];
                let instruction = parts[1..].join(" ");
                return Some((pc, instruction, machine_code.to_string()));
            }
        }
    }
    None
}


