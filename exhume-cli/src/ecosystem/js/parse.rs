//! All oxc parser calls live here, so oxc API churn is contained to one file.

use std::collections::HashSet;
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::module_record::{
    ExportEntry, ExportExportName, ExportImportName, ImportImportName,
};

use crate::ecosystem::{Export, Import, ImportedName, Module, ReExport};
use crate::finding::Span;

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
    specifiers.sort();

    let imports = record
        .import_entries
        .iter()
        .map(|entry| Import {
            specifier: entry.module_request.name.as_str().to_owned(),
            name: match &entry.import_name {
                ImportImportName::Name(name) => ImportedName::Name(name.name.as_str().to_owned()),
                ImportImportName::Default(_) => ImportedName::Name("default".to_owned()),
                ImportImportName::NamespaceObject => ImportedName::All,
            },
        })
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
