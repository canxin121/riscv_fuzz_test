use rand::rng;
use rand::seq::SliceRandom;
use riscv_instruction::separated_instructions::*;
use serde::{Deserialize, Serialize};
// 这会导入:
// pub enum RV32Extensions
// {
//     B, C, D, F, H, I, M, Q, S, Sdext, Smdbltrp, Smrnmi, Svinval, V, Zaamo,
//     Zabha, Zacas, Zalasr, Zalrsc, Zawrs, Zba, Zbb, Zbc, Zbkb, Zbkx, Zbs, Zcb,
//     Zcd, Zcf, Zcmop, Zcmp, Zfbfmin, Zfh, Zicbom, Zicboz, Zicfilp, Zicfiss,
//     Zicond, Zicsr, Zifencei, Zilsd, Zimop, Zknd, Zkne, Zknh, Zks, Zvbb, Zvbc,
//     Zvfbfmin, Zvfbfwma, Zvkg, Zvkned, Zvknha, Zvks
// } #[doc = r" Available extensions for RV64 ISA base"]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum RV64Extensions
// {
//     B, C, D, F, H, I, M, Q, S, Sdext, Smdbltrp, Smrnmi, Svinval, V, Zaamo,
//     Zabha, Zacas, Zalasr, Zalrsc, Zawrs, Zba, Zbb, Zbc, Zbkb, Zbkx, Zbs, Zcb,
//     Zcd, Zcmop, Zcmp, Zfbfmin, Zfh, Zicbom, Zicboz, Zicfilp, Zicfiss, Zicond,
//     Zicsr, Zifencei, Zilsd, Zimop, Zkn, Zknd, Zkne, Zknh, Zks, Zvbb, Zvbc,
//     Zvfbfmin, Zvfbfwma, Zvkg, Zvkned, Zvknha, Zvks
// }
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IsaBase {
    RV32,
    RV64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GenerationOrder {
    Sequential,
    RandomShuffle,
}

#[derive(Debug, Clone)]
pub struct InstructionsGenerator<
    E: Copy + Eq + std::hash::Hash + Serialize + for<'a> Deserialize<'a>,
> {
    order: GenerationOrder,
    counts: HashMap<E, usize>,
}

impl<E: Copy + Eq + std::hash::Hash + Serialize + for<'a> Deserialize<'a>>
    InstructionsGenerator<E>
{
    pub fn with(mut self, ext: E, count: usize) -> Self {
        *self.counts.entry(ext).or_insert(0) += count;
        self
    }

    pub fn order(mut self, order: GenerationOrder) -> Self {
        self.order = order;
        self
    }
}

pub trait ExtensionRng: Copy + Eq + std::hash::Hash + Serialize + for<'a> Deserialize<'a> {
    fn random_instruction<R: rand::Rng>(&self, rng: &mut R) -> RiscvInstruction;
}

impl ExtensionRng for RV64Extensions {
    fn random_instruction<R: rand::Rng>(&self, rng: &mut R) -> RiscvInstruction {
        RV64Extensions::random_instruction(self, rng)
    }
}

impl ExtensionRng for RV32Extensions {
    fn random_instruction<R: rand::Rng>(&self, rng: &mut R) -> RiscvInstruction {
        RV32Extensions::random_instruction(self, rng)
    }
}

impl<E: ExtensionRng> InstructionsGenerator<E> {
    pub fn generate_with_rng<R: rand::Rng>(&self, rng: &mut R) -> Vec<RiscvInstruction> {
        let mut instructions = Vec::new();

        for (&ext, &count) in &self.counts {
            for _ in 0..count {
                let instr = ext.random_instruction(rng);
                instructions.push(instr);
            }
        }

        if self.order == GenerationOrder::RandomShuffle {
            instructions.shuffle(rng);
        }

        instructions
    }

    pub fn generate(&self) -> Vec<RiscvInstruction> {
        let mut rng = rng();
        self.generate_with_rng(&mut rng)
    }
}

impl InstructionsGenerator<RV64Extensions> {
    pub fn new_rv64() -> Self {
        Self {
            order: GenerationOrder::Sequential,
            counts: HashMap::new(),
        }
    }
}

impl InstructionsGenerator<RV32Extensions> {
    pub fn new_rv32() -> Self {
        Self {
            order: GenerationOrder::Sequential,
            counts: HashMap::new(),
        }
    }
}

pub fn remove_special_instructions(instructions: Vec<RiscvInstruction>) -> Vec<RiscvInstruction> {
    instructions
        .into_iter()
        .filter(|instruction| {
            match instruction {
                RiscvInstruction::RV32(rv32_instr) => {
                    match rv32_instr {
                        // RV32指令集按扩展分组，然后匹配特定指令
                        RV32Instruction::I(instr) => {
                            match instr {
                                RV32IInstructions::JAL(_)
                                | RV32IInstructions::JALR(_)
                                | RV32IInstructions::BEQ(_)
                                | RV32IInstructions::BNE(_)
                                | RV32IInstructions::BLT(_)
                                | RV32IInstructions::BGE(_)
                                | RV32IInstructions::BLTU(_)
                                | RV32IInstructions::BGEU(_)
                                | RV32IInstructions::ECALL(_)
                                | RV32IInstructions::EBREAK(_)
                                | RV32IInstructions::MRET(_)
                                // 加一个 WFI 指令，虽然它不直接跳转，但通常用于等待事件或中断
                                | RV32IInstructions::WFI(_) => false,
                                _ => true, // 保留 RV32I 中其他非跳转指令
                            }
                        }
                        RV32Instruction::C(instr) => {
                            match instr {
                                RV32CInstructions::C_J(_)
                                | RV32CInstructions::C_JAL(_)
                                | RV32CInstructions::C_JR(_)
                                | RV32CInstructions::C_JALR(_)
                                | RV32CInstructions::C_BEQZ(_)
                                | RV32CInstructions::C_BNEZ(_) => false,
                                _ => true, // 保留 RV32C 中其他非跳转指令
                            }
                        }
                        RV32Instruction::S(instr) => {
                            match instr {
                                RV32SInstructions::SRET(_) => false,
                                RV32SInstructions::SFENCE_VMA(_) => false, // 间接影响控制流
                            }
                        }
                        RV32Instruction::Sdext(instr) => match instr {
                            RV32SdextInstructions::DRET(_) => false,
                        },
                        RV32Instruction::Smrnmi(instr) => match instr {
                            RV32SmrnmiInstructions::MNRET(_) => false,
                        },
                        RV32Instruction::Zalrsc(instr) => {
                            // LR/SC 虽然不直接跳转，但通常与分支结合实现原子操作的重试
                            match instr {
                                RV32ZalrscInstructions::LR_W(_)
                                | RV32ZalrscInstructions::SC_W(_) => false,
                            }
                        }
                        RV32Instruction::V(instr) => {
                            // VSETVLI, VSETVL, VSETIVLI 改变向量配置，影响指令流
                            match instr {
                                RV32VInstructions::VSETVLI(_)
                                | RV32VInstructions::VSETVL(_)
                                | RV32VInstructions::VSETIVLI(_) => false,
                                _ => true, // 保留 RV32V 中其他指令
                            }
                        }
                        // 处理其他可能间接影响控制流的指令
                        RV32Instruction::Svinval(instr) => {
                            // Svinval 指令，用于TLB/缓存失效，间接影响指令获取
                            match instr {
                                RV32SvinvalInstructions::SFENCE_W_INVAL(_)
                                | RV32SvinvalInstructions::HINVAL_VVMA(_)
                                | RV32SvinvalInstructions::SFENCE_INVAL_IR(_)
                                | RV32SvinvalInstructions::HINVAL_GVMA(_)
                                | RV32SvinvalInstructions::SINVAL_VMA(_) => false,
                            }
                        }
                        RV32Instruction::H(instr) => {
                            // H 指令，用于管理虚拟内存，间接影响指令获取
                            match instr {
                                RV32HInstructions::HFENCE_GVMA(_)
                                | RV32HInstructions::HFENCE_VVMA(_) => false,
                                _ => true,
                            }
                        }
                        RV32Instruction::Zawrs(instr) => {
                            // Zawrs 指令，用于等待保留集，可能暂停执行
                            match instr {
                                RV32ZawrsInstructions::WRS_STO(_)
                                | RV32ZawrsInstructions::WRS_NTO(_) => false,
                            }
                        }
                        // 其他扩展（Zicond, Zicfilp, Zicbom, Zicboz, Zcmop, Zimop, Zcmp, Zvbb, Zvks, Zvkned, Zvknha, Zbkx, Zbb, Zbc, Zabha, Zacas, Zknh, Zks, Zkne, Zfbfmin, Zvfbfwma, Zcd, F, D, Q, B）
                        // 包含的指令通常不直接导致程序计数器跳转，因此默认保留。
                        // 如果需要更细致的过滤，可以为每个扩展添加匹配逻辑。
                        _ => true, // 默认保留其他所有 RV32 扩展的指令
                    }
                }
                RiscvInstruction::RV64(rv64_instr) => {
                    match rv64_instr {
                        // RV64指令集按扩展分组，然后匹配特定指令
                        RV64Instruction::I(instr) => {
                            match instr {
                                RV64IInstructions::JAL(_)
                                | RV64IInstructions::JALR(_)
                                | RV64IInstructions::BEQ(_)
                                | RV64IInstructions::BNE(_)
                                | RV64IInstructions::BLT(_)
                                | RV64IInstructions::BGE(_)
                                | RV64IInstructions::BLTU(_)
                                | RV64IInstructions::BGEU(_)
                                | RV64IInstructions::ECALL(_)
                                | RV64IInstructions::EBREAK(_)
                                | RV64IInstructions::MRET(_)
                                | RV64IInstructions::WFI(_) => false,
                                _ => true, // 保留 RV64I 中其他非跳转指令
                            }
                        }
                        RV64Instruction::C(instr) => {
                            match instr {
                                RV64CInstructions::C_J(_)
                                | RV64CInstructions::C_JALR(_)
                                | RV64CInstructions::C_JR(_)
                                | RV64CInstructions::C_BEQZ(_)
                                | RV64CInstructions::C_BNEZ(_) => false,
                                // RV64CInstructions 中的 C_LDSP, C_LD, C_SDSP, C_SD 虽是加载/存储，但间接影响堆栈，这里不作为跳转过滤
                                _ => true, // 保留 RV64C 中其他非跳转指令
                            }
                        }
                        RV64Instruction::S(instr) => {
                            match instr {
                                RV64SInstructions::SRET(_) => false,
                                RV64SInstructions::SFENCE_VMA(_) => false, // 间接影响控制流
                            }
                        }
                        RV64Instruction::Sdext(instr) => match instr {
                            RV64SdextInstructions::DRET(_) => false,
                        },
                        RV64Instruction::Smrnmi(instr) => match instr {
                            RV64SmrnmiInstructions::MNRET(_) => false,
                        },
                        RV64Instruction::Zalrsc(instr) => {
                            // LR/SC 同样处理
                            match instr {
                                RV64ZalrscInstructions::LR_W(_)
                                | RV64ZalrscInstructions::SC_W(_)
                                | RV64ZalrscInstructions::LR_D(_)
                                | RV64ZalrscInstructions::SC_D(_) => false,
                            }
                        }
                        RV64Instruction::V(instr) => {
                            // VSETVLI, VSETVL, VSETIVLI 改变向量配置，影响指令流
                            match instr {
                                RV64VInstructions::VSETVLI(_)
                                | RV64VInstructions::VSETVL(_)
                                | RV64VInstructions::VSETIVLI(_) => false,
                                _ => true, // 保留 RV64V 中其他指令
                            }
                        }
                        // 处理其他可能间接影响控制流的指令
                        RV64Instruction::Svinval(instr) => {
                            // Svinval 指令
                            match instr {
                                RV64SvinvalInstructions::SFENCE_W_INVAL(_)
                                | RV64SvinvalInstructions::HINVAL_VVMA(_)
                                | RV64SvinvalInstructions::SFENCE_INVAL_IR(_)
                                | RV64SvinvalInstructions::HINVAL_GVMA(_)
                                | RV64SvinvalInstructions::SINVAL_VMA(_) => false,
                            }
                        }
                        RV64Instruction::H(instr) => {
                            // H 指令
                            match instr {
                                RV64HInstructions::HFENCE_GVMA(_)
                                | RV64HInstructions::HFENCE_VVMA(_) => false,
                                _ => true,
                            }
                        }
                        RV64Instruction::Zawrs(instr) => {
                            // Zawrs 指令
                            match instr {
                                RV64ZawrsInstructions::WRS_STO(_)
                                | RV64ZawrsInstructions::WRS_NTO(_) => false,
                            }
                        }
                        // 其他扩展默认保留
                        _ => true, // 默认保留其他所有 RV64 扩展的指令
                    }
                }
            }
        })
        .collect()
}
