use std::fmt::Write;

use serde::Serialize;

use crate::finding::{Confidence, Finding};

#[derive(Debug, Clone, Serialize)]
pub struct PackageReport {
    pub name: String,
    pub path: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub total: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub packages: Vec<PackageReport>,
    pub summary: Summary,
}

impl Report {
    pub fn new(mut packages: Vec<PackageReport>) -> Self {
        packages.sort_by(|a, b| a.path.cmp(&b.path));
        let mut summary = Summary::default();
        for package in &mut packages {
            package.findings.sort_by(|a, b| {
                (a.kind, &a.path, a.span.start, &a.name).cmp(&(
                    b.kind,
                    &b.path,
                    b.span.start,
                    &b.name,
                ))
            });
            for finding in &package.findings {
                summary.total += 1;
                match finding.confidence {
                    Confidence::High => summary.high += 1,
                    Confidence::Medium => summary.medium += 1,
                    Confidence::Low => summary.low += 1,
                }
            }
        }
        Report { packages, summary }
    }

    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        self.packages.iter().flat_map(|p| p.findings.iter())
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for package in self.packages.iter().filter(|p| !p.findings.is_empty()) {
            let _ = writeln!(out, "{} ({})", package.name, package.path);
            let mut findings = package.findings.iter().peekable();
            while let Some(first) = findings.peek() {
                let kind = first.kind;
                let group: Vec<_> =
                    std::iter::from_fn(|| findings.next_if(|f| f.kind == kind)).collect();
                let _ = writeln!(out, "  {} ({})", kind.heading(), group.len());
                for f in group {
                    let location = format!("{}:{}:{}", f.path, f.span.line, f.span.column);
                    let _ = writeln!(out, "    {location:<48} {}", f.confidence.as_str());
                }
            }
            out.push('\n');
        }
        let s = &self.summary;
        let _ = writeln!(
            out,
            "{} findings ({} high, {} medium, {} low)",
            s.total, s.high, s.medium, s.low
        );
        out
    }
}
