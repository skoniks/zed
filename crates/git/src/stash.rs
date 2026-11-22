use crate::Oid;
use anyhow::{Context, Result, anyhow};
use std::{str::FromStr, sync::Arc};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StashEntry {
    pub index: usize,
    pub oid: Oid,
    pub base: Option<Oid>,
    pub message: String,
    pub branch: Option<String>,
    pub timestamp: i64,
}

impl StashEntry {
    pub fn name(&self) -> String {
        format!("stash@{{{}}}", self.index)
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct GitStash {
    pub entries: Arc<[StashEntry]>,
}

impl GitStash {
    pub fn apply(&mut self, other: GitStash) {
        self.entries = other.entries;
    }
}

impl FromStr for GitStash {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.trim().is_empty() {
            return Ok(Self::default());
        }

        let mut entries = Vec::new();
        let mut errors = Vec::new();

        for (line_num, line) in s.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            match parse_stash_line(line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    errors.push(format!("Line {}: {}", line_num + 1, e));
                }
            }
        }

        // If we have some valid entries but also some errors, log the errors but continue
        if !errors.is_empty() && !entries.is_empty() {
            log::warn!("Failed to parse some stash entries: {}", errors.join(", "));
        } else if !errors.is_empty() {
            return Err(anyhow!(
                "Failed to parse stash entries: {}",
                errors.join(", ")
            ));
        }

        Ok(Self {
            entries: entries.into(),
        })
    }
}

/// Parse a single stash line in the format: "stash@{N}\0<oid>\0<parents>\0<timestamp>\0<message>"
fn parse_stash_line(line: &str) -> Result<StashEntry> {
    let parts: Vec<&str> = line.splitn(5, '\0').collect();
    let [selector, oid, parents, timestamp, subject] = parts.as_slice() else {
        return Err(anyhow!(
            "Expected 5 null-separated parts, got {}",
            parts.len()
        ));
    };

    let index = parse_stash_index(selector)
        .with_context(|| format!("Failed to parse stash index from '{selector}'"))?;
    let oid = Oid::from_str(oid).with_context(|| format!("Failed to parse OID from '{oid}'"))?;
    // The remaining parents are snapshots of the index and, optionally, untracked files.
    let base = parents
        .split_whitespace()
        .next()
        .map(Oid::from_str)
        .transpose()
        .context("Failed to parse stash base OID")?;
    let timestamp = timestamp
        .parse::<i64>()
        .with_context(|| format!("Failed to parse timestamp from '{timestamp}'"))?;
    let (branch, message) = parse_stash_message(subject);

    Ok(StashEntry {
        index,
        oid,
        base,
        message: message.to_string(),
        branch: branch.map(Into::into),
        timestamp,
    })
}

/// Parse stash index from format "stash@{N}" where N is the index
fn parse_stash_index(input: &str) -> Result<usize> {
    let trimmed = input.trim();

    if !trimmed.starts_with("stash@{") || !trimmed.ends_with('}') {
        return Err(anyhow!(
            "Invalid stash index format: expected 'stash@{{N}}'"
        ));
    }

    let index_str = trimmed
        .strip_prefix("stash@{")
        .and_then(|s| s.strip_suffix('}'))
        .ok_or_else(|| anyhow!("Failed to extract index from stash reference"))?;

    index_str
        .parse::<usize>()
        .with_context(|| format!("Invalid stash index number: '{}'", index_str))
}

/// Parse stash message and extract branch information if present
///
/// Handles the following formats:
/// - "WIP on <branch>: <message>" -> (Some(branch), message)
/// - "On <branch>: <message>" -> (Some(branch), message)
/// - "<message>" -> (None, message)
fn parse_stash_message(input: &str) -> (Option<&str>, &str) {
    // Handle "WIP on <branch>: <message>" pattern
    if let Some(stripped) = input.strip_prefix("WIP on ")
        && let Some(colon_pos) = stripped.find(": ")
    {
        let branch = &stripped[..colon_pos];
        let message = &stripped[colon_pos + 2..];
        if !branch.is_empty() && !message.is_empty() {
            return (Some(branch), message);
        }
    }

    // Handle "On <branch>: <message>" pattern
    if let Some(stripped) = input.strip_prefix("On ")
        && let Some(colon_pos) = stripped.find(": ")
    {
        let branch = &stripped[..colon_pos];
        let message = &stripped[colon_pos + 2..];
        if !branch.is_empty() && !message.is_empty() {
            return (Some(branch), message);
        }
    }

    // Fallback: treat entire input as message with no branch
    (None, input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stash_index() {
        assert_eq!(parse_stash_index("stash@{0}").unwrap(), 0);
        assert_eq!(parse_stash_index("stash@{42}").unwrap(), 42);
        assert_eq!(parse_stash_index("  stash@{5}  ").unwrap(), 5);

        assert!(parse_stash_index("invalid").is_err());
        assert!(parse_stash_index("stash@{not_a_number}").is_err());
        assert!(parse_stash_index("stash@{0").is_err());
    }

    #[test]
    fn test_parse_stash_message() {
        // WIP format
        let (branch, message) = parse_stash_message("WIP on main: working on feature");
        assert_eq!(branch, Some("main"));
        assert_eq!(message, "working on feature");

        // On format
        let (branch, message) = parse_stash_message("On feature-branch: some changes");
        assert_eq!(branch, Some("feature-branch"));
        assert_eq!(message, "some changes");

        // No branch format
        let (branch, message) = parse_stash_message("just a regular message");
        assert_eq!(branch, None);
        assert_eq!(message, "just a regular message");

        // Edge cases
        let (branch, message) = parse_stash_message("WIP on : empty message");
        assert_eq!(branch, None);
        assert_eq!(message, "WIP on : empty message");

        let (branch, message) = parse_stash_message("On branch-name:");
        assert_eq!(branch, None);
        assert_eq!(message, "On branch-name:");
    }

    #[test]
    fn test_parse_stash_line() {
        let line =
            "stash@{0}\0abc123\x00111111 222222 333333\x001234567890\0WIP on main: test commit";
        let entry = parse_stash_line(line).unwrap();

        assert_eq!(entry.name(), "stash@{0}");
        assert_eq!(entry.base, Some("111111".parse().unwrap()));
        assert_eq!(entry.message, "test commit");
        assert_eq!(entry.branch, Some("main".to_string()));
        assert_eq!(entry.timestamp, 1234567890);
    }

    #[test]
    fn test_git_stash_from_str() {
        let input = concat!(
            "stash@{0}\0abc123\x00111111 222222\x001234567890\0WIP on main: first stash\n",
            "stash@{1}\0def456\0\x001234567891\0On feature: second stash",
        );
        let stash = GitStash::from_str(input).unwrap();

        assert_eq!(stash.entries.len(), 2);
        assert_eq!(stash.entries[0].index, 0);
        assert_eq!(stash.entries[0].branch, Some("main".to_string()));
        assert_eq!(stash.entries[1].index, 1);
        assert_eq!(stash.entries[1].base, None);
        assert_eq!(stash.entries[1].branch, Some("feature".to_string()));
    }

    #[test]
    fn test_git_stash_empty_input() {
        let stash = GitStash::from_str("").unwrap();
        assert_eq!(stash.entries.len(), 0);

        let stash = GitStash::from_str("   \n  \n  ").unwrap();
        assert_eq!(stash.entries.len(), 0);
    }
}
