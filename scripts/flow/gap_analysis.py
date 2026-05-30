#!/usr/bin/env python3
"""Gap analysis engine for the flow graph.

Traverses the flow graph from the root, joins each reachable node against
MIGRATION_LEDGER.yml and rust_symbol_manifest.json, classifies findings,
suppresses intentional differences, and returns a prioritized report.

Finding categories:
  MISSING_FLOW   — reachable node whose Rust counterpart is absent or stubbed
  DYNAMIC_GAP    — dynamic edge (opcode/event/virtual) targeting a
                   missing/stubbed Rust handler
  BRANCH_GAP     — conditional edge whose entire target chain is unimplemented
  ORDER_MISMATCH — boot/init `order` values with no known Rust counterpart
                   (low-confidence, review-only)

Usage (from repo root):
    python3 scripts/flow/gap_analysis.py
"""
from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

_ROOT = _HERE.parent.parent

# Priority weights (D4 in design.md)
_W_DEPTH = 0.4
_W_STATUS = 0.4
_W_CRIT = 0.2

# Boot-spine / high-criticality node names
_BOOT_SPINE: frozenset[str] = frozenset({
    "main", "startServer", "mainLoader",
})
_HIGH_CRIT: frozenset[str] = frozenset({
    "ProtocolGame::parsePacket", "Connection::parsePacket",
    "ServiceManager::run", "Game::start",
    "ProtocolGame::onRecvFirstMessage", "ProtocolStatus::onRecvFirstMessage",
})

# Status weights for priority formula
_STATUS_WEIGHT = {
    "missing": 1.0,       # not in ledger at all
    "stub": 0.9,          # exists but empty body_hash
    "dynamic_target": 0.8,
    "branch_target": 0.7,
    "order_issue": 0.4,
}

NodeKey = tuple[str, str]  # (file, qualified_name)


# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------

@dataclass
class Finding:
    category: str           # MISSING_FLOW | DYNAMIC_GAP | BRANCH_GAP | ORDER_MISMATCH
    priority: float
    cpp_file: str
    qualified_name: str
    ledger_status: str      # "missing", "migrated", "migrated_with_changes", "intentionally_removed", …
    rust_symbol: str        # best-known Rust equivalent or ""
    depth: int              # BFS depth from root
    condition: str          # edge condition triggering the finding (for BRANCH_GAP/DYNAMIC_GAP)
    note: str               # human-readable detail

    def as_dict(self) -> dict:
        return {
            "category": self.category,
            "priority": round(self.priority, 4),
            "cpp_file": self.cpp_file,
            "qualified_name": self.qualified_name,
            "ledger_status": self.ledger_status,
            "rust_symbol": self.rust_symbol,
            "depth": self.depth,
            "condition": self.condition,
            "note": self.note,
        }


# ---------------------------------------------------------------------------
# Data loading helpers
# ---------------------------------------------------------------------------

def _load_graph(root: Path) -> dict[NodeKey, dict]:
    """Load all YAML shards; return {(file, qname): node}."""
    nodes: dict[NodeKey, dict] = {}
    nodes_dir = root / "flow_graph" / "nodes"
    for yml in sorted(nodes_dir.glob("*.yml")):
        doc = ledger_io.load(yml.read_text()) or {}
        for n in (doc.get("nodes") or []):
            key: NodeKey = (n["file"], n["qualified_name"])
            nodes[key] = n
    return nodes


def _load_index(root: Path) -> NodeKey:
    doc = ledger_io.load((root / "flow_graph" / "index.yml").read_text()) or {}
    r = doc["root"]
    return (r["file"], r["qualified_name"])


def _load_ledger(root: Path) -> dict[str, dict]:
    """Return {cpp_symbol: ledger_entry} (keyed by qualified_name only)."""
    doc = ledger_io.load((root / "MIGRATION_LEDGER.yml").read_text()) or {}
    index: dict[str, dict] = {}
    for s in (doc.get("symbols") or []):
        sym = s["cpp"]["symbol"]
        index[sym] = s
    return index


def _load_rust_manifest(root: Path) -> dict[str, dict]:
    """Return {qualified_name: entry}."""
    data = json.loads((root / "rust_symbol_manifest.json").read_text())
    return {e["qualified_name"]: e for e in data}


def _load_intentional_diffs(root: Path) -> frozenset[str]:
    """Return set of cpp_symbol values from intentional_differences.yml."""
    try:
        text = (root / "intentional_differences.yml").read_text()
        # ledger_io may fail on odd indentation; fall back to regex extraction
        try:
            doc = ledger_io.load(text) or {}
            diffs = doc.get("differences") or []
            return frozenset(d.get("cpp_symbol", "") for d in diffs)
        except Exception:
            matches = re.findall(r"cpp_symbol:\s*[\"']?([^\"'\n]+)[\"']?", text)
            return frozenset(m.strip() for m in matches)
    except FileNotFoundError:
        return frozenset()


