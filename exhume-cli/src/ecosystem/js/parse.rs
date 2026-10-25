//! All oxc parser calls live here, so oxc API churn is contained to one file.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Specifiers of static imports and re-exports, in source order.
pub fn imports(path: &Path, source: &str) -> Vec<String> {
    let source_type = SourceType::from_path(path).unwrap_or_default();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let mut specifiers: Vec<(u32, String)> = parsed
        .module_record
        .requested_modules
        .iter()
        .filter_map(|(specifier, requests)| {
            let first = requests.iter().map(|r| r.span.start).min()?;
            Some((first, specifier.as_str().to_owned()))
        })
        .collect();
    specifiers.sort();
    specifiers.into_iter().map(|(_, s)| s).collect()
}
