#[cfg(not(unix))]
compile_error!("execenv is intended for Unix-like systems!");

use anyhow::Result;
use args::{ExecCommand, FilesMapping};
use cli::parse_cli_args;
use envsubst::envsubst;
use run_process::exec_proc;

mod args;
mod cli;
mod envsubst;
mod run_process;

fn main() -> Result<()> {
    let cli_args = parse_cli_args();
    for file_arg in cli_args.files {
        let file_mapping = FilesMapping::parse(&file_arg)?;
        envsubst(
            file_mapping.input(),
            file_mapping.output(),
            &cli_args.missing_env,
        )?;
    }
    if cli_args.exec {
        let command = ExecCommand::parse(cli_args.cmd)?;
        exec_proc(command.prog(), command.args())?;
    }
    Ok(())
}
