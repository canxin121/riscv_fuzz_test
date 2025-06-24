use clap::{Parser, Subcommand};
use log::info;
use log::warn;
use rayon::prelude::*;
use riscv_fuzz_test::consts::rocket::RV64_ROCKET_SUPPORTED_EXTENSIONS;
use riscv_fuzz_test::elf::build::build_elf;
use riscv_fuzz_test::emulators::{EmulatorType, OutputFormat, run_emulator, run_single_emulator};
use riscv_fuzz_test::error::{Result, RiscvFuzzError};
use riscv_fuzz_test::output_diff::analysis::remove_rocket_illegal_inst::get_rocket_illegal_instruction_originals;
use riscv_fuzz_test::output_diff::analysis::remove_rocket_illegal_inst::has_rocket_only_illegal_instructions;
use riscv_fuzz_test::output_diff::analysis::shortten_asm_for_regs::extract_minimal_instructions_for_regs;
use riscv_fuzz_test::output_diff::diff::RegistersDumpDiff;
use riscv_fuzz_test::output_diff::diff::compare_outputs;
use riscv_fuzz_test::output_diff::diff::standard_diff::StandardExecutionOutputDiff;
// Added
use riscv_fuzz_test::output_diff::diff_diff::compare_output_diffs; // Added
use riscv_fuzz_test::output_diff::utils::remove_instructions_assembly;
use riscv_fuzz_test::output_parser::common::CommonExecutionOutput; // Added
use riscv_fuzz_test::output_parser::debug::DebugExecutionOutput; // Added
use riscv_fuzz_test::output_parser::parse_output_from_file; // Added
use riscv_fuzz_test::output_parser::standard::StandardExecutionOutput;
use riscv_fuzz_test::random_asm::asm_maker::{
    generate_instructions, generate_standard_asm_from_insts,
};
use riscv_fuzz_test::random_asm::inst_generator::GenerationOrder;
use riscv_fuzz_test::utils::{
    build_rv64_march, extract_user_code_instructions, resolve_output_dir,
};
use std::collections::HashMap;
use std::fs::{self, create_dir_all};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering}; // Added import for warn! macro

#[derive(Parser)]
#[command(name = "riscv-fuzz-test")]
#[command(about = "RISC-V fuzzing test tool for comparing Spike and Rocket emulators")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate random assembly code and run comparison in parallel
    Random {
        /// Number of instructions to generate per extension
        #[arg(short, long, default_value = "50")]
        inst_num: usize,
        /// Number of parallel test instances (default: number of CPU cores)
        #[arg(short = 'p', long)]
        parallel: Option<usize>,
        /// Fixed output directory
        #[arg(long, conflicts_with = "workspace_dir")]
        output_dir: Option<PathBuf>,
        /// Workspace directory for random output directories (default mode)
        #[arg(long, default_value = "workspace", conflicts_with = "output_dir")]
        workspace_dir: Option<PathBuf>,
    },
    /// Run comparison with existing assembly file
    Run {
        /// Path to assembly file (.s or .S)
        #[arg(short, long)]
        assembly_file: PathBuf,
        /// Output build directory  
        #[arg(short, long, default_value = "build")]
        build_dir: PathBuf,
        /// Output format for parsing and diffing
        #[arg(short = 'f', long, value_enum, default_value = "standard")] // Added
        format: OutputFormat, // Added
        /// Enable automatic retry when Rocket-only illegal instructions are found
        #[arg(long, default_value = "true")]
        auto_retry: bool,
    },
    /// Run single emulator with specified output format
    Emulate {
        /// Path to assembly file (.s or .S)
        #[arg(short, long)]
        assembly_file: PathBuf,
        /// Which emulator to use
        #[arg(short = 'e', long, value_enum)]
        emulator: EmulatorType,
        /// Output format for parsing
        #[arg(short = 'f', long, value_enum, default_value = "standard")]
        format: OutputFormat,
        /// Output build directory
        #[arg(short, long, default_value = "emulate_build")]
        build_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let march_string = setup_environment()?;

    match cli.command {
        Commands::Random {
            inst_num,
            parallel,
            output_dir,
            workspace_dir,
        } => {
            let num_threads = parallel.unwrap_or_else(|| num_cpus::get());
            info!(
                "🎲 Running in random mode with {} instructions per extension, {} parallel instances",
                inst_num, num_threads
            );

            let resolved_output_dir = resolve_output_dir(output_dir, workspace_dir)?;
            let _ = create_dir_all(&resolved_output_dir);

            run_parallel_random_tests(&resolved_output_dir, inst_num, num_threads, &march_string)?;
        }
        Commands::Run {
            assembly_file,
            build_dir,
            format, // Added
            auto_retry,
        } => {
            info!(
                "📁 Running in file mode with assembly file: {:?}, format: {:?}, auto_retry: {}", // Updated log
                assembly_file, format, auto_retry
            );

            if !assembly_file.exists() {
                return Err(RiscvFuzzError::file(format!(
                    "Assembly file does not exist: {:?}",
                    assembly_file
                )));
            }

            let extension = assembly_file
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            if !matches!(extension.to_lowercase().as_str(), "s" | "S") {
                return Err(RiscvFuzzError::config(
                    "Assembly file must have .s or .S extension",
                ));
            }

            let _ = create_dir_all(&build_dir);

            process_assembly_file(
                &build_dir,
                &assembly_file,
                &march_string,
                format,
                auto_retry,
            )?; // Pass auto_retry
        }
        Commands::Emulate {
            assembly_file,
            emulator,
            format,
            build_dir,
        } => {
            info!(
                "🔬 Running emulation mode with {} emulator, {} format",
                match emulator {
                    EmulatorType::Spike => "Spike",
                    EmulatorType::Rocket => "Rocket",
                },
                match format {
                    OutputFormat::Standard => "standard",
                    OutputFormat::Debug => "debug",
                    OutputFormat::Common => "common",
                }
            );

            if !assembly_file.exists() {
                return Err(RiscvFuzzError::file(format!(
                    "Assembly file does not exist: {:?}",
                    assembly_file
                )));
            }

            let extension = assembly_file
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            if !matches!(extension.to_lowercase().as_str(), "s" | "S") {
                return Err(RiscvFuzzError::config(
                    "Assembly file must have .s or .S extension",
                ));
            }

            let _ = create_dir_all(&build_dir);

            run_single_emulator(&build_dir, &assembly_file, &march_string, emulator, format)?;
        }
    }

    Ok(())
}

