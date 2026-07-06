const DELIMITER: char = '=';

pub fn parse_files_arg(arg: &str) -> (String, String) {
    match arg.split_once(DELIMITER) {
        Some((left, right)) => (left.to_string(), right.to_string()),
        None => (arg.to_string(), arg.to_string()),
    }
}

pub fn parse_exec_arg(arg: Vec<String>) -> (String, Vec<String>) {
    if let Some((program, args)) = arg.split_first() {
        (program.to_string(), args.to_vec())
    } else {
        (String::new(), Vec::<String>::new())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn files_arg_without_target_file() {
        let file_arg = String::from("/root/opt/file.yaml");
        assert_eq!(
            parse_files_arg(&file_arg),
            (
                String::from("/root/opt/file.yaml"),
                String::from("/root/opt/file.yaml")
            )
        )
    }

    #[test]
    fn files_arg_with_target_file() {
        let file_arg =
            format!("/tmp/config_template.yaml.tmp{DELIMITER}/app/configs/New-app_config.xml");
        assert_eq!(
            parse_files_arg(&file_arg),
            (
                String::from("/tmp/config_template.yaml.tmp"),
                String::from("/app/configs/New-app_config.xml")
            )
        )
    }

    #[test]
    fn files_arg_with_delimiter_only() {
        let file_arg = String::from(DELIMITER);
        assert_eq!(
            parse_files_arg(&file_arg),
            (String::from(""), String::from(""))
        )
    }

    #[test]
    fn files_arg_with_empty_string() {
        let file_arg = String::from("");
        assert_eq!(
            parse_files_arg(&file_arg),
            (String::from(""), String::from(""))
        )
    }

    #[test]
    fn exec_arg_without_args() {
        let exec_arg = vec![String::from("datetime")];
        assert_eq!(
            parse_exec_arg(exec_arg),
            (String::from("datetime"), Vec::<String>::new())
        )
    }

    #[test]
    fn exec_arg_with_args() {
        let exec_arg = vec![String::from("ls"), String::from("-la")];
        assert_eq!(
            parse_exec_arg(exec_arg),
            (String::from("ls"), vec!(String::from("-la")))
        )
    }

    #[test]
    fn exec_arg_with_empty_string() {
        let exec_arg = vec![String::new()];
        assert_eq!(
            parse_exec_arg(exec_arg),
            (String::new(), Vec::<String>::new())
        )
    }
}
