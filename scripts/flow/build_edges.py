#!/usr/bin/env python3
"""Extract static call edges from the read-only C++ source tree.

Reads forgottenserver-upstream/src/ and derives kind:static,
confidence:static out-edges for each callable function body, then
merges them into the flow_graph/nodes/*.yml shards while preserving
all kind:dynamic and confidence:curated edges.  Running twice with no
source change produces no diff (idempotent).

Static extractor: heuristic regex (see O1 note in flow_graph/README.md).

Usage (from repo root):
    python3 scripts/flow/build_edges.py
"""
from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

from bootstrap_nodes import shard_name, load_shard, write_shard_if_changed  # noqa: E402

_ROOT = _HERE.parent.parent
_UPSTREAM = _ROOT / "forgottenserver-upstream"

# ---------------------------------------------------------------------------
# Callable symbol kinds (have a function body worth scanning)
# ---------------------------------------------------------------------------

_CALLABLE_KINDS: frozenset[str] = frozenset({
    "method",
    "static_function",
    "free_function",
    "function",
    "member_function",
    "constructor",
    "destructor",
    "virtual_method",
})

# ---------------------------------------------------------------------------
# Manifest index
# ---------------------------------------------------------------------------

def build_manifest_index(manifest: list[dict]) -> dict[str, list[dict]]:
    """Map qualified_name → entries, preferring .cpp over .h definitions."""
    by_qn: dict[str, list[dict]] = defaultdict(list)
    for s in manifest:
        by_qn[s["qualified_name"]].append(s)

    resolved: dict[str, list[dict]] = {}
    for qn, entries in by_qn.items():
        cpp = [e for e in entries if e["file"].endswith(".cpp")]
        resolved[qn] = cpp if cpp else entries
    return resolved


# ---------------------------------------------------------------------------
# Global variable type extraction (e.g. Game g_game; → g_game: Game)
# ---------------------------------------------------------------------------

# Matches file-scope declarations like:
#   Game g_game;    DatabaseTasks g_databaseTasks;    extern Scripts* g_scripts;
_GLOBAL_DECL = re.compile(
    r"^(?:extern\s+)?([A-Z][A-Za-z0-9_<>:]*)\s*\*?\s+(g_[A-Za-z0-9_]+)\s*[=;]"
)


def extract_global_types(source_lines: list[str]) -> dict[str, str]:
    """Return {g_var: ClassName} from file-scope global declarations."""
    type_map: dict[str, str] = {}
    for line in source_lines:
        m = _GLOBAL_DECL.match(line.strip())
        if m:
            type_map[m.group(2)] = m.group(1)
    return type_map


# ---------------------------------------------------------------------------
# Call-site patterns
# ---------------------------------------------------------------------------

# Qualified calls: Foo::bar(  or  ns::Foo::bar(  or  tfs::io::func(
# (requires at least one :: so we avoid single-word false positives)
_QUALIFIED_CALL = re.compile(
    r"\b([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+)"
    r"\s*(?:<[^>()]{0,64}>)?\s*\("
)

# Member/arrow calls on tracked g_ globals: g_foo.method(  or  g_foo->method(
_MEMBER_CALL = re.compile(
    r"\b(g_[A-Za-z0-9_]+)\s*(?:->|\.)\s*([A-Za-z_][A-Za-z0-9_]*)\s*\("
)


def extract_call_targets(
    body: str,
    manifest_index: dict[str, list[dict]],
    global_types: dict[str, str],
) -> list[tuple[str, str]]:
    """Return sorted, deduplicated list of (file, qualified_name) targets."""
    targets: set[tuple[str, str]] = set()

    for m in _QUALIFIED_CALL.finditer(body):
        qn = m.group(1)
        for entry in manifest_index.get(qn, []):
            targets.add((entry["file"], entry["qualified_name"]))

    for m in _MEMBER_CALL.finditer(body):
        var_name, method_name = m.group(1), m.group(2)
        class_name = global_types.get(var_name)
        if class_name:
            qn = f"{class_name}::{method_name}"
            for entry in manifest_index.get(qn, []):
                targets.add((entry["file"], entry["qualified_name"]))

    return sorted(targets)


# ---------------------------------------------------------------------------
# Edge merge
# ---------------------------------------------------------------------------