/// 并行运行多个随机测试实例
fn run_parallel_random_tests(
    base_output_dir: &PathBuf,
    inst_num: usize,
    num_threads: usize,
    march_string: &str,
) -> Result<()> {
    let counter = AtomicUsize::new(0);

    // 配置rayon线程池
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .map_err(|e| RiscvFuzzError::config(&format!("Failed to initialize thread pool: {}", e)))?;

    info!("🚀 Starting {} parallel random test instances", num_threads);

    // 创建指定数量的迭代器并并行处理
    (0..num_threads)
        .into_par_iter()
        .try_for_each(|_| -> Result<()> {
            let test_id = counter.fetch_add(1, Ordering::SeqCst);
            let test_dir = base_output_dir.join(format!("test_{:06}", test_id));
            let _ = create_dir_all(&test_dir);

            info!("🎯 Starting random test #{}", test_id);

            match run_single_random_test(&test_dir, inst_num, march_string) {
                Ok(()) => {
                    info!("✅ Random test #{} completed successfully", test_id);
                }
                Err(e) => {
                    info!("❌ Random test #{} failed: {}", test_id, e);
                    // 继续运行其他测试，不中断整个流程
                }
            }

            Ok(())
        })?;

    Ok(())
}

/// 运行单个随机测试实例
fn run_single_random_test(test_dir: &PathBuf, inst_num: usize, march_string: &str) -> Result<()> {
    // 生成随机汇编代码
    let assembly_file = generate_random_assembly(test_dir, inst_num)?;

    // 处理汇编文件, 随机测试默认使用 Standard 格式
    process_assembly_file(
        test_dir,
        &assembly_file,
        march_string,
        OutputFormat::Standard,
        true, // Random tests always enable auto_retry
    )?;

    Ok(())
}

