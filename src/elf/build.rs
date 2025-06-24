use crate::error::{Result, RiscvFuzzError};
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// ELF 构建结果，包含所有生成的文件路径
#[derive(Debug, Clone)]
pub struct ElfBuildResult {
    /// 预处理后的汇编文件路径（仅当输入为 .S 文件时）
    pub preprocessed_assembly: Option<PathBuf>,
    /// 目标文件路径
    pub object_file: PathBuf,
    /// 可执行文件路径
    pub executable_file: PathBuf,
    /// 反汇编文件路径
    pub disassembly_file: PathBuf,
}

impl ElfBuildResult {
    /// 获取所有生成的文件路径
    pub fn all_files(&self) -> Vec<&PathBuf> {
        let mut files = vec![
            &self.object_file,
            &self.executable_file,
            &self.disassembly_file,
        ];
        if let Some(ref preprocessed) = self.preprocessed_assembly {
            files.push(preprocessed);
        }
        files
    }
}

/// 一键编译 ELF 文件，返回详细的构建结果
pub fn build_elf<P: AsRef<std::path::Path>>(
    assembly_file: P,
    linker_script: P,
    arch: &str,
) -> Result<ElfBuildResult> {
    let total_start = Instant::now();

    // 从汇编文件推导所有文件路径
    let object_file = assembly_file.as_ref().with_extension("o");
    let executable_file = assembly_file.as_ref().with_extension("elf");
    let dump_file = assembly_file.as_ref().with_extension("dump");

    // 清理旧文件
    let mut files_to_clean = vec![
        object_file.clone(),
        executable_file.clone(),
        dump_file.clone(),
    ];

    if assembly_file
        .as_ref()
        .extension()
        .map_or(false, |ext| ext == "S")
    {
        files_to_clean.push(assembly_file.as_ref().with_extension("s"));
    }

    let mut cleaned_count = 0;
    for file in &files_to_clean {
        if file.exists() {
            fs::remove_file(file)?;
            cleaned_count += 1;
            debug!("Removed file: {}", file.display());
        }
    }

    if cleaned_count > 0 {
        info!("✅ Cleaned {} old files", cleaned_count);
    }

    // 检查汇编文件是否存在
    if !assembly_file.as_ref().exists() {
        error!(
            "Assembly file {} not found",
            assembly_file.as_ref().display()
        );
        return Err(RiscvFuzzError::file(format!(
            "Assembly file not found: {}",
            assembly_file.as_ref().display()
        )));
    }

    // 预处理 .S 文件（如果需要）
    let (assembly_to_use, preprocessed_assembly) = if assembly_file
        .as_ref()
        .extension()
        .map_or(false, |ext| ext == "S")
    {
        // 获取 gcc 兼容的 march 字符串
        let base_arch = if let Some(base_end) = arch.find('_') {
            &arch[..base_end]
        } else {
            arch
        };

        let gcc_arch = if base_arch.starts_with("rv32") {
            let extensions = &base_arch[4..];
            format!("rv32{}", filter_extensions(extensions))
        } else if base_arch.starts_with("rv64") {
            let extensions = &base_arch[4..];
            format!("rv64{}", filter_extensions(extensions))
        } else {
            "rv64id".to_string()
        };

        debug!("Original march: {}, GCC march: {}", arch, gcc_arch);

        let preprocessed_file = assembly_file.as_ref().with_extension("s");
        let output = Command::new("riscv64-unknown-elf-gcc")
            .args(&[
                &format!("-march={}", gcc_arch),
                "-E",
                assembly_file.as_ref().to_str().unwrap(),
                "-o",
                preprocessed_file.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            error!("❌ Assembly preprocessing failed");
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines().take(5) {
                error!("Preprocessing error: {}", line);
            }
            return Err(RiscvFuzzError::elf_build("preprocessing", &stderr));
        }

        info!("✅ Assembly preprocessing completed");
        (preprocessed_file.clone(), Some(preprocessed_file))
    } else {
        (assembly_file.as_ref().to_path_buf(), None)
    };

    // 汇编目标文件
    let output = Command::new("riscv64-unknown-elf-as")
        .args(&[
            &format!("-march={}", arch),
            "-g",
            "-o",
            object_file.to_str().unwrap(),
            assembly_to_use.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        error!("❌ Assembly failed");
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines().take(5) {
            error!("Assembly error: {}", line);
        }
        return Err(RiscvFuzzError::elf_build("assembly", &stderr));
    }

    info!("✅ Assembly completed");

    // 链接可执行文件
    let output = Command::new("riscv64-unknown-elf-ld")
        .args(&[
            "-T",
            linker_script.as_ref().to_str().unwrap(),
            "-o",
            executable_file.to_str().unwrap(),
            object_file.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        error!("❌ Linking failed");
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!("Linker error: {}", stderr);
        return Err(RiscvFuzzError::elf_build("linking", &stderr));
    }

    info!("✅ Linking completed");

    // 生成反汇编文件
    let output = Command::new("riscv64-unknown-elf-objdump")
        .args(&["-S", executable_file.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(RiscvFuzzError::elf_build("disassembly", &stderr));
    }

    fs::write(&dump_file, output.stdout)?;
    info!(
        "✅ Executable disassembly generated: {}",
        dump_file.display()
    );

    let total_elapsed = total_start.elapsed();

    // 一次性构建结果
    let result = ElfBuildResult {
        preprocessed_assembly,
        object_file,
        executable_file,
        disassembly_file: dump_file,
    };

    info!(
        "✅ ELF build completed successfully in {:.2}s!",
        total_elapsed.as_secs_f64(),
    );

    Ok(result)
}

fn filter_extensions(extensions: &str) -> String {
    let supported_extensions = ['i', 'm', 'a', 'f', 'd', 'c'];
    let mut result = String::new();

    for ch in extensions.chars() {
        if supported_extensions.contains(&ch) {
            result.push(ch);
        } else {
            debug!("Filtering out unsupported extension '{}' for gcc", ch);
        }
    }

    if result.is_empty() {
        result.push('i');
    }

    if !result.contains('d') {
        result.push('d');
        debug!("Added missing 'd' extension for gcc compatibility");
    }

    result
}
