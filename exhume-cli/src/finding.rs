use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    UnusedFile,
}

impl FindingKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FindingKind::UnusedFile => "unused-file",
        }
    }

    pub fn heading(self) -> &'static str {
        match self {
            FindingKind::UnusedFile => "Unused Files",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        }
    }
}

/// Why a finding's confidence is below High. Codes are added as blind spots are recognised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReasonCode {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reason {
    pub code: ReasonCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

impl Span {
    pub const FILE_START: Span = Span {
        start: 0,
        end: 0,
        line: 1,
        column: 1,
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub id: String,
    pub kind: FindingKind,
    pub package: String,
    pub path: String,
    pub name: String,
    pub span: Span,
    pub confidence: Confidence,
    pub reasons: Vec<Reason>,
}

impl Finding {
    pub fn new(kind: FindingKind, package: &str, path: &str, name: &str, span: Span) -> Self {
        Finding {
            id: finding_id(kind, package, path, name),
            kind,
            package: package.to_owned(),
            path: path.to_owned(),
            name: name.to_owned(),
            span,
            confidence: Confidence::High,
            reasons: Vec::new(),
        }
    }
}

/// A stable identity that ignores line numbers, so baselines survive edits elsewhere in a file.
/// FNV-1a is used because std's hasher is not guaranteed stable across Rust releases.
fn finding_id(kind: FindingKind, package: &str, path: &str, name: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for part in [kind.as_str(), package, path, name] {
        for byte in part.bytes().chain(std::iter::once(0)) {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}
