use crate::cli::EnvErrorMode;
use anyhow::{Context, Result};
use std::{
    env,
    fs::{File, remove_file, rename},
    io::{BufWriter, Write},
    path::Path,
    str,
};

mod parser;

pub fn envsubst(
    env_file_path: &Path,
    result_file_path: &Path,
    env_error_mode: &EnvErrorMode,
) -> Result<()> {
    let tmp_file_path = result_file_path.with_added_extension("execenv_tmp");
    let input_file = File::open(env_file_path)
        .with_context(|| format!("failed to open input file `{}`", env_file_path.display()))?;
    let tmp_file = File::create(&tmp_file_path).with_context(|| {
        format!(
            "failed to create temporary file `{}`",
            tmp_file_path.display()
        )
    })?;
    tmp_file.set_permissions(input_file.metadata()?.permissions())?;
    let mut writer = BufWriter::new(tmp_file);

    parser::substitute(input_file, &mut writer, env_error_mode, get_env)
        .and_then(|_| {
            writer
                .flush()
                .with_context(|| format!("failed to flush `{}`", tmp_file_path.display()))
        })
        .and_then(|_| {
            rename(&tmp_file_path, result_file_path).with_context(|| {
                format!(
                    "failed to rename `{}` to `{}`",
                    tmp_file_path.display(),
                    result_file_path.display()
                )
            })
        })
        .inspect_err(|_| {
            let _ = remove_file(&tmp_file_path);
        })
}

fn get_env(key: &str) -> Result<String> {
    let env = env::var(key);
    match env {
        Ok(val) => Ok(val),
        Err(err) => Err(err.into()),
    }
}
