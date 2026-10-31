use std::path::Path;
use std::process::{Command, Output};

fn exhume(fixture: &str) -> Output {
    exhume_with(fixture, &[])
}

fn exhume_with(fixture: &str, flags: &[&str]) -> Output {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(fixture);
    Command::new(env!("CARGO_BIN_EXE_exhume"))
        .arg(dir)
        .args(flags)
        .output()
        .expect("exhume binary should run")
}

#[test]
fn exits_with_1_and_lists_unused_files_when_findings_exist() {
    let output = exhume("single-package");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(1), "stdout:\n{stdout}");
    assert!(stdout.contains("single-package (.)"), "stdout:\n{stdout}");
    assert!(stdout.contains("Unused Files (4)"), "stdout:\n{stdout}");
    assert!(stdout.contains("src/dead.ts:1:1"), "stdout:\n{stdout}");
    assert!(
        stdout.contains("4 findings (4 high, 0 medium, 0 low)"),
        "stdout:\n{stdout}"
    );
}

#[test]
fn exits_with_0_when_nothing_is_unused() {
    let output = exhume("clean-package");
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(0), "stdout:\n{stdout}");
    assert!(stdout.contains("0 findings"), "stdout:\n{stdout}");
}

#[test]
fn exits_with_2_and_explains_why_on_a_yarn_plug_n_play_workspace() {
    let output = exhume("yarn-pnp");
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(2), "stderr:\n{stderr}");
    assert!(stderr.contains("Plug'n'Play"), "stderr:\n{stderr}");
}

#[test]
fn include_public_api_flag_reports_unused_public_api() {
    let public_api_export = "packages/lib/src/index.ts:5:14";

    let default = String::from_utf8(exhume("public-api").stdout).unwrap();
    let included =
        String::from_utf8(exhume_with("public-api", &["--include-public-api"]).stdout).unwrap();

    assert!(!default.contains(public_api_export), "stdout:\n{default}");
    assert!(included.contains(public_api_export), "stdout:\n{included}");
}
