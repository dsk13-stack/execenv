use anyhow::{Result, bail, ensure};
use std::fmt::{self, Write};
use std::path::{Path, PathBuf};

const FILE_MAPPING_DELIMITER: char = '=';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesMapping {
    input: PathBuf,
    output: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecCommand {
    prog: String,
    args: Vec<String>,
}

impl FilesMapping {
    pub fn parse(arg: &str) -> Result<Self> {
        let (input, output) = match arg.split_once(FILE_MAPPING_DELIMITER) {
            Some((left, right)) => (left, right),
            None => (arg, arg),
        };

        ensure!(
            !input.is_empty() && !output.is_empty(),
            "incorrect file argument `{arg}`",
        );

        Ok(Self {
            input: PathBuf::from(input),
            output: PathBuf::from(output),
        })
    }

    pub fn input(&self) -> &Path {
        &self.input
    }

    pub fn output(&self) -> &Path {
        &self.output
    }
}

impl ExecCommand {
    pub fn parse(cmd_line: Vec<String>) -> Result<Self> {
        if let Some((cmd, args)) = cmd_line.split_first()
            && !cmd.is_empty()
        {
            Ok(Self {
                prog: cmd.clone(),
                args: args.to_vec(),
            })
        } else {
            bail!("--exec requires a command")
        }
    }

    pub fn prog(&self) -> &str {
        &self.prog
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }
}

impl fmt::Display for ExecCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.prog())?;
        if !self.args.is_empty() {
            f.write_char(' ')?;
            f.write_str(&self.args().join(" "))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn files_arg_without_target_file() {
        let file_arg = FilesMapping::parse("/root/opt/file.yaml").unwrap();
        assert_eq!(file_arg.input(), Path::new("/root/opt/file.yaml"));
        assert_eq!(file_arg.output(), Path::new("/root/opt/file.yaml"));
    }

    #[test]
    fn files_arg_with_target_file() {
        let file_arg = FilesMapping::parse(&format!(
            "/tmp/config_template.yaml.tmp{FILE_MAPPING_DELIMITER}/app/configs/New-app_config.xml"
        ))
        .unwrap();
        assert_eq!(file_arg.input(), Path::new("/tmp/config_template.yaml.tmp"));
        assert_eq!(
            file_arg.output(),
            Path::new("/app/configs/New-app_config.xml")
        );
    }

    #[test]
    fn files_arg_with_delimiter_only() {
        let file_arg = FilesMapping::parse(&format!("{FILE_MAPPING_DELIMITER}"));
        assert!(file_arg.is_err_and(|err| err.to_string().contains(&format!(
            "incorrect file argument `{FILE_MAPPING_DELIMITER}`"
        ))));
    }

    #[test]
    fn files_arg_with_empty_string() {
        let file_arg = FilesMapping::parse("");
        assert!(file_arg.is_err_and(|err| err.to_string().contains("incorrect file argument ``")));
    }

    #[test]
    fn exec_arg_without_args() {
        let exec_arg = ExecCommand::parse(vec![String::from("datetime")]).unwrap();
        assert_eq!(exec_arg.to_string(), String::from("datetime"))
    }

    #[test]
    fn exec_arg_with_args() {
        let exec_arg = ExecCommand::parse(vec![String::from("ls"), String::from("-la")]).unwrap();
        assert_eq!(exec_arg.to_string(), String::from("ls -la"))
    }

    #[test]
    fn exec_arg_with_empty_string() {
        let exec_arg = ExecCommand::parse(vec![String::new()]);
        assert!(exec_arg.is_err_and(|err| err.to_string().contains("--exec requires a command")))
    }

    #[test]
    fn exec_arg_with_empty_vector() {
        let exec_arg = ExecCommand::parse(Vec::new());
        assert!(exec_arg.is_err_and(|err| err.to_string().contains("--exec requires a command")))
    }
}
