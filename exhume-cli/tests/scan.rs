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
                    serde_json::to_value(r.code)
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
    assert_expected_in(name, "expected.json", report);
}

/// Like `assert_expected`, for a fixture that plants different findings for different options.
fn assert_expected_in(name: &str, expected_file: &str, report: &Report) {
    let raw = std::fs::read_to_string(fixture(name).join(expected_file)).unwrap();
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

fn package_names(report: &Report) -> Vec<&str> {
    report.packages.iter().map(|p| p.name.as_str()).collect()
}

#[test]
fn pnpm_workspace_packages_are_discovered_and_resolved_by_name_and_alias() {
    let report = scan(&fixture("pnpm-monorepo"));
    assert_eq!(
        package_names(&report),
        ["pnpm-monorepo", "web", "@acme/ui", "@acme/utils"]
    );
    assert_expected("pnpm-monorepo", &report);
}

#[test]
fn workspaces_field_in_array_form_discovers_every_package() {
    let report = scan(&fixture("npm-workspaces"));
    assert_eq!(package_names(&report), ["npm-workspaces", "app", "lib"]);
    assert_expected("npm-workspaces", &report);
}

#[test]
fn workspaces_field_in_object_form_discovers_every_package() {
    let report = scan(&fixture("yarn-workspaces"));
    assert_eq!(package_names(&report), ["yarn-workspaces", "core"]);
    assert_expected("yarn-workspaces", &report);
}

#[test]
fn yarn_plug_n_play_workspaces_are_rejected() {
    let error = exhume::scan(&fixture("yarn-pnp"), &Options::default()).unwrap_err();
    assert!(
        format!("{error:#}").contains("Yarn Plug'n'Play (PnP) is not supported"),
        "{error:#}"
    );
}

#[test]
fn yarn_plug_n_play_is_rejected_from_yarnrc_before_install() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("package.json"), r#"{ "name": "fresh" }"#).unwrap();
    std::fs::write(dir.path().join(".yarnrc.yml"), "nodeLinker: \"pnp\"\n").unwrap();

    let error = exhume::scan(dir.path(), &Options::default()).unwrap_err();

    assert!(format!("{error:#}").contains("Plug'n'Play"), "{error:#}");
}

#[test]
fn a_pnpm_workspace_file_without_packages_falls_back_to_the_workspaces_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("pnpm-workspace.yaml"),
        "catalog:\n  react: ^19\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{ "name": "root", "workspaces": ["pkgs/*"] }"#,
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("pkgs/one")).unwrap();
    std::fs::write(
        dir.path().join("pkgs/one/package.json"),
        r#"{ "name": "one" }"#,
    )
    .unwrap();

    let report = scan(dir.path());

    assert_eq!(package_names(&report), ["root", "one"]);
}

#[test]
fn two_workspace_packages_with_the_same_name_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("package.json"),
        r#"{ "name": "root", "workspaces": ["a", "b"] }"#,
    )
    .unwrap();
    for member in ["a", "b"] {
        std::fs::create_dir_all(dir.path().join(member)).unwrap();
        std::fs::write(
            dir.path().join(member).join("package.json"),
            r#"{ "name": "dup" }"#,
        )
        .unwrap();
    }

    let error = exhume::scan(dir.path(), &Options::default()).unwrap_err();

    assert!(
        format!("{error:#}").contains("two Workspace Packages are named \"dup\""),
        "{error:#}"
    );
}

#[test]
fn exports_of_reachable_files_that_nothing_imports_are_unused_exports() {
    let report = scan(&fixture("barrels"));
    assert_expected("barrels", &report);
}

#[test]
fn type_only_and_default_exports_are_used_and_reported_like_named_ones() {
    let report = scan(&fixture("export-kinds"));
    assert_expected("export-kinds", &report);
}

#[test]
fn namespace_member_access_counts_only_the_members_used_and_an_escaping_namespace_uses_all() {
    let report = scan(&fixture("namespaces"));
    assert_expected("namespaces", &report);
}

#[test]
fn literal_require_and_dynamic_import_are_imports() {
    let report = scan(&fixture("lazy"));
    assert_expected("lazy", &report);
}

#[test]
fn a_non_literal_dynamic_import_lowers_the_findings_it_could_load() {
    let report = scan(&fixture("dynamic-imports"));
    assert_expected("dynamic-imports", &report);
}

#[test]
fn findings_behind_a_cjs_boundary_or_an_unresolvable_specifier_are_lowered() {
    let report = scan(&fixture("cjs-and-unresolved"));
    assert_expected("cjs-and-unresolved", &report);
}

#[test]
fn an_unresolvable_path_alias_lowers_the_findings_of_its_package() {
    let report = scan(&fixture("unresolved-alias"));
    assert_expected("unresolved-alias", &report);
}

#[test]
fn exports_reachable_from_a_public_packages_entry_points_are_public_api() {
    let report = scan(&fixture("public-api"));
    assert_expected("public-api", &report);
}

#[test]
fn include_public_api_reports_public_api_exports_too() {
    let options = Options {
        include_public_api: true,
    };
    let report = exhume::scan(&fixture("public-api"), &options).expect("scan should succeed");
    assert_expected_in("public-api", "expected-include-public-api.json", &report);
}

#[test]
fn a_public_tag_marks_an_export_used_and_an_ignore_comment_suppresses_the_next_line() {
    let report = scan(&fixture("in-code-ignores"));
    assert_expected("in-code-ignores", &report);
}

#[test]
fn declared_dependencies_that_no_file_of_their_package_imports_are_unused() {
    let report = scan(&fixture("dependencies"));
    assert_expected("dependencies", &report);
}

#[test]
fn dependencies_run_through_their_installed_binaries_in_scripts_are_used() {
    let report = scan(&fixture("scripts"));
    assert_expected("scripts", &report);
}