/// 处理汇编文件的完整流程：编译、运行模拟器、分析差异、可能的重试
fn process_assembly_file(
    build_dir: &PathBuf,
    assembly_file: &PathBuf,
    march_string: &str,
    format: OutputFormat, // Added format parameter
    auto_retry: bool,     // Added auto_retry parameter
) -> Result<()> {
    let linker_script = PathBuf::from("assets/linker.ld");

    // 编译汇编文件
    let build_result = build_elf(assembly_file, &linker_script, march_string)?;

    // 定义原始输出文件路径
    let spike_raw_output_path = build_dir.join("spike_output.bin");
    let rocket_raw_output_path = build_dir.join("rocket_output.bin");

    // 运行模拟器
    info!("🏃 Running Spike emulator...");
    let spike_run_res = run_emulator(
        &spike_raw_output_path,
        &build_result.executable_file,
        march_string,
        EmulatorType::Spike,
    );

    info!("🏃 Running Rocket emulator...");
    let rocket_run_res = run_emulator(
        &rocket_raw_output_path,
        &build_result.executable_file,
        march_string,
        EmulatorType::Rocket,
    );

    // 根据格式处理输出和差异
    match format {
        OutputFormat::Standard => {
            let spike_output = spike_run_res.ok().and_then(|p| {
                parse_output_from_file::<StandardExecutionOutput, _>(
                    &p,
                    &build_result.disassembly_file,
                    EmulatorType::Spike,
                )
                .ok()
            });
            let rocket_output = rocket_run_res.ok().and_then(|p| {
                parse_output_from_file::<StandardExecutionOutput, _>(
                    &p,
                    &build_result.disassembly_file,
                    EmulatorType::Rocket,
                )
                .ok()
            });

            if let (Some(spike_out), Some(rocket_out)) = (spike_output, rocket_output) {
                info!("🔄 Comparing Standard outputs...");
                let initial_diff = compare_outputs(&spike_out, &rocket_out);

                let initial_diff_json = serde_json::to_string_pretty(&initial_diff)?;
                let initial_diff_text = initial_diff.to_string();
                let initial_diff_json_file = build_dir.join("diff_standard.json");
                let initial_diff_text_file = build_dir.join("diff_standard.md");
                fs::write(&initial_diff_json_file, initial_diff_json)?;
                fs::write(&initial_diff_text_file, initial_diff_text)?;
                info!(
                    "💾 Initial Standard diff saved to: {:?} and {:?}",
                    initial_diff_json_file, initial_diff_text_file
                );

                // 检查是否有Rocket中的Illegal instruction异常 (此逻辑特定于 StandardExecutionOutputDiff)
                let rocket_has_illegal_instructions = initial_diff
                    .exceptions_diff // This is StandardExecutionOutputDiff specific
                    .as_ref()
                    .map_or(false, |ex_diff| {
                        has_rocket_only_illegal_instructions(ex_diff)
                    });

                if rocket_has_illegal_instructions && auto_retry {
                    let illegal_instructions = initial_diff
                        .exceptions_diff
                        .as_ref()
                        .map_or_else(Vec::new, |ex_diff| {
                            get_rocket_illegal_instruction_originals(ex_diff)
                        });

                    if !illegal_instructions.is_empty() {
                        info!(
                            "🚨 Found {} Rocket-only illegal instructions, attempting retry...",
                            illegal_instructions.len()
                        );

                        let new_build_dir = build_dir.join("rocket_illegal_retry");
                        let _ = create_dir_all(&new_build_dir);
                        let new_assembly_file = new_build_dir.join("retry_output.S");

                        remove_instructions_assembly::<PathBuf>(
                            &assembly_file,
                            &new_assembly_file,
                            &illegal_instructions,
                        )?;

                        let new_build_result =
                            build_elf(&new_assembly_file, &linker_script, march_string)?;

                        // Re-run emulators for retry
                        info!("🏃 Re-running Spike emulator for retry...");
                        let new_spike_run_res = run_emulator(
                            &new_build_dir.join("spike_output_retry.bin"),
                            &new_build_result.executable_file,
                            march_string,
                            EmulatorType::Spike,
                        );
                        info!("🏃 Re-running Rocket emulator for retry...");
                        let new_rocket_run_res = run_emulator(
                            &new_build_dir.join("rocket_output_retry.bin"),
                            &new_build_result.executable_file,
                            march_string,
                            EmulatorType::Rocket,
                        );

                        let new_spike_out_parsed = new_spike_run_res.ok().and_then(|p| {
                            parse_output_from_file::<StandardExecutionOutput, _>(
                                &p,
                                &new_build_result.disassembly_file,
                                EmulatorType::Spike,
                            )
                            .ok()
                        });
                        let new_rocket_out_parsed = new_rocket_run_res.ok().and_then(|p| {
                            parse_output_from_file::<StandardExecutionOutput, _>(
                                &p,
                                &new_build_result.disassembly_file,
                                EmulatorType::Rocket,
                            )
                            .ok()
                        });

                        if let (Some(new_spike_out), Some(new_rocket_out)) =
                            (new_spike_out_parsed, new_rocket_out_parsed)
                        {
                            info!("🔄 Comparing Standard outputs after retry...");
                            let new_diff = compare_outputs(&new_spike_out, &new_rocket_out);

                            let new_diff_json = serde_json::to_string_pretty(&new_diff)?;
                            let new_diff_text = new_diff.to_string();
                            let new_diff_json_file = new_build_dir.join("new_diff_standard.json");
                            let new_diff_text_file = new_build_dir.join("new_diff_standard.md");
                            fs::write(&new_diff_json_file, new_diff_json)?;
                            fs::write(&new_diff_text_file, new_diff_text)?;
                            info!(
                                "💾 New Standard diff saved to: {:?} and {:?}",
                                new_diff_json_file, new_diff_text_file
                            );

                            let retry_report = compare_output_diffs(&initial_diff, &new_diff);
                            let retry_report_file = new_build_dir.join("retry_report.md");
                            fs::write(&retry_report_file, retry_report.to_string())?;
                            info!("💾 Retry report saved to: {:?}", retry_report_file);

                            // 检查删除非法指令后是否仍有寄存器差异
                            if let Some(reg_diff) = &new_diff.register_dump_diff {
                                if !reg_diff.is_empty() && has_register_differences(reg_diff) {
                                    info!(
                                        "🎯 Found register differences after illegal instruction removal, performing minimal analysis..."
                                    );

                                    // 提取存在差异的寄存器列表
                                    let differing_regs = extract_differing_registers(reg_diff);
                                    if !differing_regs.is_empty() {
                                        // 提取用户代码指令
                                        let assembly_content =
                                            fs::read_to_string(&new_assembly_file)?;
                                        let user_instructions =
                                            extract_user_code_instructions(&assembly_content);

                                        // 进行最小化分析
                                        let minimal_instructions =
                                            extract_minimal_instructions_for_regs(
                                                user_instructions,
                                                differing_regs.clone(),
                                            );

                                        if !minimal_instructions.is_empty() {
                                            info!(
                                                "🔬 Performing minimal analysis with {} instructions for {} registers",
                                                minimal_instructions.len(),
                                                differing_regs.len()
                                            );

                                            // 创建最小化分析目录
                                            let minimal_build_dir =
                                                new_build_dir.join("minimal_analysis");
                                            let _ = create_dir_all(&minimal_build_dir);

                                            // 生成最小化汇编
                                            let minimal_assembly_file =
                                                minimal_build_dir.join("minimal_output.S");
                                            generate_minimal_assembly_for_analysis(
                                                &minimal_instructions,
                                                &minimal_assembly_file,
                                                &assembly_content,
                                            )?;

                                            // 运行最小化分析
                                            run_minimal_analysis(
                                                &minimal_build_dir,
                                                &minimal_assembly_file,
                                                march_string,
                                                &new_diff, // 传递rocket retry的差异结果进行对比
                                            )?;
                                        } else {
                                            info!("⚠️ No instructions found for minimal analysis");
                                        }
                                    }
                                }
                            }
                        } else {
                            warn!("⚠️ Failed to parse one or both emulator outputs after retry.");
                        }
                    } else {
                        info!(
                            "ℹ️ No specific illegal instructions identified for removal, no retry performed."
                        );
                    }
                } else if rocket_has_illegal_instructions && !auto_retry {
                    info!(
                        "ℹ️ Rocket-only illegal instructions found in Standard diff, but auto_retry is disabled."
                    );
                } else {
                    info!(
                        "ℹ️ No Rocket-only illegal instructions found in Standard diff, no retry needed."
                    );
                }
            } else {
                warn!("⚠️ Failed to parse one or both emulator outputs for Standard format.");
            }
        }
        OutputFormat::Debug => {
            let spike_output = spike_run_res.ok().and_then(|p| {
                parse_output_from_file::<DebugExecutionOutput, _>(
                    &p,
                    &build_result.disassembly_file,
                    EmulatorType::Spike,
                )
                .ok()
            });
            let rocket_output = rocket_run_res.ok().and_then(|p| {
                parse_output_from_file::<DebugExecutionOutput, _>(
                    &p,
                    &build_result.disassembly_file,
                    EmulatorType::Rocket,
                )
                .ok()
            });

            if let (Some(spike_out), Some(rocket_out)) = (spike_output, rocket_output) {
                info!("🔄 Comparing Debug outputs...");
                let diff = compare_outputs(&spike_out, &rocket_out);
                let diff_json = serde_json::to_string_pretty(&diff)?;
                let diff_text = diff.to_string();
                let diff_json_file = build_dir.join("diff_debug.json");
                let diff_text_file = build_dir.join("diff_debug.md");
                fs::write(&diff_json_file, diff_json)?;
                fs::write(&diff_text_file, diff_text)?;
                info!(
                    "💾 Debug diff saved to: {:?} and {:?}",
                    diff_json_file, diff_text_file
                );
            } else {
                warn!("⚠️ Failed to parse one or both emulator outputs for Debug format.");
            }
        }
        OutputFormat::Common => {
            let spike_output = spike_run_res.ok().and_then(|p| {
                parse_output_from_file::<CommonExecutionOutput, _>(
                    &p,
                    &build_result.disassembly_file,
                    EmulatorType::Spike,
                )
                .ok()
            });
            let rocket_output = rocket_run_res.ok().and_then(|p| {
                parse_output_from_file::<CommonExecutionOutput, _>(
                    &p,
                    &build_result.disassembly_file,
                    EmulatorType::Rocket,
                )
                .ok()
            });

            if let (Some(spike_out), Some(rocket_out)) = (spike_output, rocket_output) {
                info!("🔄 Comparing Common outputs...");
                let diff = compare_outputs(&spike_out, &rocket_out);
                let diff_json = serde_json::to_string_pretty(&diff)?;
                let diff_text = diff.to_string();
                let diff_json_file = build_dir.join("diff_common.json");
                let diff_text_file = build_dir.join("diff_common.md");
                fs::write(&diff_json_file, diff_json)?;
                fs::write(&diff_text_file, diff_text)?;
                info!(
                    "💾 Common diff saved to: {:?} and {:?}",
                    diff_json_file, diff_text_file
                );
            } else {
                warn!("⚠️ Failed to parse one or both emulator outputs for Common format.");
            }
        }
    }

    Ok(())
}

