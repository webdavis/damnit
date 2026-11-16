use std::process::{Command, Stdio};

pub trait ProgramRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), String>;
}

pub struct SystemRunner;

impl ProgramRunner for SystemRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), String> {
        let output = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?;
        match output.status.success() {
            true => Ok(()),
            false => Err(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        }
    }
}
