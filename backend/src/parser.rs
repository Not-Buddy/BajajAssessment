use regex::Regex;
use std::collections::HashSet;

pub struct ParseResult {
    /// Valid, deduplicated (parent, child) pairs in encounter order.
    pub valid_edges: Vec<(String, String)>,
    pub invalid_entries: Vec<String>,
    /// Subsequent duplicate occurrences (each edge listed only once even if
    /// it repeats N times).
    pub duplicate_edges: Vec<String>,
}

/// Validates and deduplicates the raw input strings.
pub fn parse_entries(data: &[String]) -> ParseResult {
    // Matches exactly "X->Y" where X and Y are single uppercase ASCII letters.
    let re = Regex::new(r"^([A-Z])->([A-Z])$").unwrap();

    let mut valid_edges: Vec<(String, String)> = Vec::new();
    let mut invalid_entries: Vec<String> = Vec::new();
    let mut duplicate_edges: Vec<String> = Vec::new();

    // Track which edges we have already accepted (for dedup).
    let mut seen_edges: HashSet<String> = HashSet::new();
    // Track which edges we have already pushed to duplicate_edges (so each
    // duplicate is listed only once regardless of repetition count).
    let mut reported_duplicates: HashSet<String> = HashSet::new();

    for raw in data {
        let trimmed = raw.trim();

        // ── Validate format ──────────────────────────────────────────────────
        if let Some(caps) = re.captures(trimmed) {
            let parent = caps[1].to_string();
            let child = caps[2].to_string();

            // Self-loop is invalid
            if parent == child {
                invalid_entries.push(raw.clone());
                continue;
            }

            let edge_key = format!("{}->{}", parent, child);

            if seen_edges.contains(&edge_key) {
                // Duplicate — record only the first extra occurrence
                if !reported_duplicates.contains(&edge_key) {
                    duplicate_edges.push(edge_key.clone());
                    reported_duplicates.insert(edge_key);
                }
            } else {
                seen_edges.insert(edge_key);
                valid_edges.push((parent, child));
            }
        } else {
            // Does not match X->Y pattern
            invalid_entries.push(raw.clone());
        }
    }

    ParseResult {
        valid_edges,
        invalid_entries,
        duplicate_edges,
    }
}
