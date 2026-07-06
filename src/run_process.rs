use anyhow::{Context, Result};
use std::{os::unix::process::CommandExt, process::Command};

pub fn exec_proc(cmd: &str, args: &[String]) -> Result<()> {
    let err = Command::new(cmd).args(args).exec();

    Err(err).with_context(|| format!("failed to execute command `{} {}`", cmd, args.join(" ")))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn failed_to_execute() {
        let cmd = String::from("no_command");
        let args = Vec::new();
        let result = exec_proc(&cmd, &args);
        assert!(result.is_err_and(|err| err.to_string().contains("failed to execute command")))
    }
}