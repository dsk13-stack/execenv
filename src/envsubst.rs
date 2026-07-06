use crate::cli::EnvErrorMode;
use anyhow::{Context, Result, bail};
use std::{
    env,
    fs::{File, remove_file, rename},
    io::{BufWriter, Read, Write},
    path::Path,
    str,
};

#[cfg(not(test))]
const READ_BUFFER_SIZE: usize = 1024 * 256;
#[cfg(not(test))]
const MAX_ENV_LEN: usize = 1024 * 4;
const ENV_SIGN: u8 = b'$';
const START_SIGN: u8 = b'{';
const END_SIGN: u8 = b'}';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanStep {
    Normal,
    FindEnvSign,
    CollectEnvName,
}

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

    substitute(input_file, &mut writer, env_error_mode, get_env)
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

pub fn substitute<R, W, F>(
    mut reader: R,
    mut writer: W,
    env_error_mode: &EnvErrorMode,
    mut resolve_env_func: F,
) -> Result<()>
where
    R: Read,
    W: Write,
    F: FnMut(&str) -> Result<String>,
{
    let mut read_buffer: [u8; READ_BUFFER_SIZE] = [0; READ_BUFFER_SIZE];
    let mut scan_step = ScanStep::Normal;
    let mut env_name_buffer = Vec::with_capacity(MAX_ENV_LEN);
    let mut out_buffer: Vec<u8> = Vec::with_capacity(READ_BUFFER_SIZE);
    let mut read_size: usize = 1;

    while read_size > 0 {
        read_size = reader.read(&mut read_buffer)?;
        for &byte in &read_buffer[..read_size] {
            match scan_step {
                ScanStep::Normal => {
                    if byte == ENV_SIGN {
                        scan_step = ScanStep::FindEnvSign;
                        continue;
                    }
                    out_buffer.push(byte);
                }
                ScanStep::FindEnvSign => {
                    if byte == START_SIGN {
                        scan_step = ScanStep::CollectEnvName;
                        continue;
                    }
                    out_buffer.push(ENV_SIGN);
                    out_buffer.push(byte);
                    scan_step = ScanStep::Normal;
                }
                ScanStep::CollectEnvName => {
                    if env_name_buffer.len() >= MAX_ENV_LEN {
                        out_buffer.push(ENV_SIGN);
                        out_buffer.push(START_SIGN);
                        out_buffer.extend_from_slice(&env_name_buffer);
                        out_buffer.push(byte);
                        env_name_buffer.clear();
                        scan_step = ScanStep::Normal;
                    } else if byte == END_SIGN {
                        let env_name = str::from_utf8(&env_name_buffer)?;
                        let restored_string =
                            process_env(env_name, env_error_mode, &mut resolve_env_func)?;
                        out_buffer.extend(restored_string.as_bytes());
                        env_name_buffer.clear();
                        scan_step = ScanStep::Normal;
                    } else {
                        env_name_buffer.push(byte);
                    }
                }
            }
        }
        writer.write_all(&out_buffer)?;
        out_buffer.clear();
    }
    flush_pending_state(scan_step, &env_name_buffer, &mut out_buffer);
    writer.write_all(&out_buffer)?;
    Ok(())
}

fn get_env(key: &str) -> Result<String> {
    let env = env::var(key);
    match env {
        Ok(val) => Ok(val),
        Err(err) => Err(err.into()),
    }
}

fn process_env<F: FnMut(&str) -> Result<String>>(
    env_name: &str,
    env_error_mode: &EnvErrorMode,
    resolve_env_func: &mut F,
) -> Result<String> {
    let result = resolve_env_func(env_name);
    match result {
        Ok(val) => Ok(val),
        Err(err) => match env_error_mode {
            EnvErrorMode::Empty => Ok(String::new()),
            EnvErrorMode::Keep => Ok(format!(
                "{}{}{}{}",
                ENV_SIGN as char, START_SIGN as char, env_name, END_SIGN as char
            )),
            EnvErrorMode::Error => bail!("{err} `{env_name}`"),
        },
    }
}

fn flush_pending_state(scan_step: ScanStep, env_name_buffer: &[u8], out_buffer: &mut Vec<u8>) {
    match scan_step {
        ScanStep::Normal => {}

        ScanStep::FindEnvSign => {
            out_buffer.push(ENV_SIGN);
        }

        ScanStep::CollectEnvName => {
            out_buffer.push(ENV_SIGN);
            out_buffer.push(START_SIGN);
            out_buffer.extend_from_slice(env_name_buffer);
        }
    }
}

