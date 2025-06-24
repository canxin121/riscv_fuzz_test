use crate::error::{Result, RiscvFuzzError};
use std::path::{Path, PathBuf};

pub fn remove_instructions_assembly<P: AsRef<Path>>(
    assembly_file: &PathBuf,
    new_assembly_file: &PathBuf,
    removed_instructions: &[String],
) -> Result<()> {
    if removed_instructions.is_empty() {
        return Err(RiscvFuzzError::config("No illegal instructions to remove"));
    }

    let mut cleaned_assembly = String::new();
    let assembly_content = std::fs::read_to_string(assembly_file)?;

    for line in assembly_content.lines() {
        if !removed_instructions
            .iter()
            .any(|instr| line.contains(instr))
        {
            cleaned_assembly.push_str(line);
            cleaned_assembly.push('\n');
        }
    }

    std::fs::write(&new_assembly_file, cleaned_assembly)?;

    Ok(())
}
