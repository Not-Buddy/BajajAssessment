use actix_web::{get, post, web, HttpResponse, Responder};
use tracing::{info, warn};
use crate::models::{BfhlRequest, BfhlResponse, Summary};
use crate::parser::parse_entries;
use crate::graph::build_hierarchies;

// ── Identity constants — update these with your real details ──────────────────
const USER_ID: &str = "aarykinge";
const EMAIL_ID: &str = "ak8098@srmist.edu.in";
const COLLEGE_ROLL_NUMBER: &str = "RA2311003020064";

#[get("/")]
pub async fn root_get_handler() -> impl Responder {
    HttpResponse::Ok().body("RUST Bajaj endpoint")
}

#[get("/bfhl")]
pub async fn bfhl_get_handler() -> impl Responder {
    HttpResponse::Ok().body("RUST Bajaj endpoint")
}

#[post("/bfhl")]
pub async fn bfhl_handler(body: web::Json<BfhlRequest>) -> impl Responder {
    let total_input = body.data.len();
    info!(total_input, "POST /bfhl received");

    // 1. Parse & validate input
    let parsed = parse_entries(&body.data);

    info!(
        valid   = parsed.valid_edges.len(),
        invalid = parsed.invalid_entries.len(),
        dupes   = parsed.duplicate_edges.len(),
        "Parsing complete"
    );

    if !parsed.invalid_entries.is_empty() {
        warn!(entries = ?parsed.invalid_entries, "Invalid entries found");
    }
    if !parsed.duplicate_edges.is_empty() {
        warn!(edges = ?parsed.duplicate_edges, "Duplicate edges found");
    }

    // 2. Build hierarchies from valid edges
    let hierarchies = build_hierarchies(&parsed.valid_edges);

    // 3. Compute summary
    let total_trees  = hierarchies.iter().filter(|h| h.has_cycle.is_none()).count();
    let total_cycles = hierarchies.iter().filter(|h| h.has_cycle.is_some()).count();

    // largest_tree_root: max depth among non-cyclic trees; lex-smaller root on tie
    let largest_tree_root = hierarchies
        .iter()
        .filter_map(|h| h.depth.map(|d| (d, &h.root)))
        .max_by(|(d1, r1), (d2, r2)| d1.cmp(d2).then(r2.cmp(r1)))
        .map(|(_, r)| r.clone())
        .unwrap_or_default();

    info!(
        trees   = total_trees,
        cycles  = total_cycles,
        largest = %largest_tree_root,
        "Response summary"
    );

    let summary = Summary { total_trees, total_cycles, largest_tree_root };

    let response = BfhlResponse {
        user_id:              USER_ID.to_string(),
        email_id:             EMAIL_ID.to_string(),
        college_roll_number:  COLLEGE_ROLL_NUMBER.to_string(),
        hierarchies,
        invalid_entries:      parsed.invalid_entries,
        duplicate_edges:      parsed.duplicate_edges,
        summary,
    };

    HttpResponse::Ok().json(response)
}
