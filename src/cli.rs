use clap::{Parser, ValueEnum};

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum EnvErrorMode {
    Empty,
    Error,
    Keep,
}

#[derive(Parser, Debug)]
#[command(version)]
#[command(arg_required_else_help = true)]
#[command(about = "Render environment variables into files, then optionally exec a command")]
#[command(
    long_about = r#"execenv renders `${VAR}` placeholders in one or more files using values from the current environment.

By default, each file is rewritten in place. To keep the source file unchanged and write the rendered output somewhere else, pass a mapping in the `<input>=<output>` form.

Rendering is performed before the command is executed. If any file fails to render, the command is not started.

Missing variables are controlled by `--missing-env`:
  empty  Replace missing variables with an empty string. This is the default.
  keep   Leave unresolved `${VAR}` placeholders unchanged.
  error  Fail immediately when a referenced variable is not set.

When `--exec` is passed, execenv replaces itself with the requested command using exec(2). Everything after `--exec` is treated as the command and its arguments, including values that look like flags.

You can also use only the rendering step by omitting `--exec`."#
)]
pub struct Cli {
    /// How to handle missing environment variables.
    ///
    /// Values:
    ///     empty: replace missing variables with an empty string
    ///     error: fail if a referenced variable is not set
    ///     keep:  leave unresolved placeholders unchanged
    #[arg(short, long, default_value = "empty", verbatim_doc_comment)]
    pub missing_env: EnvErrorMode,

    /// Files to render.
    ///
    /// Use a bare path to rewrite a file in place:
    ///     /app/config.yaml
    ///
    /// Use <input>=<output> to render one file into another:
    ///     /templates/config.yaml=/app/config.yaml
    #[arg(short, long, num_args = 1.., verbatim_doc_comment)]
    pub files: Vec<String>,

    /// Execute a command after all files have been rendered successfully.
    ///
    /// Everything after --exec is treated as the command and its arguments,
    /// including values that start with a hyphen.
    #[arg(short, long, verbatim_doc_comment)]
    pub exec: bool,

    #[arg(requires = "exec", trailing_var_arg = true, allow_hyphen_values = true)]
    pub cmd: Vec<String>,
}

pub fn parse_cli_args() -> Cli {
    Cli::parse()
}