#[cfg(test)]
const READ_BUFFER_SIZE: usize = 1;
#[cfg(test)]
const MAX_ENV_LEN: usize = 15;

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Cursor;

    #[test]
    fn missing_var_empty_mode() {
        let mock_reader = b"This var must be empty: ${EMPTY_VAR}";
        let mut mock_writer = Cursor::new(Vec::new());
        let mock_get_env = |key: &str| {
            if key.eq("SUBST_VAR") {
                Ok(String::from("value"))
            } else {
                bail!("Not found")
            }
        };
        substitute(
            &mock_reader[..],
            &mut mock_writer,
            &EnvErrorMode::Empty,
            mock_get_env,
        )
        .unwrap();
        let written_data = mock_writer.into_inner();
        assert_eq!(written_data, b"This var must be empty: ");
    }

    #[test]
    fn missing_var_error_mode() {
        let mock_reader = b"This must be an error: ${UNDECLARED_VAR}";
        let mut mock_writer = Cursor::new(Vec::new());
        let mock_get_env = |key: &str| {
            if key.eq("SUBST_VAR") {
                Ok(String::from("value"))
            } else {
                bail!("Not found")
            }
        };
        let result = substitute(
            &mock_reader[..],
            &mut mock_writer,
            &EnvErrorMode::Error,
            mock_get_env,
        );
        assert!(result.is_err_and(|err| { err.to_string().contains("`UNDECLARED_VAR`") }));
    }

    #[test]
    fn missing_var_keep_mode() {
        let mock_reader = b"This var must be the same: ${SAME_VAR}";
        let mut mock_writer = Cursor::new(Vec::new());
        let mock_get_env = |key: &str| {
            if key.eq("SUBST_VAR") {
                Ok(String::from("value"))
            } else {
                bail!("Not found")
            }
        };
        substitute(
            &mock_reader[..],
            &mut mock_writer,
            &EnvErrorMode::Keep,
            mock_get_env,
        )
        .unwrap();
        let written_data = mock_writer.into_inner();
        assert_eq!(written_data, b"This var must be the same: ${SAME_VAR}");
    }

    #[test]
    fn multibyte_utf8_survives_tiny_reads() {
        let mock_reader = "привет ${MY_VAR} € 🎉 конец".as_bytes();
        let mut mock_writer = Cursor::new(Vec::new());
        let mock_get_env = |key: &str| {
            if key.eq("SUBST_VAR") {
                Ok(String::from("value"))
            } else {
                bail!("Not found")
            }
        };
        substitute(
            mock_reader,
            &mut mock_writer,
            &EnvErrorMode::Keep,
            mock_get_env,
        )
        .unwrap();
        let written_data = mock_writer.into_inner();
        assert_eq!(written_data, "привет ${MY_VAR} € 🎉 конец".as_bytes());
    }

    #[test]
    fn normal_substitution() {
        let mock_reader = b"This var must be substitute: ${SUBST_VAR}";
        let mut mock_writer = Cursor::new(Vec::new());
        let mock_get_env = |key: &str| {
            if key.eq("SUBST_VAR") {
                Ok(String::from("value"))
            } else {
                bail!("Not found")
            }
        };
        substitute(
            &mock_reader[..],
            &mut mock_writer,
            &EnvErrorMode::Keep,
            mock_get_env,
        )
        .unwrap();
        let written_data = mock_writer.into_inner();
        assert_eq!(written_data, b"This var must be substitute: value");
    }

    #[test]
    fn overflow_env_name_preserved() {
        let mock_reader = b"Long var: ${VEEEEEEERY_LOOOONG_VAAAAAR}";
        let mut mock_writer = Cursor::new(Vec::new());
        substitute(
            &mock_reader[..],
            &mut mock_writer,
            &EnvErrorMode::Error,
            get_env,
        )
        .unwrap();
        let written_data = mock_writer.into_inner();
        assert_eq!(written_data, b"Long var: ${VEEEEEEERY_LOOOONG_VAAAAAR}");
    }

    #[test]
    fn trailing_dollar_at_eof_is_preserved() {
        let mock_reader = b"This text must be the same: $";
        let mut mock_writer = Cursor::new(Vec::new());
        substitute(
            &mock_reader[..],
            &mut mock_writer,
            &EnvErrorMode::Keep,
            get_env,
        )
        .unwrap();
        let written_data = mock_writer.into_inner();
        assert_eq!(written_data, b"This text must be the same: $");
    }

    #[test]
    fn unterminated_brace_at_eof_is_preserved() {
        let mock_reader = b"This text must be the same: ${SUBST_VA";
        let mut mock_writer = Cursor::new(Vec::new());
        substitute(
            &mock_reader[..],
            &mut mock_writer,
            &EnvErrorMode::Keep,
            get_env,
        )
        .unwrap();
        let written_data = mock_writer.into_inner();
        assert_eq!(written_data, b"This text must be the same: ${SUBST_VA");
    }
}
