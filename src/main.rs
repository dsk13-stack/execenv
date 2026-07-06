#[cfg(not(unix))]
compile_error!("execenv is intended for Unix-like systems!");

use anyhow::{Result, ensure};
use cli::parse_cli_args;
use envsubst::envsubst;
use run_process::exec_proc;
use std::path::Path;
use utils::{parse_exec_arg, parse_files_arg};

mod cli;
mod envsubst;
mod run_process;
mod utils;

fn main() -> Result<()> {
    let cli_args = parse_cli_args();
    for file_arg in cli_args.files {
        let (init_file, result_file) = parse_files_arg(&file_arg);
        ensure!(
            !init_file.is_empty() && !result_file.is_empty(),
            "incorrect file argument `{file_arg}`",
        );
        envsubst(
            Path::new(&init_file),
            Path::new(&result_file),
            &cli_args.missing_env,
        )?;
    }
    if cli_args.exec {
        let (cmd, args) = parse_exec_arg(cli_args.cmd);
        ensure!(!cmd.is_empty(), "--exec requires a command");
        exec_proc(&cmd, &args)?;
    }
    Ok(())
}
