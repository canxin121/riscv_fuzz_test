use log::{debug, error, info};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use crate::emulators::{EmulatorType, write_output_to_log};
use crate::error::{Result, RiscvFuzzError};
use crate::output_parser::OutputParser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpikeConfig {
    /// ISA 架构 (如 "RV64G")
    pub isa: String,
    /// 输出日志文件路径
    pub log_file: PathBuf,
}

impl Default for SpikeConfig {
    fn default() -> Self {
        Self {
            isa: "RV64G".to_string(),
            log_file: PathBuf::from("execution_trace.log"),
        }
    }
}

pub fn spike_run_program<P: AsRef<std::path::Path>>(
    config: &SpikeConfig,
    program_path: P,
) -> Result<()> {
    let start_time = Instant::now();

    // 检查程序文件是否存在
    if !program_path.as_ref().exists() {
        error!(
            "Program file not found: {}",
            program_path.as_ref().display()
        );
        return Err(RiscvFuzzError::file(format!(
            "Program file not found: {}",
            program_path.as_ref().display()
        )));
    }

    // 构建并执行命令
    let exec_start = Instant::now();
    debug!("Executing Spike simulator");

    let mut cmd = Command::new("spike");
    cmd.arg(format!("--isa={}", config.isa));
    cmd.arg(program_path.as_ref());
    let output = cmd.output()?;

    let exec_time = exec_start.elapsed();
    debug!(
        "Spike execution completed in {:.3}s",
        exec_time.as_secs_f64()
    );

    // 写入日志
    let write_start = Instant::now();
    write_output_to_log(&config.log_file, &output.stdout)?;
    let write_time = write_start.elapsed();
    debug!("Log writing completed in {:.3}s", write_time.as_secs_f64());

    let elapsed = start_time.elapsed();
    info!(
        "✅ Spike simulation completed successfully in {:.2}s",
        elapsed.as_secs_f64()
    );

    Ok(())
}

/// 运行Spike并解析输出为指定格式
pub fn spike_run_programs_and_parse<T, P: AsRef<std::path::Path>>(
    config: &SpikeConfig,
    program_paths: P,
    dump_path: P,
) -> Result<T>
where
    T: OutputParser,
{
    // 详细打印参数
    println!("Running Spike with config: {:?}", config);
    println!("Program paths: {:?}", program_paths.as_ref());
    println!("Dump path: {:?}", dump_path.as_ref());

    spike_run_program(config, program_paths)?;
    let dump_path_buf = dump_path.as_ref().to_path_buf();
    let parsed = T::parse_from_file(&config.log_file, &dump_path_buf, EmulatorType::Spike)?;

    Ok(parsed)
}
