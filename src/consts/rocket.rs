use riscv_instruction::separated_instructions::{RV32Extensions, RV64Extensions};

// RV32 Rocket 支持的扩展 (根据实际测试结果修正)
pub const RV32_ROCKET_SUPPORTED_EXTENSIONS: &[RV32Extensions] = &[
    // 基础扩展 (G包含的)
    RV32Extensions::I, // 基础整数指令
    RV32Extensions::M, // 乘除法扩展
    RV32Extensions::F, // 单精度浮点
    // RV32Extensions::D,        // 双精度浮点
    RV32Extensions::C,        // 压缩指令
    RV32Extensions::Zicsr,    // 控制状态寄存器
    RV32Extensions::Zifencei, // 指令同步
    // B扩展的部分支持 (移除不支持的Zbc和部分指令)
    RV32Extensions::Zba, // 地址生成
    RV32Extensions::Zbb, // 基础位操作
    // RV32Extensions::Zbc, // 进位位操作 - Rocket不完全支持clmul系列
    RV32Extensions::Zbs, // 单比特操作
    // 浮点扩展 - 基础支持，但不包括最新的指令
    // RV32Extensions::Zfh, // 半精度浮点 - 移除，Rocket支持有限

    // 虚拟化扩展 - 基础虚拟化指令
    RV32Extensions::H, // 虚拟化扩展
    // 特权架构扩展
    RV32Extensions::S, // 监督者模式
    // 原子操作
    RV32Extensions::Zaamo, // 原子内存操作
    RV32Extensions::Zalrsc, // Load-Reserved/Store-Conditional

                           // 条件操作 - 移除，Rocket不支持czero系列
                           // RV32Extensions::Zicond, // 条件操作
];

// RV64 Rocket 支持的扩展 (根据实际测试结果修正)
pub const RV64_ROCKET_SUPPORTED_EXTENSIONS: &[RV64Extensions] = &[
    // 基础扩展 (G包含的)
    RV64Extensions::I, // 基础整数指令
    RV64Extensions::M, // 乘除法扩展
    RV64Extensions::F, // 单精度浮点
    // RV64Extensions::D,        // 双精度浮点
    RV64Extensions::C,        // 压缩指令
    RV64Extensions::Zicsr,    // 控制状态寄存器
    RV64Extensions::Zifencei, // 指令同步
    // B扩展的部分支持
    RV64Extensions::Zba, // 地址生成
    RV64Extensions::Zbb, // 基础位操作
    // RV64Extensions::Zbc, // 进位位操作 - 移除clmul系列指令
    RV64Extensions::Zbs, // 单比特操作
    // 浮点扩展 - 保守支持
    // RV64Extensions::Zfh, // 半精度浮点 - 移除新式指令

    // 虚拟化扩展
    RV64Extensions::H, // 虚拟化扩展
    // 特权架构扩展
    RV64Extensions::S, // 监督者模式
    // 原子操作
    RV64Extensions::Zaamo, // 原子内存操作
    RV64Extensions::Zalrsc, // Load-Reserved/Store-Conditional

                           // 移除的扩展
                           // RV64Extensions::Zicond, // 条件操作 - czero.eqz/czero.nez不支持
                           // RV64Extensions::Q, // 四精度浮点 - 大部分指令不支持
];
