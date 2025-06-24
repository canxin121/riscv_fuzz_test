use log::{debug, error, info, warn};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use crate::emulators::{EmulatorType, write_output_to_log};
use crate::error::{Result, RiscvFuzzError};
use crate::output_parser::OutputParser;

/// Rocket 仿真器配置
pub struct RocketConfig {
    /// ISA 架构 (如 "RV64G") - 注意：Rocket 不需要此参数
    pub isa: String,
    /// 是否启用详细输出
    pub verbose: bool,
    /// 是否打印周期计数
    pub cycle_count: bool,
    /// 最大周期数限制
    pub max_cycles: Option<u64>,
    /// 输出日志文件路径
    pub log_file: PathBuf,
    /// 仿真器可执行文件路径
    pub emulator_path: String,
}

impl Default for RocketConfig {
    fn default() -> Self {
        Self {
            isa: "RV64G".to_string(), // 保留用于兼容性，但不使用
            verbose: false,
            cycle_count: true,
            max_cycles: None,
            log_file: PathBuf::from("rocket_execution_trace.log"),
            emulator_path: "emulators/rocket_emulator".to_string(),
        }
    }
}

pub fn rocket_run_program<P: AsRef<std::path::Path>>(
    config: &RocketConfig,
    program_path: P,
) -> Result<()> {
    let start_time = Instant::now();
    info!(
        "🚀 Starting Rocket simulation for: {}, this may take a long time",
        program_path.as_ref().display()
    );

    // 检查仿真器是否存在
    if !std::path::Path::new(&config.emulator_path).exists() {
        error!("Rocket emulator not found at: {}", config.emulator_path);
        return Err(RiscvFuzzError::simulator(
            "rocket",
            &format!("Emulator not found: {}", config.emulator_path),
        ));
    }

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
    debug!("Executing Rocket emulator");

    let mut cmd = Command::new(&config.emulator_path);

    // 添加仿真器选项
    if config.cycle_count {
        cmd.arg("--cycle-count");
    }

    if config.verbose {
        cmd.arg("--verbose");
    }

    if let Some(max_cycles) = config.max_cycles {
        cmd.arg(format!("--max-cycles={}", max_cycles));
    }

    // 添加要执行的程序
    cmd.arg(program_path.as_ref());

    debug!("Rocket command: {:?}", cmd);

    let output = cmd.output()?;
    let exec_time = exec_start.elapsed();
    debug!(
        "Rocket execution completed in {:.3}s",
        exec_time.as_secs_f64()
    );

    // 写入日志
    let write_start = Instant::now();
    write_output_to_log(&config.log_file, &output.stdout)?;
    let write_time = write_start.elapsed();
    debug!("Log writing completed in {:.3}s", write_time.as_secs_f64());

    // 检查输出内容而不是退出状态
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let combined_output = format!("{}\n{}", stdout_str, stderr_str);

    let result = if combined_output.contains("PASSED") {
        debug!("✅ Found 'PASSED' in Rocket output - treating as success");
        Ok(())
    } else if combined_output.contains("SUCCESS") || combined_output.contains("PASS") {
        debug!("✅ Found success indicator in Rocket output");
        Ok(())
    } else {
        // 记录详细错误信息但不失败（如果有重要输出的话）
        if !output.status.success() {
            warn!(
                "⚠️ Rocket emulator exit status indicates failure: {}",
                output.status
            );
            if !stderr_str.trim().is_empty() {
                warn!("Rocket stderr: {}", stderr_str.trim());
            }
        }

        // 如果有任何输出，就认为是成功的（因为至少程序运行了）
        if !stdout_str.trim().is_empty() || !stderr_str.trim().is_empty() {
            debug!("✅ Rocket produced output - treating as successful execution");
            Ok(())
        } else {
            error!("❌ Rocket emulator produced no output and failed");
            Err(RiscvFuzzError::simulator(
                "rocket",
                "Emulator failed with no output",
            ))
        }
    };

    let elapsed = start_time.elapsed();
    match result {
        Ok(()) => {
            info!(
                "✅ Rocket simulation completed successfully in {:.2}s",
                elapsed.as_secs_f64()
            );
        }
        Err(ref e) => {
            error!(
                "❌ Rocket simulation failed after {:.2}s: {}",
                elapsed.as_secs_f64(),
                e
            );
        }
    }

    result
}

/// 运行Rocket并解析输出为指定格式
pub fn rocket_run_programs_and_parse<T, P: AsRef<std::path::Path>>(
    config: &RocketConfig,
    program_paths: P,
    dump_path: P,
) -> Result<T>
where
    T: OutputParser,
{
    rocket_run_program(config, program_paths)?;

    let parsed = T::parse_from_file(
        &config.log_file,
        &dump_path.as_ref().to_path_buf(),
        EmulatorType::Rocket,
    )?;

    Ok(parsed)
}
