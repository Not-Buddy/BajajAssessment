use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Request ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BfhlRequest {
    pub data: Vec<String>,
}

// ── Hierarchy ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HierarchyObject {
    pub root: String,
    pub tree: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_cycle: Option<bool>,
}

// ── Summary ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_trees: usize,
    pub total_cycles: usize,
    pub largest_tree_root: String,
}

// ── Response ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BfhlResponse {
    pub user_id: String,
    pub email_id: String,
    pub college_roll_number: String,
    pub hierarchies: Vec<HierarchyObject>,
    pub invalid_entries: Vec<String>,
    pub duplicate_edges: Vec<String>,
    pub summary: Summary,
}
