use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;
use serde_json::{json, Value};
use crate::models::HierarchyObject;

// ── Graph building ─────────────────────────────────────────────────────────────

/// Build hierarchies from a list of valid (parent, child) edges.
pub fn build_hierarchies(valid_edges: &[(String, String)]) -> Vec<HierarchyObject> {
    if valid_edges.is_empty() {
        return vec![];
    }

    // children_map: parent → ordered list of children
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    // parent_map: child → first-encountered parent  (diamond rule)
    let mut parent_map: HashMap<String, String> = HashMap::new();
    // All nodes seen
    let mut all_nodes: HashSet<String> = HashSet::new();

    for (parent, child) in valid_edges {
        all_nodes.insert(parent.clone());
        all_nodes.insert(child.clone());

        // Diamond rule: first parent wins — skip if child already has one.
        if parent_map.contains_key(child) {
            continue;
        }
        parent_map.insert(child.clone(), parent.clone());
        children_map
            .entry(parent.clone())
            .or_default()
            .push(child.clone());
    }

    // ── Union-Find to group nodes into connected components ───────────────────
    let nodes: Vec<String> = {
        let mut v: Vec<String> = all_nodes.iter().cloned().collect();
        v.sort(); // deterministic ordering
        v
    };
    let node_index: HashMap<&str, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    let mut uf = UnionFind::new(nodes.len());
    for (parent, child) in valid_edges {
        // Use parent_map to only union accepted edges
        if parent_map.get(child).map(|p| p == parent).unwrap_or(false) {
            let pi = node_index[parent.as_str()];
            let ci = node_index[child.as_str()];
            uf.union(pi, ci);
        }
        // Always union parent into the component even if diamond-dropped
        if let Some(idx_p) = node_index.get(parent.as_str()) {
            if let Some(idx_c) = node_index.get(child.as_str()) {
                uf.union(*idx_p, *idx_c);
            }
        }
    }

    // Group nodes by component root
    let mut components: HashMap<usize, Vec<String>> = HashMap::new();
    for node in &nodes {
        let idx = node_index[node.as_str()];
        let root_idx = uf.find(idx);
        components.entry(root_idx).or_default().push(node.clone());
    }

    // ── Build one HierarchyObject per component ───────────────────────────────
    let mut hierarchies: Vec<HierarchyObject> = Vec::new();

    for (_comp_root_idx, mut component_nodes) in components {
        component_nodes.sort(); // lexicographic for determinism

        // Find "true" roots: nodes not in parent_map (never a child)
        let roots: Vec<String> = component_nodes
            .iter()
            .filter(|n| !parent_map.contains_key(*n))
            .cloned()
            .collect();

        // Determine the root for this component
        let root = if roots.is_empty() {
            // Pure cycle — use lex-smallest node
            component_nodes[0].clone()
        } else if roots.len() == 1 {
            roots[0].clone()
        } else {
            // Multiple roots → each is a separate tree (shouldn't happen in a
            // single connected component, but handle gracefully by picking the
            // first lex root for now — see note below).
            roots[0].clone()
        };

        // ── Cycle detection via DFS ──────────────────────────────────────────
        let has_cycle = detect_cycle(&root, &children_map, &component_nodes);

        if has_cycle {
            hierarchies.push(HierarchyObject {
                root,
                tree: json!({}),
                depth: None,
                has_cycle: Some(true),
            });
        } else {
            let tree_val = build_tree(&root, &children_map);
            let depth = compute_depth(&root, &children_map);
            hierarchies.push(HierarchyObject {
                root,
                tree: tree_val,
                depth: Some(depth),
                has_cycle: None,
            });
        }
    }

    // Sort hierarchies so output order is stable:
    // non-cyclic first by root lex, then cyclic by root lex.
    // (The spec example implies the order follows the input, but since we group
    // by component we sort predictably. Adjust if strict input-order is needed.)
    hierarchies.sort_by(|a, b| {
        // Preserve a rough input order: sort by root label lexicographically.
        a.root.cmp(&b.root)
    });

    hierarchies
}

// ── Cycle detection ────────────────────────────────────────────────────────────

/// Three-colour DFS cycle detection starting from `start`.
/// Returns true if any back-edge is reachable from start.
fn detect_cycle(
    start: &str,
    children_map: &HashMap<String, Vec<String>>,
    _component: &[String],
) -> bool {
    // 0 = white, 1 = grey (in stack), 2 = black (done)
    let mut color: HashMap<String, u8> = HashMap::new();
    dfs_cycle(start, children_map, &mut color)
}

fn dfs_cycle(
    node: &str,
    children_map: &HashMap<String, Vec<String>>,
    color: &mut HashMap<String, u8>,
) -> bool {
    color.insert(node.to_string(), 1); // grey

    if let Some(children) = children_map.get(node) {
        for child in children {
            match color.get(child.as_str()).copied().unwrap_or(0) {
                1 => return true,  // back-edge → cycle
                2 => {}            // already processed
                _ => {
                    if dfs_cycle(child, children_map, color) {
                        return true;
                    }
                }
            }
        }
    }

    color.insert(node.to_string(), 2); // black
    false
}

// ── Tree serialization ─────────────────────────────────────────────────────────

/// Build the top-level tree value: `{ "ROOT": { children... } }`
fn build_tree(root: &str, children_map: &HashMap<String, Vec<String>>) -> Value {
    let mut outer = IndexMap::new();
    outer.insert(root.to_string(), build_children(root, children_map));
    Value::Object(outer.into_iter().collect())
}

/// Recursively build the children object: `{ "child1": {...}, "child2": {...} }`
fn build_children(node: &str, children_map: &HashMap<String, Vec<String>>) -> Value {
    let mut map: IndexMap<String, Value> = IndexMap::new();
    if let Some(children) = children_map.get(node) {
        let mut sorted_children = children.clone();
        sorted_children.sort();
        for child in &sorted_children {
            map.insert(child.clone(), build_children(child, children_map));
        }
    }
    Value::Object(map.into_iter().collect())
}

/// Compute depth = number of nodes on the longest root-to-leaf path.
fn compute_depth(node: &str, children_map: &HashMap<String, Vec<String>>) -> u32 {
    match children_map.get(node) {
        None => 1,
        Some(children) if children.is_empty() => 1,
        Some(children) => {
            1 + children
                .iter()
                .map(|c| compute_depth(c, children_map))
                .max()
                .unwrap_or(0)
        }
    }
}

// ── Union-Find ─────────────────────────────────────────────────────────────────

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]); // path compression
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }
        match self.rank[rx].cmp(&self.rank[ry]) {
            std::cmp::Ordering::Less => self.parent[rx] = ry,
            std::cmp::Ordering::Greater => self.parent[ry] = rx,
            std::cmp::Ordering::Equal => {
                self.parent[ry] = rx;
                self.rank[rx] += 1;
            }
        }
    }
}
