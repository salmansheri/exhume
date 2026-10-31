//! Discovers the Workspace Packages of a JS workspace from `pnpm-workspace.yaml` or the
//! root manifest's `workspaces` field.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::Deserialize;

use super::manifest::Manifest;
use crate::ecosystem::WorkspacePackage;

#[derive(Deserialize)]
struct PnpmWorkspace {
    packages: Option<Vec<String>>,
}

/// Every Workspace Package, the root first, each paired with its manifest.
pub fn discover(root: &Path) -> anyhow::Result<Vec<(WorkspacePackage, Manifest)>> {
    let mut packages = Vec::new();
    let mut dirs_by_name: HashMap<String, PathBuf> = HashMap::new();
    for (dir, manifest) in member_dirs(root)? {
        let name = manifest.name.clone().unwrap_or_else(|| {
            dir.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        });
        // Imports name packages, so a duplicate name would make resolution ambiguous.
        if let Some(first) = dirs_by_name.insert(name.clone(), dir.clone()) {
            bail!(
                "two Workspace Packages are named {name:?}: {} and {}",
                first.display(),
                dir.display()
            );
        }
        packages.push((
            WorkspacePackage {
                name,
                dir,
                private: manifest.private,
            },
            manifest,
        ));
    }
    Ok(packages)
}

fn member_dirs(root: &Path) -> anyhow::Result<Vec<(PathBuf, Manifest)>> {
    reject_plug_n_play(root)?;
    let root_manifest = Manifest::read_optional(root)?;
    let patterns = match pnpm_patterns(root)? {
        Some(patterns) => patterns,
        None => root_manifest
            .as_ref()
            .map(Manifest::workspace_patterns)
            .unwrap_or_default(),
    };
    let mut packages = Vec::new();
    match root_manifest {
        Some(manifest) => packages.push((root.to_path_buf(), manifest)),
        None if patterns.is_empty() => {
            bail!(
                "no package.json found at {}",
                root.join("package.json").display()
            )
        }
        None => {}
    }
    if patterns.is_empty() {
        return Ok(packages);
    }

    let (include, exclude) = compile(&patterns)?;
    let mut members: Vec<PathBuf> = crate::walk::files(root, super::SKIPPED_DIRS)
        .filter(|path| path.file_name().is_some_and(|n| n == "package.json"))
        .filter_map(|path| path.parent().map(Path::to_path_buf))
        .filter(|dir| {
            let relative = dir.strip_prefix(root).unwrap_or(dir);
            !relative.as_os_str().is_empty()
                && include.is_match(relative)
                && !exclude.is_match(relative)
        })
        .collect();
    members.sort();
    for dir in members {
        let manifest = Manifest::read(&dir)?;
        packages.push((dir, manifest));
    }
    Ok(packages)
}

fn pnpm_patterns(root: &Path) -> anyhow::Result<Option<Vec<String>>> {
    let path = root.join("pnpm-workspace.yaml");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Ok(None);
    };
    let workspace: PnpmWorkspace =
        serde_saphyr::from_str(&raw).with_context(|| format!("invalid {}", path.display()))?;
    Ok(workspace.packages)
}

/// Splits workspace patterns into included and `!`-negated globs.
fn compile(patterns: &[String]) -> anyhow::Result<(GlobSet, GlobSet)> {
    let mut include = GlobSetBuilder::new();
    let mut exclude = GlobSetBuilder::new();
    for pattern in patterns {
        let (builder, pattern) = match pattern.strip_prefix('!') {
            Some(negated) => (&mut exclude, negated),
            None => (&mut include, pattern.as_str()),
        };
        let pattern = pattern.trim_start_matches("./").trim_end_matches('/');
        let glob = GlobBuilder::new(pattern)
            .literal_separator(true)
            .build()
            .with_context(|| format!("invalid workspace pattern {pattern:?}"))?;
        builder.add(glob);
    }
    Ok((include.build()?, exclude.build()?))
}

/// Exhume resolves packages through `node_modules`, which Plug'n'Play replaces.
fn reject_plug_n_play(root: &Path) -> anyhow::Result<()> {
    let has_pnp_runtime = [".pnp.cjs", ".pnp.js"]
        .iter()
        .any(|f| root.join(f).is_file());
    let configured = std::fs::read_to_string(root.join(".yarnrc.yml")).is_ok_and(|raw| {
        raw.lines().any(|line| {
            let line = line.trim().replace(['"', '\''], "");
            line.strip_prefix("nodeLinker:")
                .is_some_and(|v| v.trim() == "pnp")
        })
    });
    if has_pnp_runtime || configured {
        bail!(
            "Yarn Plug'n'Play (PnP) is not supported; set `nodeLinker: node-modules` in .yarnrc.yml \
             and reinstall"
        );
    }
    Ok(())
}
