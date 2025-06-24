use thiserror::Error;

/// Unified error type covering all possible error scenarios
#[derive(Error, Debug)]
pub enum RiscvFuzzError {
    #[error("IO operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("File operation error: {message}")]
    File { message: String },

    #[error("ELF build failed: {stage} - {details}")]
    ElfBuild { stage: String, details: String },

    #[error("Simulator execution failed: {simulator} - {message}")]
    Simulator { simulator: String, message: String },

    #[error("Output parsing failed: {format} - {message}")]
    OutputParsing { format: String, message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Instruction generation failed: {message}")]
    InstructionGeneration { message: String },

    #[error("Diff analysis error: {message}")]
    DiffAnalysis { message: String },

    #[error("PC tracing failed: PC=0x{pc:016X} - {message}")]
    PcTracing { pc: u64, message: String },

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("System error: {message}")]
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

/// Simplified Result type alias
pub type Result<T> = std::result::Result<T, RiscvFuzzError>;
