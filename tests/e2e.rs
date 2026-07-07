use assert_cmd::Command;
use pretty_assertions::assert_eq;
use std::fs;
use tempfile::tempdir;

fn create_cmd() -> Command {
    Command::cargo_bin("execenv").unwrap()
}

#[test]
fn subst_env_in_place_and_exec() {
    let mut command = create_cmd();

    let text = "${NAME} was here. Briefly.";
    let result_text = "Brian was here. Briefly.";
    let dir = tempdir().unwrap();
    let input_file_path = dir.path().join("conf.tmp");

    fs::write(&input_file_path, text).unwrap();

    let assert = command
        .args([
            "--files",
            input_file_path.to_str().unwrap(),
            "--exec",
            "/bin/true",
        ])
        .env("NAME", "Brian")
        .assert();

    assert.success();
    assert_eq!(result_text, fs::read_to_string(input_file_path).unwrap());
}

#[test]
fn subst_env_in_output_file_and_exec() {
    let mut command = create_cmd();

    let text = "${NAME} was here. Briefly.";
    let result_text = "Brian was here. Briefly.";
    let dir = tempdir().unwrap();
    let input_file_path = dir.path().join("conf.tmp");
    let output_file_path = dir.path().join("conf.txt");
    let mapping = format!(
        "{}={}",
        input_file_path.display(),
        output_file_path.display()
    );
    fs::write(&input_file_path, text).unwrap();

    let assert = command
        .args(["--files", &mapping, "--exec", "/bin/true"])
        .env("NAME", "Brian")
        .assert();

    assert.success();
    assert_eq!(result_text, fs::read_to_string(output_file_path).unwrap());
    assert_eq!(text, fs::read_to_string(input_file_path).unwrap());
}

#[test]
fn render_error_aborts_exec() {
    let mut command = create_cmd();

    let text = "${MISSING} was here. Briefly.";
    let dir = tempdir().unwrap();
    let input_file_path = dir.path().join("conf.tmp");
    fs::write(&input_file_path, text).unwrap();

    let assert = command
        .args([
            "--files",
            input_file_path.to_str().unwrap(),
            "--missing-env",
            "error",
            "--exec",
            "/bin/true",
        ])
        .assert();

    assert
        .failure()
        .stderr(predicates::str::contains("MISSING"));
    assert_eq!(text, fs::read_to_string(input_file_path).unwrap());
}

#[test]
fn subst_env_in_multiple_files_in_place() {
    let mut command = create_cmd();

    let text_1 = "${NAME_1} was here. Briefly.";
    let result_text_1 = "Brian was here. Briefly.";
    let text_2 = "${NAME_2} was here. Briefly.";
    let result_text_2 = "Alex was here. Briefly.";

    let dir = tempdir().unwrap();
    let input_file_path_1 = dir.path().join("conf_1.txt");
    let input_file_path_2 = dir.path().join("conf_2.txt");

    fs::write(&input_file_path_1, text_1).unwrap();
    fs::write(&input_file_path_2, text_2).unwrap();

    let assert = command
        .args([
            "--files",
            input_file_path_1.to_str().unwrap(),
            input_file_path_2.to_str().unwrap(),
            "--exec",
            "/bin/true",
        ])
        .env("NAME_1", "Brian")
        .env("NAME_2", "Alex")
        .assert();

    assert.success();
    assert_eq!(
        result_text_1,
        fs::read_to_string(input_file_path_1).unwrap()
    );
    assert_eq!(
        result_text_2,
        fs::read_to_string(input_file_path_2).unwrap()
    );
}