/// 检查是否存在整数或浮点寄存器差异
fn has_register_differences(reg_diff: &RegistersDumpDiff) -> bool {
    !reg_diff.int_registers_diff.is_empty() || !reg_diff.float_registers_diff.is_empty()
}

/// 提取存在差异的寄存器名称
fn extract_differing_registers(reg_diff: &RegistersDumpDiff) -> Vec<String> {
    let mut differing_regs = Vec::new();

    // 添加整数寄存器差异
    for (idx, _name, _val1, _val2) in &reg_diff.int_registers_diff {
        differing_regs.push(format!("x{}", idx));
    }

    // 添加浮点寄存器差异
    for (idx, _val1, _val2) in &reg_diff.float_registers_diff {
        differing_regs.push(format!("f{}", idx));
    }

    differing_regs
}

/// 生成用于最小化分析的汇编文件
fn generate_minimal_assembly_for_analysis(
    minimal_instructions: &[String],
    output_file: &PathBuf,
    original_assembly: &str,
) -> Result<()> {
    // 提取原汇编文件的头部和尾部
    let lines: Vec<&str> = original_assembly.lines().collect();
    let mut header_lines = Vec::new();
    let mut footer_lines = Vec::new();
    let mut in_user_code = false;
    let mut after_user_code = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "_user_code:" {
            header_lines.push(line);
            in_user_code = true;
        } else if in_user_code && trimmed.ends_with(":") && !trimmed.contains(' ') {
            after_user_code = true;
            footer_lines.push(line);
        } else if !in_user_code {
            header_lines.push(line);
        } else if after_user_code {
            footer_lines.push(line);
        }
    }

    let mut result = String::new();

    // 添加头部
    for line in header_lines {
        result.push_str(line);
        result.push('\n');
    }

    // 添加最小化指令
    for inst in minimal_instructions {
        result.push_str("    ");
        result.push_str(inst);
        result.push('\n');
    }

    // 添加尾部
    for line in footer_lines {
        result.push_str(line);
        result.push('\n');
    }

    fs::write(output_file, result)?;
    Ok(())
}