# ---------------------------------------------------------------------------
# BFS traversal
# ---------------------------------------------------------------------------

def bfs(root_key: NodeKey, graph: dict[NodeKey, dict]) -> dict[NodeKey, int]:
    """Return {node_key: depth} for all nodes reachable from root_key."""
    from collections import deque
    visited: dict[NodeKey, int] = {}
    queue: deque[tuple[NodeKey, int]] = deque([(root_key, 0)])
    while queue:
        key, depth = queue.popleft()
        if key in visited:
            continue
        visited[key] = depth
        node = graph.get(key)
        if not node:
            continue
        for edge in (node.get("edges") or []):
            tgt = edge.get("target", {})
            tkey: NodeKey = (tgt.get("file", ""), tgt.get("qualified_name", ""))
            if tkey not in visited and tkey[1]:
                queue.append((tkey, depth + 1))
    return visited


# ---------------------------------------------------------------------------
# Priority calculation
# ---------------------------------------------------------------------------

def _criticality(qname: str) -> float:
    if qname in _BOOT_SPINE:
        return 1.0
    if qname in _HIGH_CRIT:
        return 0.8
    return 0.5


def _priority(depth: int, status_weight: float, qname: str) -> float:
    return (
        _W_DEPTH * (1.0 / (1.0 + depth))
        + _W_STATUS * status_weight
        + _W_CRIT * _criticality(qname)
    )


# ---------------------------------------------------------------------------
# Finding checks
# ---------------------------------------------------------------------------

def _rust_symbol_for(ledger_entry: dict | None) -> str:
    """Best Rust qualified_name from a ledger entry, or empty string."""
    if not ledger_entry:
        return ""
    for r in (ledger_entry.get("rust") or []):
        qn = r.get("qualified_name") or r.get("symbol") or ""
        if qn:
            return qn
    return ""


def _is_stub(rust_qname: str, rust_manifest: dict[str, dict]) -> bool:
    """True if the Rust manifest entry for this symbol has no body_hash."""
    if not rust_qname:
        return True
    entry = rust_manifest.get(rust_qname)
    if not entry:
        return True
    # Non-code kinds (module, field, enum_variant) don't have bodies
    kind = entry.get("kind", "")
    if kind in ("module", "field", "enum_variant", "enum", "struct", "trait",
                "type_alias", "constant", "static"):
        return False
    return not bool(entry.get("body_hash", ""))


# ---------------------------------------------------------------------------
# Main analysis function
# ---------------------------------------------------------------------------

