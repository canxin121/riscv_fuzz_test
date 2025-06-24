pub mod rocket;
pub mod spike;
use crate::{
    elf::build::build_elf,
    error::Result, // Added RiscvFuzzError for run_emulator
    output_parser::{common::CommonExecutionOutput, debug::DebugExecutionOutput},
};
use std::{
    fmt::Display,
    fs::{self, File},
    io::{self, Write as _},
    path::PathBuf,
};

use clap::ValueEnum;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{
    emulators::{
        rocket::{RocketConfig, rocket_run_programs_and_parse},
        spike::{SpikeConfig, spike_run_programs_and_parse},
    },
    output_parser::{OutputParser, standard::StandardExecutionOutput},
};

// Add serde::Serialize to the imports if it's not already there for the whole crate
// use serde::Serialize; // Assuming it's available at crate level or imported in this module

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize, ValueEnum, Hash)] // 添加 Default
pub enum EmulatorType {
    Spike,
    Rocket,
}

impl Display for EmulatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulatorType::Spike => write!(f, "Spike"),
            EmulatorType::Rocket => write!(f, "Rocket"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimulatorResult<T = StandardExecutionOutput>
where
    T: OutputParser,
{
    pub spike_output: Option<T>,
    pub spike_output_file: Option<PathBuf>,
    pub spike_log_file: Option<PathBuf>,
    pub rocket_output: Option<T>,
    pub rocket_output_file: Option<PathBuf>,
    pub rocket_log_file: Option<PathBuf>,
}

pub fn run_and_parse_all_simulators<T, P: AsRef<std::path::Path>>(
    build_dir: P,
    march_string: &str,
    executable_file: P,
    dump_file: P,
) -> SimulatorResult<T>
where
    T: OutputParser + Serialize, // Added Serialize
{
    // Run Spike
    let spike_config = SpikeConfig {
        isa: march_string.to_string(),
        log_file: build_dir.as_ref().join("spike_execution_trace.log"),
    };
    let spike_output_path = build_dir.as_ref().join("spike_output.json");

    let spike_execution_result = spike_run_programs_and_parse::<T, PathBuf>(
        &spike_config,
        executable_file.as_ref().to_path_buf(),
        dump_file.as_ref().to_path_buf(),
    );

    let (spike_output, spike_output_file) = match spike_execution_result {
        Ok(parsed_output) => {
            // 序列化解析后的输出
            let result_str = serde_json::to_string(&parsed_output).unwrap();
            match std::fs::write(&spike_output_path, result_str) {
                Ok(_) => (Some(parsed_output), Some(spike_output_path)),
                Err(e) => {
                    error!(
                        "Failed to write Spike output to file {:?}: {}",
                        spike_output_path, e
                    );
                    panic!(
                        "Failed to write Spike output to file {:?}: {}",
                        spike_output_path, e
                    );
                }
            }
        }
        Err(e) => {
            error!("Spike execution failed: {:?}", e);
            (None, None)
        }
    };

    // Run Rocket
    let rocket_config = RocketConfig {
        isa: march_string.to_string(),
        verbose: false,
        cycle_count: false,
        max_cycles: None,
        log_file: build_dir.as_ref().join("rocket_execution_trace.log"),
        emulator_path: "emulators/rocket_emulator".to_string(),
    };

    let rocket_output_path = build_dir.as_ref().join("rocket_output.json");

    let rocket_execution_result =
        rocket_run_programs_and_parse::<T, P>(&rocket_config, executable_file, dump_file);

    let (rocket_output, rocket_output_file) = match rocket_execution_result {
        Ok(parsed_output) => {
            // 序列化解析后的输出
            let result_str = serde_json::to_string(&parsed_output).unwrap();
            match std::fs::write(&rocket_output_path, result_str) {
                Ok(_) => (Some(parsed_output), Some(rocket_output_path)),
                Err(e) => {
                    error!(
                        "Failed to write Rocket output to file {:?}: {}",
                        rocket_output_path, e
                    );
                    panic!(
                        "Failed to write Rocket output to file {:?}: {}",
                        rocket_output_path, e
                    );
                }
            }
        }
        Err(e) => {
            error!("Rocket execution failed: {:?}", e);
            (None, None)
        }
    };

    SimulatorResult {
        spike_output,
        spike_log_file: Some(spike_config.log_file),
        spike_output_file,
        rocket_output,
        rocket_log_file: Some(rocket_config.log_file),
        rocket_output_file,
    }
}

/// 将命令输出写入日志文件
fn write_output_to_log<P: AsRef<std::path::Path>>(log_path: P, stdout: &[u8]) -> io::Result<()> {
    let mut file = File::create(log_path.as_ref())?;

    // 写入标准输出
    if !stdout.is_empty() {
        file.write_all(stdout)?;
    }

    file.flush()?;
    Ok(())
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum OutputFormat {
    /// Standard format (deduplicated exceptions + register dump)
    Standard,
    /// Debug format (focused on register dumps)
    Debug,
    /// Common format (raw parsed data)
    Common,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Standard => write!(f, "Standard"),
            OutputFormat::Debug => write!(f, "Debug"),
            OutputFormat::Common => write!(f, "Common"),
        }
    }
}

