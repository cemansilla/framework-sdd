//! Parser for `.sdd/changes/CHANGELOG.md`.
//!
//! The changelog is plain markdown. Entries are `## [ID] <date> — <title>`
//! headings, or `### [ID] …` headings nested under `## [Unreleased]`
//! (Keep a Changelog convention), followed by optional `- **Key**: value`
//! field lines (see spec §20: identificador, fecha, motivo, origen,
//! artefactos afectados, impacto, tareas afectadas).

/// One parsed changelog entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangelogEntry {
    /// Identifier between brackets, e.g. `CHG-001` or `TASK-FW-200`.
    pub id: String,
    /// ISO-8601 date (`YYYY-MM-DD`) when present in the heading.
    pub date: Option<String>,
    /// Human readable title (text after the date/separator).
    pub title: String,
    /// `- **Key**: value` field lines in document order.
    pub fields: Vec<(String, String)>,
}

impl ChangelogEntry {
    /// Look up a field value by key, ignoring ASCII case.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// Parse changelog markdown into entries, preserving document order.
///
/// Headings without a `[ID]` marker and the template `## [Unreleased]`
/// section marker are ignored.
pub fn parse_changelog(markdown: &str) -> Vec<ChangelogEntry> {
    let mut entries: Vec<ChangelogEntry> = Vec::new();
    let mut current: Option<ChangelogEntry> = None;

    for line in markdown.lines() {
        // Level-2 (`## [ID]`) and level-3 (`### [ID]`, e.g. nested under
        // `## [Unreleased]`) headings both start a new entry.
        let heading = line
            .strip_prefix("### [")
            .or_else(|| line.strip_prefix("## ["));
        if let Some(rest) = heading {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            if let Some(entry) = parse_heading(rest) {
                current = Some(entry);
            }
            continue;
        }

        let Some(entry) = current.as_mut() else {
            continue;
        };

        if let Some((key, value)) = parse_field(line) {
            entry.fields.push((key, value));
        }
    }

    if let Some(entry) = current.take() {
        entries.push(entry);
    }

    entries
}

/// Parse the part of a `## [ID]`/`### [ID]` heading that follows the bracket.
///
/// Returns `None` for the `[Unreleased]` section marker.
fn parse_heading(rest: &str) -> Option<ChangelogEntry> {
    let close = rest.find(']')?;
    let id = rest[..close].trim();
    if id.is_empty() || id.eq_ignore_ascii_case("Unreleased") {
        return None;
    }

    let after_id = &rest[close + 1..];
    let (date, after_date) = match extract_date(after_id) {
        Some((date, tail)) => (Some(date.to_string()), tail),
        None => (None, after_id),
    };

    let title = after_date
        .trim_start_matches(|c: char| c == '—' || c == '-' || c == ':' || c.is_whitespace())
        .trim()
        .to_string();

    Some(ChangelogEntry {
        id: id.to_string(),
        date,
        title,
        fields: Vec::new(),
    })
}

/// If `s` starts with an ISO date (`YYYY-MM-DD`), return it and the rest.
fn extract_date(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if s.len() < 10 {
        return None;
    }
    let (head, tail) = s.split_at(10);
    let bytes = head.as_bytes();
    let is_digits = |range: std::ops::Range<usize>| bytes[range].iter().all(u8::is_ascii_digit);
    if bytes[4] == b'-' && bytes[7] == b'-' && is_digits(0..4) && is_digits(5..7) && is_digits(8..10)
    {
        Some((head, tail))
    } else {
        None
    }
}

/// Parse a `- **Key**: value` line.
fn parse_field(line: &str) -> Option<(String, String)> {
    let rest = line.trim().strip_prefix("- **")?;
    let end = rest.find("**:")?;
    let key = rest[..end].trim();
    if key.is_empty() {
        return None;
    }
    let value = rest[end + 3..].trim();
    Some((key.to_string(), value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entry_with_date_and_fields() {
        let markdown = r#"# Changelog

## [Unreleased]

## [CHG-001] 2026-09-26 — Traceable changelog entries

- **Motivo**: Spec §20 requires full traceability fields.
- **Origen**: change
- **Impacto**: Parser and CLI output
- **Tareas afectadas**: TASK-FW-201
"#;
        let entries = parse_changelog(markdown);
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert_eq!(entry.id, "CHG-001");
        assert_eq!(entry.date.as_deref(), Some("2026-09-26"));
        assert_eq!(entry.title, "Traceable changelog entries");
        assert_eq!(entry.fields.len(), 4);
        assert_eq!(
            entry.field("motivo"),
            Some("Spec §20 requires full traceability fields.")
        );
        assert_eq!(entry.field("IMPACTO"), Some("Parser and CLI output"));
        assert_eq!(entry.field("missing"), None);
    }

    #[test]
    fn test_parse_entry_without_date() {
        let markdown = "## [TASK-FW-200] — Changelog command reads disk\n";
        let entries = parse_changelog(markdown);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "TASK-FW-200");
        assert_eq!(entries[0].date, None);
        assert_eq!(entries[0].title, "Changelog command reads disk");
    }

    #[test]
    fn test_unreleased_marker_is_skipped() {
        let entries = parse_changelog("## [Unreleased]\n");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_non_bracket_headings_are_ignored() {
        let markdown = "# Title\n\n## Plain section\n\nSome text.\n";
        assert!(parse_changelog(markdown).is_empty());
    }

    #[test]
    fn test_malformed_lines_are_ignored() {
        let markdown = "## [CHG-002] 2026-09-26 — Title\n\n- plain bullet\n- **no terminator\nnot a field\n";
        let entries = parse_changelog(markdown);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].fields.is_empty());
    }

    #[test]
    fn test_multiple_entries_preserve_order() {
        let markdown = "## [CHG-002] 2026-01-02 — Second\n\n## [CHG-001] 2026-01-01 — First\n";
        let entries = parse_changelog(markdown);
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["CHG-002", "CHG-001"]);
    }

    #[test]
    fn test_nested_entry_under_unreleased() {
        let markdown = "## [Unreleased]\n\n### [CHG-001] 2026-09-26 — Nested entry\n\n- **Origen**: change\n";
        let entries = parse_changelog(markdown);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "CHG-001");
        assert_eq!(entries[0].title, "Nested entry");
        assert_eq!(entries[0].field("origen"), Some("change"));
    }

    #[test]
    fn test_mixed_heading_levels_preserve_order() {
        let markdown = "## [CHG-002] 2026-01-02 — Level two\n\n## [Unreleased]\n\n### [CHG-001] 2026-01-01 — Level three\n";
        let entries = parse_changelog(markdown);
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(ids, vec!["CHG-002", "CHG-001"]);
    }

    #[test]
    fn test_empty_input() {
        assert!(parse_changelog("").is_empty());
        assert!(parse_changelog("# Changelog\n").is_empty());
    }

    #[test]
    fn test_heading_without_closing_bracket_is_ignored() {
        assert!(parse_changelog("## [CHG-001 broken\n").is_empty());
    }
}