def analyze(root: Path = _ROOT) -> tuple[list[Finding], list[Finding]]:
    """Return (actionable_findings, suppressed_findings) both sorted by priority desc."""
    graph = _load_graph(root)
    root_key = _load_index(root)
    ledger = _load_ledger(root)
    rust_manifest = _load_rust_manifest(root)
    intentional_syms = _load_intentional_diffs(root)

    reachable = bfs(root_key, graph)

    findings: list[Finding] = []
    suppressed: list[Finding] = []
    seen: set[tuple[str, str, str]] = set()  # (category, file, qname)

    def _add(f: Finding) -> None:
        key = (f.category, f.cpp_file, f.qualified_name)
        if key in seen:
            return
        seen.add(key)
        sym = f.qualified_name
        # Suppress if symbol or a parent scope matches intentional differences
        if any(sym in idiff or idiff in sym for idiff in intentional_syms if idiff):
            suppressed.append(f)
        else:
            findings.append(f)

    for node_key, depth in reachable.items():
        cpp_file, qname = node_key
        node = graph.get(node_key)
        ledger_entry = ledger.get(qname)
        ledger_status = ledger_entry["status"] if ledger_entry else "missing"
        rust_qname = _rust_symbol_for(ledger_entry)

        # Intentionally removed symbols are not gaps
        if ledger_status == "intentionally_removed":
            continue

        # 1. MISSING_FLOW: no ledger entry or stub Rust impl
        if ledger_status == "missing":
            _add(Finding(
                category="MISSING_FLOW",
                priority=_priority(depth, _STATUS_WEIGHT["missing"], qname),
                cpp_file=cpp_file,
                qualified_name=qname,
                ledger_status=ledger_status,
                rust_symbol="",
                depth=depth,
                condition="",
                note="Symbol not found in MIGRATION_LEDGER.yml",
            ))
        elif _is_stub(rust_qname, rust_manifest):
            _add(Finding(
                category="MISSING_FLOW",
                priority=_priority(depth, _STATUS_WEIGHT["stub"], qname),
                cpp_file=cpp_file,
                qualified_name=qname,
                ledger_status=ledger_status,
                rust_symbol=rust_qname,
                depth=depth,
                condition="",
                note=f"Rust symbol {rust_qname!r} has no body (stub or unimplemented)",
            ))

        if not node:
            continue

        # 2. DYNAMIC_GAP / BRANCH_GAP from edges
        for edge in (node.get("edges") or []):
            kind = edge.get("kind", "")
            confidence = edge.get("confidence", "")
            condition = edge.get("condition", "") or ""
            tgt = edge.get("target", {})
            tfile = tgt.get("file", "")
            tqname = tgt.get("qualified_name", "")

            if not tqname:
                continue

            tgt_ledger = ledger.get(tqname)
            tgt_ledger_status = tgt_ledger["status"] if tgt_ledger else "missing"
            if tgt_ledger_status == "intentionally_removed":
                continue
            tgt_rust = _rust_symbol_for(tgt_ledger)
            tgt_is_gap = tgt_ledger_status == "missing" or _is_stub(tgt_rust, rust_manifest)

            if not tgt_is_gap:
                continue

            if kind == "dynamic" and confidence == "curated":
                _add(Finding(
                    category="DYNAMIC_GAP",
                    priority=_priority(depth, _STATUS_WEIGHT["dynamic_target"], qname),
                    cpp_file=cpp_file,
                    qualified_name=qname,
                    ledger_status=ledger_status,
                    rust_symbol=rust_qname,
                    depth=depth,
                    condition=condition,
                    note=(
                        f"Dynamic edge → {tqname!r} "
                        f"(Rust: {tgt_rust or 'missing'})"
                    ),
                ))
            elif condition and kind == "static":
                _add(Finding(
                    category="BRANCH_GAP",
                    priority=_priority(depth, _STATUS_WEIGHT["branch_target"], qname),
                    cpp_file=cpp_file,
                    qualified_name=qname,
                    ledger_status=ledger_status,
                    rust_symbol=rust_qname,
                    depth=depth,
                    condition=condition,
                    note=(
                        f"Branch guard {condition!r} → {tqname!r} "
                        f"(Rust: {tgt_rust or 'missing'})"
                    ),
                ))

        # 3. ORDER_MISMATCH (low-confidence): check ordered edges on boot-spine nodes
        ordered_edges = sorted(
            (e for e in (node.get("edges") or []) if e.get("order") is not None),
            key=lambda e: e.get("order", 0),
        )
        if ordered_edges and qname in _BOOT_SPINE:
            for edge in ordered_edges:
                tqname_o = edge.get("target", {}).get("qualified_name", "")
                tgt_l = ledger.get(tqname_o)
                tgt_status = tgt_l["status"] if tgt_l else "missing"
                if tgt_status == "intentionally_removed":
                    continue
                tgt_rust_o = _rust_symbol_for(tgt_l)
                if _is_stub(tgt_rust_o, rust_manifest):
                    _add(Finding(
                        category="ORDER_MISMATCH",
                        priority=_priority(depth, _STATUS_WEIGHT["order_issue"], qname),
                        cpp_file=cpp_file,
                        qualified_name=qname,
                        ledger_status=ledger_status,
                        rust_symbol=rust_qname,
                        depth=depth,
                        condition=f"order={edge['order']}",
                        note=(
                            f"Boot-spine {qname!r} calls {tqname_o!r} at order "
                            f"{edge['order']} but Rust counterpart is unverified "
                            f"(low-confidence)"
                        ),
                    ))

    findings.sort(key=lambda f: f.priority, reverse=True)
    suppressed.sort(key=lambda f: f.priority, reverse=True)
    return findings, suppressed


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> int:
    try:
        findings, suppressed = analyze()
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        import traceback; traceback.print_exc(file=sys.stderr)
        return 1

    n = len(findings)
    s = len(suppressed)
    by_cat: dict[str, int] = {}
    for f in findings:
        by_cat[f.category] = by_cat.get(f.category, 0) + 1

    print(f"Gap analysis: {n} actionable finding(s), {s} suppressed")
    for cat, count in sorted(by_cat.items()):
        print(f"  {cat}: {count}")
    if findings:
        print("\nTop 5:")
        for f in findings[:5]:
            print(f"  [{f.category}] pri={f.priority:.3f} depth={f.depth} "
                  f"{f.qualified_name!r} ({f.cpp_file})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