/// Runs a specified emulator with the given program and saves its raw output.
/// The `output_format` parameter can influence emulator flags (e.g., Spike's -d).
pub fn run_emulator(
    raw_output_target_path: &PathBuf,
    executable_file: &PathBuf,
    march_string: &str,
    emulator_type: EmulatorType,
) -> Result<PathBuf> {
    match emulator_type {
        EmulatorType::Spike => {
            let config = SpikeConfig {
                isa: march_string.to_string(),
                // Enable Spike's debug mode if the desired output format is Debug.
                // This is a heuristic; Spike's native debug output might differ
                // from what our DebugExecutionOutput expects, but it's a common case.
                log_file: raw_output_target_path.clone(),
            };
            spike::spike_run_program(&config, executable_file)?;
        }
        EmulatorType::Rocket => {
            let config = RocketConfig {
                isa: march_string.to_string(), // Not directly used by Rocket command but kept for consistency
                verbose: false, // Default, can be configured if needed based on output_format
                cycle_count: false, // Default
                max_cycles: None, // Default
                log_file: raw_output_target_path.clone(),
                emulator_path: "emulators/rocket_emulator".to_string(),
            };
            rocket::rocket_run_program(&config, executable_file)?;
        }
    }
    Ok(raw_output_target_path.clone())
}

/// 运行单个模拟器并解析输出
pub fn run_single_emulator<P: AsRef<std::path::Path>>(
    build_dir: P,
    assembly_file: P,
    march_string: &str,
    emulator: EmulatorType,
    format: OutputFormat,
) -> Result<()> {
    let build_dir = build_dir.as_ref().to_path_buf();
    let linker_script = PathBuf::from("assets/linker.ld");

    // 编译汇编文件
    info!("🔨 Compiling assembly file...");
    let build_result = build_elf(
        &assembly_file.as_ref().to_path_buf(),
        &linker_script,
        march_string,
    )?;

    // 根据选择的模拟器和格式运行
    match (emulator, format) {
        (EmulatorType::Spike, OutputFormat::Standard) => {
            run_emulator_with_format::<StandardExecutionOutput, &PathBuf>(
                &build_dir,
                &build_result.executable_file,
                &build_result.disassembly_file,
                march_string,
                EmulatorType::Spike,
            )?;
        }
        (EmulatorType::Spike, OutputFormat::Debug) => {
            run_emulator_with_format::<DebugExecutionOutput, &PathBuf>(
                &build_dir,
                &build_result.executable_file,
                &build_result.disassembly_file,
                march_string,
                EmulatorType::Spike,
            )?;
        }
        (EmulatorType::Spike, OutputFormat::Common) => {
            run_emulator_with_format::<CommonExecutionOutput, &PathBuf>(
                &build_dir,
                &build_result.executable_file,
                &build_result.disassembly_file,
                march_string,
                EmulatorType::Spike,
            )?;
        }
        (EmulatorType::Rocket, OutputFormat::Standard) => {
            run_emulator_with_format::<StandardExecutionOutput, &PathBuf>(
                &build_dir,
                &build_result.executable_file,
                &build_result.disassembly_file,
                march_string,
                EmulatorType::Rocket,
            )?;
        }
        (EmulatorType::Rocket, OutputFormat::Debug) => {
            run_emulator_with_format::<DebugExecutionOutput, &PathBuf>(
                &build_dir,
                &build_result.executable_file,
                &build_result.disassembly_file,
                march_string,
                EmulatorType::Rocket,
            )?;
        }
        (EmulatorType::Rocket, OutputFormat::Common) => {
            run_emulator_with_format::<CommonExecutionOutput, &PathBuf>(
                &build_dir,
                &build_result.executable_file,
                &build_result.disassembly_file,
                march_string,
                EmulatorType::Rocket,
            )?;
        }
    }

    Ok(())
}

/// 运行指定模拟器并按指定格式解析输出
pub fn run_emulator_with_format<T, P: AsRef<std::path::Path>>(
    build_dir: P,
    executable_file: P,
    dump_file: P,
    march_string: &str,
    emulator: EmulatorType,
) -> Result<()>
where
    T: OutputParser + std::fmt::Display + Serialize, // Added Serialize
{
    let parsed_output = match emulator {
        EmulatorType::Spike => {
            let config = SpikeConfig {
                isa: march_string.to_string(),
                log_file: build_dir.as_ref().join("spike_execution_trace.log"),
            };
            spike_run_programs_and_parse::<T, P>(&config, executable_file, dump_file)?
        }
        EmulatorType::Rocket => {
            let config = RocketConfig {
                isa: march_string.to_string(),
                verbose: false,
                cycle_count: false,
                max_cycles: None,
                log_file: build_dir.as_ref().join("rocket_execution_trace.log"),
                emulator_path: "emulators/rocket_emulator".to_string(),
            };
            rocket_run_programs_and_parse::<T, P>(&config, executable_file, dump_file)?
        }
    };

    // 保存结果到文件
    let json_file = build_dir.as_ref().join(format!("{}_output.json", emulator));
    let json_content = serde_json::to_string_pretty(&parsed_output)?;
    fs::write(&json_file, json_content)?;
    info!("💾 JSON output saved to: {:?}", json_file);

    let text_file = build_dir.as_ref().join(format!("{}_output.md", emulator));
    let text_content = format!("{}", parsed_output);
    fs::write(&text_file, text_content)?;
    info!("💾 Text output saved to: {:?}", text_file);

    Ok(())
}