/// 运行最小化分析
fn run_minimal_analysis(
    build_dir: &PathBuf,
    assembly_file: &PathBuf,
    march_string: &str,
    retry_diff: &StandardExecutionOutputDiff, // 传递rocket retry的差异结果
) -> Result<()> {
    let linker_script = PathBuf::from("assets/linker.ld");

    info!("🔬 Building minimal analysis ELF...");
    let build_result = build_elf(assembly_file, &linker_script, march_string)?;

    // 运行模拟器
    info!("🏃 Running minimal analysis - Spike emulator...");
    let spike_run_res = run_emulator(
        &build_dir.join("spike_minimal.bin"),
        &build_result.executable_file,
        march_string,
        EmulatorType::Spike,
    );

    info!("🏃 Running minimal analysis - Rocket emulator...");
    let rocket_run_res = run_emulator(
        &build_dir.join("rocket_minimal.bin"),
        &build_result.executable_file,
        march_string,
        EmulatorType::Rocket,
    );

    // 解析输出并比较
    if let (Ok(spike_path), Ok(rocket_path)) = (spike_run_res, rocket_run_res) {
        let spike_output = parse_output_from_file::<StandardExecutionOutput, _>(
            &spike_path,
            &build_result.disassembly_file,
            EmulatorType::Spike,
        );
        let rocket_output = parse_output_from_file::<StandardExecutionOutput, _>(
            &rocket_path,
            &build_result.disassembly_file,
            EmulatorType::Rocket,
        );

        if let (Ok(spike_out), Ok(rocket_out)) = (spike_output, rocket_output) {
            info!("🔄 Comparing minimal analysis outputs...");
            let minimal_diff = compare_outputs(&spike_out, &rocket_out);

            // 保存最小化分析结果
            let minimal_diff_json = serde_json::to_string_pretty(&minimal_diff)?;
            let minimal_diff_text = minimal_diff.to_string();
            let minimal_diff_json_file = build_dir.join("minimal_diff.json");
            let minimal_diff_text_file = build_dir.join("minimal_diff.md");
            fs::write(&minimal_diff_json_file, minimal_diff_json)?;
            fs::write(&minimal_diff_text_file, minimal_diff_text)?;

            // 生成 diff diff 报告 (比较rocket retry的差异和最小化代码的差异)
            let minimal_analysis_report = compare_output_diffs(retry_diff, &minimal_diff);
            let minimal_analysis_report_file = build_dir.join("minimal_vs_retry_diff_report.md");
            fs::write(
                &minimal_analysis_report_file,
                minimal_analysis_report.to_string(),
            )?;

            info!("💾 Minimal analysis results saved to: {:?}", build_dir);
            info!(
                "💾 Minimal vs retry diff report saved to: {:?}",
                minimal_analysis_report_file
            );

            // 检查最小化后是否仍有差异
            if let Some(reg_diff) = &minimal_diff.register_dump_diff {
                if !reg_diff.is_empty() && has_register_differences(reg_diff) {
                    info!("🎯 Minimal analysis still shows register differences");
                } else {
                    info!(
                        "✅ Minimal analysis shows no register differences - issue may be resolved"
                    );
                }
            }
        } else {
            warn!("⚠️ Failed to parse minimal analysis outputs");
        }
    } else {
        warn!("⚠️ Failed to run minimal analysis emulators");
    }

    Ok(())
}

fn generate_random_assembly(build_dir: &PathBuf, inst_num: usize) -> Result<PathBuf> {
    let mut instruction_counts = HashMap::new();
    for &extension in RV64_ROCKET_SUPPORTED_EXTENSIONS {
        instruction_counts.insert(extension, inst_num);
    }
    let rng = &mut rand::rng();

    let insts = generate_instructions(&instruction_counts, GenerationOrder::RandomShuffle, rng);

    let asm_str = generate_standard_asm_from_insts(&insts);

    let assembly_file = build_dir.join("generated_output.S");
    fs::write(&assembly_file, asm_str)?;

    Ok(assembly_file)
}

fn setup_environment() -> Result<String> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_secs()
        .init();

    let mut exts = RV64_ROCKET_SUPPORTED_EXTENSIONS.to_vec();
    exts.push(riscv_instruction::separated_instructions::RV64Extensions::D);

    let march_string = build_rv64_march(&exts);
    Ok(march_string)
}
