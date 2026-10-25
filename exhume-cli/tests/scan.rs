use std::path::{Path, PathBuf};

use exhume::{Finding, Options, Report};
use serde::Deserialize;

/// The fields of a finding that fixtures assert on, everything except the derived Finding ID.
#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct FindingRow {
    kind: String,
    package: String,
    path: String,
    name: String,
    line: u32,
    column: u32,
    confidence: String,
    reasons: Vec<String>,
}

impl From<&Finding> for FindingRow {
    fn from(f: &Finding) -> Self {
        FindingRow {
            kind: f.kind.as_str().to_owned(),
            package: f.package.clone(),
            path: f.path.clone(),
            name: f.name.clone(),
            line: f.span.line,
            column: f.span.column,
            confidence: f.confidence.as_str().to_owned(),
            reasons: f
                .reasons
                .iter()
                .map(|r| {
                    serde_json::to_value(&r.code)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned()
                })
                .collect(),
        }
    }
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn scan(dir: &Path) -> Report {
    exhume::scan(dir, &Options::default()).expect("scan should succeed")
}

/// Asserts that the report contains exactly the findings planted in the fixture's `expected.json`.
fn assert_expected(name: &str, report: &Report) {
    let raw = std::fs::read_to_string(fixture(name).join("expected.json")).unwrap();
    let mut expected: Vec<FindingRow> = serde_json::from_str(&raw).unwrap();
    let mut actual: Vec<FindingRow> = report.findings().map(FindingRow::from).collect();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);
}

#[test]
fn unreachable_files_in_a_single_package_are_unused_files() {
    let report = scan(&fixture("single-package"));
    assert_expected("single-package", &report);
    insta::assert_json_snapshot!(report);
}

#[test]
fn a_package_whose_files_are_all_reachable_has_no_findings() {
    let report = scan(&fixture("clean-package"));
    assert_eq!(report.findings().count(), 0);
}

#[test]
fn ignore_files_above_the_workspace_root_do_not_hide_its_files() {
    let outer = tempfile::tempdir().unwrap();
    std::fs::write(outer.path().join(".gitignore"), "orphan.ts\n").unwrap();
    let workspace = outer.path().join("workspace");
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(
        workspace.join("package.json"),
        r#"{ "name": "nested", "main": "src/index.ts" }"#,
    )
    .unwrap();
    std::fs::write(workspace.join("src/index.ts"), "export {};\n").unwrap();
    std::fs::write(workspace.join("src/orphan.ts"), "export {};\n").unwrap();

    let report = scan(&workspace);

    let paths: Vec<&str> = report.findings().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, ["src/orphan.ts"]);
}
