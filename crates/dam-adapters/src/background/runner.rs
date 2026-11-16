use std::process::{Command, Stdio};

pub trait ProgramRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), RunFailure>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunFailure {
    pub code: Option<i32>,
    pub message: String,
}

pub struct SystemRunner;

impl ProgramRunner for SystemRunner {
    fn run(&self, program: &str, args: &[&str]) -> Result<(), RunFailure> {
        let output = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| RunFailure {
                code: None,
                message: e.to_string(),
            })?;
        match output.status.success() {
            true => Ok(()),
            false => Err(RunFailure {
                code: output.status.code(),
                message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            }),
        }
    }
}
