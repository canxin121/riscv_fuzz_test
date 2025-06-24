pub mod common;
pub mod debug;
pub mod standard;
pub mod util;

use crate::elf::tracer::InstructionTrace;
use crate::emulators::EmulatorType;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

// Constant definitions
pub const MARKER_REGISTERS_INT_ONLY: u64 = 0xFEEDC0DE2000;
pub const MARKER_REGISTERS_INT_AND_FLOAT: u64 = 0xFEEDC0DE1000;
pub const MARKER_EXCEPTION_CSR: u64 = 0xBADC0DE1000;

/// Register dump structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistersDump {
    pub dump_type: MarkerType,
    pub int_registers: [u64; 32],
    pub core_csrs: CoreCSRs,
    pub float_registers: Option<[u64; 32]>,
    pub float_csr: Option<u64>,
    pub position: usize,
}

/// Exception dump structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExceptionDump {
    pub csrs: ExceptionCSRs,
    pub position: usize,
    pub inst_trace: Option<InstructionTrace>,
}

// Marker type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarkerType {
    RegistersIntOnly,
    RegistersIntAndFloat,
    ExceptionCSR,
    Unknown(u64),
}

impl fmt::Display for MarkerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarkerType::RegistersIntOnly => write!(f, "Integer register dump"),
            MarkerType::RegistersIntAndFloat => write!(f, "Integer + floating-point register dump"),
            MarkerType::ExceptionCSR => write!(f, "Exception CSR dump"),
            MarkerType::Unknown(val) => write!(f, "Unknown marker(0x{:016X})", val),
        }
    }
}

/// Core CSR structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreCSRs {
    pub mstatus: u64,
    pub misa: u64,
    pub medeleg: u64,
    pub mideleg: u64,
    pub mie: u64,
    pub mtvec: u64,
    pub mcounteren: u64,
    pub mscratch: u64,
    pub mepc: u64,
    pub mcause: u64,
    pub mtval: u64,
    pub mip: u64,
    pub mcycle: u64,
    pub minstret: u64,
    pub mvendorid: u64,
    pub marchid: u64,
    pub mimpid: u64,
    pub mhartid: u64,
}

/// Exception CSR structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExceptionCSRs {
    pub mstatus: u64,
    pub mcause: u64,
    pub mepc: u64,
    pub mtval: u64,
    pub mie: u64,
    pub mip: u64,
    pub mtvec: u64,
    pub mscratch: u64,
    pub mhartid: u64,
}

/// Output parser trait
pub trait OutputParser: Sized + std::fmt::Display + Serialize {
    fn parse_from_file<P: AsRef<Path>>(
        log_path: P,
        dump_path: P,
        emulator_type: EmulatorType,
    ) -> Result<Self>;
}

/// Generic parsing function
pub fn parse_output_from_file<T, P: AsRef<Path>>(
    log_path: P,
    dump_path: P,
    emulator_type: EmulatorType,
) -> Result<T>
where
    T: OutputParser,
{
    let parsed = T::parse_from_file(&log_path, &dump_path, emulator_type)?;

    // Save json file
    let json_path = log_path.as_ref().with_extension("json");
    let json_content = serde_json::to_string_pretty(&parsed)?;
    fs::write(&json_path, json_content)?;

    // Save md file
    let md_path = log_path.as_ref().with_extension("md");
    let md_content = format!("{}", parsed);
    fs::write(&md_path, md_content)?;

    Ok(parsed)
}
