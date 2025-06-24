use thiserror::Error;

/// 统一的错误类型，涵盖所有可能的错误情况
#[derive(Error, Debug)]
pub enum RiscvFuzzError {
    #[error("IO操作失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("文件操作错误: {message}")]
    File { message: String },

    #[error("ELF构建失败: {stage} - {details}")]
    ElfBuild { stage: String, details: String },

    #[error("模拟器执行失败: {simulator} - {message}")]
    Simulator { simulator: String, message: String },

    #[error("输出解析失败: {format} - {message}")]
    OutputParsing { format: String, message: String },

    #[error("配置错误: {message}")]
    Config { message: String },

    #[error("指令生成失败: {message}")]
    InstructionGeneration { message: String },

    #[error("差分分析错误: {message}")]
    DiffAnalysis { message: String },

    #[error("PC溯源失败: PC=0x{pc:016X} - {message}")]
    PcTracing { pc: u64, message: String },

    #[error("JSON序列化/反序列化失败: {0}")]
    Json(#[from] serde_json::Error),

    #[error("系统错误: {message}")]
    System { message: String },
}

impl RiscvFuzzError {
    pub fn file<S: Into<String>>(message: S) -> Self {
        Self::File {
            message: message.into(),
        }
    }

    pub fn elf_build<S: Into<String>>(stage: S, details: S) -> Self {
        Self::ElfBuild {
            stage: stage.into(),
            details: details.into(),
        }
    }

    pub fn simulator<S: Into<String>>(simulator: S, message: S) -> Self {
        Self::Simulator {
            simulator: simulator.into(),
            message: message.into(),
        }
    }

    pub fn output_parsing<S: Into<String>>(format: S, message: S) -> Self {
        Self::OutputParsing {
            format: format.into(),
            message: message.into(),
        }
    }

    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    pub fn instruction_generation<S: Into<String>>(message: S) -> Self {
        Self::InstructionGeneration {
            message: message.into(),
        }
    }

    pub fn diff_analysis<S: Into<String>>(message: S) -> Self {
        Self::DiffAnalysis {
            message: message.into(),
        }
    }

    pub fn pc_tracing<S: Into<String>>(pc: u64, message: S) -> Self {
        Self::PcTracing {
            pc,
            message: message.into(),
        }
    }

    pub fn system<S: Into<String>>(message: S) -> Self {
        Self::System {
            message: message.into(),
        }
    }
}

/// 简化的Result类型别名
pub type Result<T> = std::result::Result<T, RiscvFuzzError>;
