//! All oxc parser calls live here and in usage.rs, so oxc API churn is contained to them.

use std::collections::HashSet;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::module_record::{
    ExportEntry, ExportExportName, ExportImportName, ImportImportName,
};

use crate::ecosystem::{Export, Import, ImportedName, Module, OpaqueImport, ReExport};
use crate::finding::Span;

use super::relative_dir;
use super::usage::{self, LoadResult, NameUse};

/// The static imports, re-exports and exports of a file.
pub fn module(path: &Path, source: &str) -> Module {
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let record = &parsed.module_record;

    let mut specifiers: Vec<(u32, String)> = record
        .requested_modules
        .iter()
        .filter_map(|(specifier, requests)| {
            let first = requests.iter().map(|r| r.span.start).min()?;
            Some((first, specifier.as_str().to_owned()))
        })
        .collect();
    let file_use = usage::file_use(&parsed.program);
    specifiers.extend(
        file_use
            .loads
            .iter()
            .map(|load| (load.start, load.specifier.clone())),
    );
    specifiers.sort();
    let mut seen = HashSet::new();
    specifiers.retain(|(_, specifier)| seen.insert(specifier.clone()));

    let imports = record
        .import_entries
        .iter()
        .flat_map(|entry| {
            let used = match &entry.import_name {
                ImportImportName::Name(name) => {
                    vec![ImportedName::Name(name.name.as_str().to_owned())]
                }
                ImportImportName::Default(_) => vec![ImportedName::Name("default".to_owned())],
                ImportImportName::NamespaceObject => {
                    used_members(file_use.names.get(entry.local_name.name.as_str()))
                }
            };
            let specifier = entry.module_request.name.as_str();
            used.into_iter().map(move |name| Import {
                specifier: specifier.to_owned(),
                name,
            })
        })
        .chain(file_use.loads.iter().flat_map(|load| {
            let used = match &load.result {
                LoadResult::Discarded => Vec::new(),
                LoadResult::Members(members) => named(members),
                LoadResult::Bound(name) => used_members(file_use.names.get(name)),
                LoadResult::Escapes => vec![ImportedName::All],
            };
            used.into_iter().map(|name| Import {
                specifier: load.specifier.clone(),
                name,
            })
        }))
        .collect();

    let exports = record
        .local_export_entries
        .iter()
        .filter_map(|entry| {
            let (name, span) = export_name(&entry.export_name)?;
            Some(Export {
                name: name.to_owned(),
                span: span_at(source, span),
            })
        })
        .collect();

    // oxc also turns `import { a } from "m"; export { a }` into a re-export, keyed by the import
    // statement. Those are skipped: the import alone already marks `m`'s `a` used, so following
    // the re-export could never mark anything more.
    let import_statements: HashSet<_> = record
        .import_entries
        .iter()
        .map(|import| import.statement_span)
        .collect();
    let named = record
        .indirect_export_entries
        .iter()
        .filter(|entry| !import_statements.contains(&entry.statement_span))
        .filter_map(named_re_export);
    let stars = record.star_export_entries.iter().filter_map(|entry| {
        Some(ReExport::Star {
            specifier: entry.module_request.as_ref()?.name.as_str().to_owned(),
        })
    });

    Module {
        specifiers: specifiers.into_iter().map(|(_, s)| s).collect(),
        imports,
        exports,
        re_exports: named.chain(stars).collect(),
        opaque_imports: file_use
            .opaque_loads
            .iter()
            .map(|load| OpaqueImport {
                span: span_at(source, load.span),
                dir: load
                    .prefix
                    .as_deref()
                    .and_then(|prefix| relative_dir(path, prefix)),
            })
            .collect(),
        required: file_use
            .loads
            .iter()
            .filter(|load| load.require)
            .map(|load| load.specifier.clone())
            .collect(),
    }
}

fn named_re_export(entry: &ExportEntry) -> Option<ReExport> {
    let specifier = entry.module_request.as_ref()?.name.as_str().to_owned();
    let (exported, _) = export_name(&entry.export_name)?;
    let imported = match &entry.import_name {
        ExportImportName::Name(name) => ImportedName::Name(name.name.as_str().to_owned()),
        ExportImportName::All | ExportImportName::AllButDefault => ImportedName::All,
        ExportImportName::Null => return None,
    };
    Some(ReExport::Named {
        specifier,
        imported,
        exported: exported.to_owned(),
    })
}

/// What a namespace-like binding uses of its module: the members it reads, or everything
/// once it escapes. An unused binding uses nothing.
fn used_members(name_use: Option<&NameUse>) -> Vec<ImportedName> {
    match name_use {
        None => Vec::new(),
        Some(name_use) if name_use.escapes => vec![ImportedName::All],
        Some(name_use) => named(&name_use.members),
    }
}

fn named<'a>(members: impl IntoIterator<Item = &'a String>) -> Vec<ImportedName> {
    members
        .into_iter()
        .map(|member| ImportedName::Name(member.clone()))
        .collect()
}

/// The name an export entry exports and where that name is written.
fn export_name<'a>(name: &'a ExportExportName) -> Option<(&'a str, oxc_span::Span)> {
    match name {
        ExportExportName::Name(name) => Some((name.name.as_str(), name.span)),
        ExportExportName::Default(span) => Some(("default", *span)),
        ExportExportName::Null => None,
    }
}

/// Converts an oxc byte span into a span with a 1-based line and character column.
fn span_at(source: &str, span: oxc_span::Span) -> Span {
    let before = &source[..span.start as usize];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    Span {
        start: span.start,
        end: span.end,
        line: before.matches('\n').count() as u32 + 1,
        column: before[line_start..].chars().count() as u32 + 1,
    }
}
