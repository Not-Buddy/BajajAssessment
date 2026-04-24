#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# POST /bfhl  —  Test Suite
# Run:  bash test.sh
# Requires: curl, python3 (for pretty-print)
# ─────────────────────────────────────────────────────────────────────────────

BASE="http://localhost:8080/bfhl"
PASS=0
FAIL=0

run_test() {
  local label="$1"
  local payload="$2"
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  TEST: $label"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  curl -s -X POST "$BASE" \
    -H 'Content-Type: application/json' \
    -d "$payload" | python3 -m json.tool
}

assert_field() {
  # assert_field <label> <payload> <jq_path> <expected_value>
  local label="$1" payload="$2" path="$3" expected="$4"
  local actual
  actual=$(curl -s -X POST "$BASE" \
    -H 'Content-Type: application/json' \
    -d "$payload" | python3 -c "
import sys, json
data = json.load(sys.stdin)
keys = \"$path\".split('.')
v = data
for k in keys:
    if k.lstrip('-').isdigit():
        v = v[int(k)]
    else:
        v = v[k]
print(json.dumps(v))
" 2>/dev/null)

  if [ "$actual" = "$expected" ]; then
    echo "  ✅  PASS — $label  →  $path = $actual"
    PASS=$((PASS + 1))
  else
    echo "  ❌  FAIL — $label"
    echo "       path:     $path"
    echo "       expected: $expected"
    echo "       got:      $actual"
    FAIL=$((FAIL + 1))
  fi
}

# ─────────────────────────────────────────────────────────────────────────────
# 1. SPEC EXAMPLE  (full round-trip pretty-print)
# ─────────────────────────────────────────────────────────────────────────────
run_test "Spec example (full output)" \
  '{"data":["A->B","A->C","B->D","C->E","E->F","X->Y","Y->Z","Z->X","P->Q","Q->R","G->H","G->H","G->I","hello","1->2","A->"]}'

# ─────────────────────────────────────────────────────────────────────────────
# 2. ASSERTION TESTS  (automated pass/fail)
# ─────────────────────────────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════════════════════"
echo "  ASSERTION CHECKS"
echo "════════════════════════════════════════════════════════════════════════"

# ── 2a. Summary fields from spec example ─────────────────────────────────────
SPEC='{"data":["A->B","A->C","B->D","C->E","E->F","X->Y","Y->Z","Z->X","P->Q","Q->R","G->H","G->H","G->I","hello","1->2","A->"]}'

assert_field "Spec: total_trees"        "$SPEC" "summary.total_trees"      "3"
assert_field "Spec: total_cycles"       "$SPEC" "summary.total_cycles"     "1"
assert_field "Spec: largest_tree_root"  "$SPEC" "summary.largest_tree_root" '"A"'
assert_field "Spec: invalid count"      "$SPEC" "invalid_entries"          '["hello", "1->2", "A->"]'
assert_field "Spec: duplicate_edges"    "$SPEC" "duplicate_edges"          '["G->H"]'
assert_field "Spec: tree A depth"       "$SPEC" "hierarchies.0.depth"      "4"

# ── 2b. Invalid entries ───────────────────────────────────────────────────────
INVALIDS='{"data":["hello","1->2","AB->C","A-B","A->","","A->A"," ","a->b","A->BC"]}'
assert_field "All-invalid: no valid trees"   "$INVALIDS" "summary.total_trees"  "0"
assert_field "All-invalid: no cycles"        "$INVALIDS" "summary.total_cycles" "0"
assert_field "All-invalid: hierarchies empty" "$INVALIDS" "hierarchies"          "[]"

# ── 2c. Self-loop goes to invalid_entries ────────────────────────────────────
SELFLOOP='{"data":["A->A"]}'
assert_field "Self-loop: in invalid_entries" "$SELFLOOP" "invalid_entries" '["A->A"]'
assert_field "Self-loop: no hierarchies"     "$SELFLOOP" "hierarchies"     "[]"

# ── 2d. Whitespace trimming ───────────────────────────────────────────────────
WHITESPACE='{"data":["  A->B  ","   C->D   "]}'
assert_field "Whitespace trim: total_trees"     "$WHITESPACE" "summary.total_trees"    "2"
assert_field "Whitespace trim: no invalids"     "$WHITESPACE" "invalid_entries"        "[]"

# ── 2e. Triple duplicate  →  only ONE entry in duplicate_edges ───────────────
TRIPLE='{"data":["A->B","A->B","A->B"]}'
assert_field "Triple dup: duplicate_edges length" "$TRIPLE" "duplicate_edges" '["A->B"]'
assert_field "Triple dup: tree still built"       "$TRIPLE" "summary.total_trees" "1"

# ── 2f. Pure cycle — lex-smallest root, no depth field ───────────────────────
PURE_CYCLE='{"data":["B->A","A->B"]}'
assert_field "Pure cycle: has_cycle"  "$PURE_CYCLE" "hierarchies.0.has_cycle" "true"
assert_field "Pure cycle: root is A (lex-smallest)" "$PURE_CYCLE" "hierarchies.0.root" '"A"'
assert_field "Pure cycle: tree is {}" "$PURE_CYCLE" "hierarchies.0.tree"      "{}"

# ── 2g. Longer cycle (3-node) ────────────────────────────────────────────────
CYCLE3='{"data":["X->Y","Y->Z","Z->X"]}'
assert_field "3-node cycle: has_cycle"  "$CYCLE3" "hierarchies.0.has_cycle" "true"
assert_field "3-node cycle: tree is {}" "$CYCLE3" "hierarchies.0.tree"      "{}"

# ── 2h. Diamond — second parent edge silently dropped ────────────────────────
DIAMOND='{"data":["A->D","B->D","A->E"]}'
# B->D is dropped (D already has parent A). B has no edges left so it
# does not appear in the graph at all. Only A's tree survives: total_trees = 1.
assert_field "Diamond: total_trees = 1 (B vanishes)" "$DIAMOND" "summary.total_trees" "1"
assert_field "Diamond: A depth = 2"                  "$DIAMOND" "hierarchies.0.depth" "2"

# ── 2i. Single node chain depth ──────────────────────────────────────────────
CHAIN='{"data":["A->B","B->C","C->D","D->E"]}'
assert_field "Chain depth = 5"  "$CHAIN" "hierarchies.0.depth" "5"

# ── 2j. Tiebreak — equal depth, lex-smaller root wins largest_tree_root ──────
TIE='{"data":["A->B","C->D"]}'
# Both trees have depth 2; A < C → largest_tree_root = "A"
assert_field "Tiebreak: largest_tree_root = A" "$TIE" "summary.largest_tree_root" '"A"'

# ── 2k. Multiple independent trees ───────────────────────────────────────────
MULTI='{"data":["A->B","C->D","E->F","G->H"]}'
assert_field "Multi-tree: total_trees = 4" "$MULTI" "summary.total_trees" "4"

# ── 2l. Empty data array ─────────────────────────────────────────────────────
EMPTY='{"data":[]}'
assert_field "Empty data: no hierarchies"  "$EMPTY" "hierarchies"    "[]"
assert_field "Empty data: total_trees = 0" "$EMPTY" "summary.total_trees" "0"

# ── 2m. Mix of cycle and non-cycle in same request ───────────────────────────
MIXED='{"data":["A->B","B->C","X->Y","Y->X"]}'
assert_field "Mixed: total_trees = 1"  "$MIXED" "summary.total_trees"  "1"
assert_field "Mixed: total_cycles = 1" "$MIXED" "summary.total_cycles" "1"
assert_field "Mixed: largest = A"      "$MIXED" "summary.largest_tree_root" '"A"'

# ── 2n. Deeply nested tree ───────────────────────────────────────────────────
DEEP='{"data":["A->B","B->C","C->D","D->E","E->F","F->G","G->H"]}'
assert_field "Deep tree depth = 8" "$DEEP" "hierarchies.0.depth" "8"

# ── 2o. Wide tree (many children of one root) ────────────────────────────────
WIDE='{"data":["A->B","A->C","A->D","A->E","A->F"]}'
assert_field "Wide tree: depth = 2"       "$WIDE" "hierarchies.0.depth"     "2"
assert_field "Wide tree: total_trees = 1" "$WIDE" "summary.total_trees"     "1"

# ─────────────────────────────────────────────────────────────────────────────
# 3. EXTRA PRETTY-PRINT CASES  (for visual inspection)
# ─────────────────────────────────────────────────────────────────────────────
run_test "Diamond — B->D dropped, B becomes lone root" \
  '{"data":["A->D","B->D","A->E"]}'

run_test "Mixed cycle + tree" \
  '{"data":["A->B","B->C","X->Y","Y->X"]}'

run_test "Deep chain A→B→…→H" \
  '{"data":["A->B","B->C","C->D","D->E","E->F","F->G","G->H"]}'

run_test "Tiebreak: two trees both depth 2, root A wins" \
  '{"data":["A->B","C->D"]}'

run_test "All invalid inputs" \
  '{"data":["hello","1->2","AB->C","A-B","A->","","A->A","a->b"]}'

# ─────────────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════════════════════"
printf "  Results: %d passed, %d failed\n" "$PASS" "$FAIL"
echo "════════════════════════════════════════════════════════════════════════"
[ "$FAIL" -eq 0 ] && exit 0 || exit 1