def merge_edges(
    existing_edges: list[dict],
    new_static_targets: list[tuple[str, str]],
) -> list[dict]:
    """Rebuild edge list:
    - Preserve kind:dynamic edges (any confidence).
    - Preserve kind:static, confidence:curated edges (hand-authored).
    - Replace kind:static, confidence:static edges with freshly derived ones.
    - Skip auto-static targets already covered by a curated edge.
    """
    preserved = [
        e for e in existing_edges
        if e.get("kind") != "static" or e.get("confidence") == "curated"
    ]

    # Targets already covered by curated (or dynamic) edges.
    covered: set[tuple[str, str]] = set()
    for e in preserved:
        tgt = e.get("target")
        if isinstance(tgt, dict):
            covered.add((tgt.get("file", ""), tgt.get("qualified_name", "")))

    order = len(preserved) + 1
    for (f, qn) in new_static_targets:
        if (f, qn) not in covered:
            preserved.append({
                "target": {"file": f, "qualified_name": qn},
                "kind": "static",
                "confidence": "static",
                "order": order,
            })
            order += 1

    return preserved


# ---------------------------------------------------------------------------
# Main build
# ---------------------------------------------------------------------------

def build(root: Path = _ROOT) -> None:
    manifest: list[dict] = json.loads(
        (root / "cpp_symbol_manifest.json").read_text()
    )
    manifest_index = build_manifest_index(manifest)
    nodes_dir = root / "flow_graph" / "nodes"
    upstream_src = root / "forgottenserver-upstream" / "src"

    # Group callable symbols by .cpp file (headers have no body to scan).
    by_cpp: dict[str, list[dict]] = defaultdict(list)
    for sym in manifest:
        if sym.get("kind") in _CALLABLE_KINDS and sym["file"].endswith(".cpp"):
            by_cpp[sym["file"]].append(sym)

    total_fns = sum(len(v) for v in by_cpp.values())
    fns_with_edges = 0
    total_static_edges = 0
    files_missing = 0

    for cpp_file, symbols in sorted(by_cpp.items()):
        # Source path: strip 'src/' prefix and look under upstream root.
        rel_path = cpp_file.removeprefix("src/")
        source_path = upstream_src / rel_path
        if not source_path.exists():
            files_missing += 1
            continue

        source_lines = source_path.read_text(errors="replace").splitlines()
        global_types = extract_global_types(source_lines)

        # Load (or stub) the existing shard.
        shard_path = nodes_dir / shard_name(cpp_file)
        existing_nodes = load_shard(shard_path)

        # Build a mutable node map keyed by (file, qn).
        node_map: dict[tuple[str, str], dict] = {
            (n["file"], n["qualified_name"]): n for n in existing_nodes
        }

        shard_modified = False

        for sym in symbols:
            key = (sym["file"], sym["qualified_name"])
            node = node_map.get(key)
            if node is None:
                # Shouldn't happen after bootstrap, but handle gracefully.
                node = {
                    "file": sym["file"],
                    "qualified_name": sym["qualified_name"],
                    "edges": [],
                }
                node_map[key] = node
                existing_nodes.append(node)

            line_start = sym.get("line_start") or 0
            line_end = sym.get("line_end") or line_start
            if not line_start:
                continue

            body = "\n".join(source_lines[line_start - 1 : line_end])
            targets = extract_call_targets(body, manifest_index, global_types)

            if not targets:
                continue

            original_edges = node.get("edges") or []
            merged = merge_edges(original_edges, targets)

            if merged != original_edges:
                node["edges"] = merged
                shard_modified = True
                fns_with_edges += 1
                total_static_edges += sum(
                    1 for e in merged if e.get("confidence") == "static"
                )
            else:
                fns_with_edges += bool(any(
                    e.get("confidence") == "static" for e in original_edges
                ))

        if shard_modified:
            write_shard_if_changed(shard_path, existing_nodes)

    # Coverage report
    pct = 100.0 * fns_with_edges / total_fns if total_fns else 0.0
    print(
        f"Static edges: {total_static_edges} new/updated edges across "
        f"{fns_with_edges}/{total_fns} callable functions ({pct:.1f}% coverage)"
    )
    if files_missing:
        print(f"  ({files_missing} source file(s) not found under forgottenserver-upstream/)")


if __name__ == "__main__":
    build()
