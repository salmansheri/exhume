use std::path::{Path, PathBuf};

use exhume::{Options, Report};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct Expected {
    kind: String,
    package: String,
    path: String,
    #[serde(default)]
    name: Option<String>,
    confidence: String,
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn scan_fixture(name: &str) -> Report {
    exhume::scan(&fixture(name), &Options::default()).expect("scan should succeed")
}

/// Asserts that the report contains exactly the findings planted in the fixture's `expected.json`.
fn assert_expected(name: &str, report: &Report) {
    let raw = std::fs::read_to_string(fixture(name).join("expected.json")).unwrap();
    let mut expected: Vec<Expected> = serde_json::from_str(&raw).unwrap();
    let mut actual: Vec<Expected> = report
        .findings()
        .map(|f| {
            let json = serde_json::to_value(f).unwrap();
            let explicit_name = expected
                .iter()
                .any(|e| e.path == f.path && e.name.is_some());
            Expected {
                kind: json["kind"].as_str().unwrap().to_owned(),
                package: f.package.clone(),
                path: f.path.clone(),
                name: explicit_name.then(|| f.name.clone()),
                confidence: json["confidence"].as_str().unwrap().to_owned(),
            }
        })
        .collect();
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);
}

#[test]
fn unreachable_files_in_a_single_package_are_unused_files() {
    let report = scan_fixture("single-package");
    assert_expected("single-package", &report);
    insta::assert_json_snapshot!(report);
}

#[test]
fn a_package_whose_files_are_all_reachable_has_no_findings() {
    let report = scan_fixture("clean-package");
    assert_eq!(report.findings().count(), 0);
}
