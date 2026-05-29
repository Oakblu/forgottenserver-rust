#!/usr/bin/env python3
"""Flow graph validator.

Checks:
  1. Every node {file, qualified_name} key resolves in cpp_symbol_manifest.json.
  2. No edge targets a key absent from the manifest (dangling edge).
  3. Every node is reachable from the root following out-edges, OR is listed in
     the unreached section of index.yml.

Exits 0 on success, non-zero on any violation (prints the offending item).

Usage (from repo root):
    python3 scripts/flow/validate.py
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

# Reuse the canonical YAML parser from scripts/ledger (same restricted subset).
_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402 (path must be set first)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _find_root() -> Path:
    """Walk up from this script until cpp_symbol_manifest.json is found."""
    for p in [_HERE.parent.parent, _HERE.parent, _HERE]:
        if (p / "cpp_symbol_manifest.json").exists():
            return p
    raise RuntimeError(
        "Cannot find project root: cpp_symbol_manifest.json not found "
        f"from {_HERE}"
    )


def _load_manifest(root: Path) -> set[tuple[str, str]]:
    data: list[dict] = json.loads((root / "cpp_symbol_manifest.json").read_text())
    return {(s["file"], s["qualified_name"]) for s in data}


def _load_graph(root: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    """Return (index, nodes) loaded from flow_graph/."""
    flow_dir = root / "flow_graph"
    index_text = (flow_dir / "index.yml").read_text()
    index: dict = ledger_io.load(index_text)

    nodes: list[dict] = []
    shard_dir = flow_dir / "nodes"
    for shard_path in sorted(shard_dir.glob("*.yml")):
        shard_text = shard_path.read_text()
        shard: dict = ledger_io.load(shard_text)
        if shard and "nodes" in shard:
            raw = shard["nodes"]
            if isinstance(raw, list):
                nodes.extend(raw)
    return index, nodes


def _node_key(node: dict) -> tuple[str, str]:
    return (node["file"], node["qualified_name"])


def _edge_target_key(edge: dict) -> tuple[str, str]:
    t = edge["target"]
    return (t["file"], t["qualified_name"])


# ---------------------------------------------------------------------------
# Validation
# ---------------------------------------------------------------------------

def validate(root: Path) -> list[str]:
    """Run all checks; return list of error strings (empty = OK)."""
    errors: list[str] = []
    manifest = _load_manifest(root)
    index, nodes = _load_graph(root)

    node_keys: set[tuple[str, str]] = {_node_key(n) for n in nodes}

    # 1. Every node key resolves in manifest.
    for n in nodes:
        k = _node_key(n)
        if k not in manifest:
            errors.append(
                f"UNKNOWN_NODE_KEY: {k[0]}::{k[1]!r} not in cpp_symbol_manifest.json"
            )

    # Build adjacency list (source node → list of target keys).
    adj: dict[tuple[str, str], list[tuple[str, str]]] = {
        _node_key(n): [] for n in nodes
    }

    # 2. No edge targets a missing manifest key.
    for n in nodes:
        src = _node_key(n)
        for edge in n.get("edges") or []:
            tgt = _edge_target_key(edge)
            if tgt not in manifest:
                errors.append(
                    f"DANGLING_EDGE: {src[0]}::{src[1]!r} -> "
                    f"{tgt[0]}::{tgt[1]!r} (target not in cpp_symbol_manifest.json)"
                )
            adj[src].append(tgt)

    # 3. Reachability / orphan check.
    #
    # orphan_policy in index.yml controls the scope:
    #   strict           (default) — every node must be reachable from root
    #                                or listed in unreached.
    #   entrypoint-only  — only verify that nodes declared in entrypoint_chain
    #                      are reachable; bulk-bootstrapped nodes are skipped.
    orphan_policy: str = (index.get("orphan_policy") or "strict")

    root_ref = index.get("root") or {}
    root_key: tuple[str, str] = (
        root_ref.get("file", ""),
        root_ref.get("qualified_name", ""),
    )

    unreached_raw = index.get("unreached") or []
    unreached_keys: set[tuple[str, str]] = set()
    if isinstance(unreached_raw, list):
        for u in unreached_raw:
            if isinstance(u, dict):
                unreached_keys.add((u["file"], u["qualified_name"]))

    # BFS from root through the node graph.
    reachable: set[tuple[str, str]] = set()
    if root_key in node_keys:
        queue: list[tuple[str, str]] = [root_key]
        while queue:
            curr = queue.pop()
            if curr in reachable:
                continue
            reachable.add(curr)
            for tgt in adj.get(curr, []):
                if tgt in node_keys and tgt not in reachable:
                    queue.append(tgt)

    if orphan_policy == "strict":
        # All nodes must be reachable from root or listed in unreached.
        for n in nodes:
            k = _node_key(n)
            if k not in reachable and k not in unreached_keys:
                errors.append(
                    f"ORPHAN_NODE: {k[0]}::{k[1]!r} is unreachable from root "
                    f"and not listed in unreached"
                )
    else:
        # entrypoint-only: only the declared boot-spine must be reachable.
        chain_raw = index.get("entrypoint_chain") or []
        for entry in (chain_raw if isinstance(chain_raw, list) else []):
            if not isinstance(entry, dict):
                continue
            k = (entry.get("file", ""), entry.get("qualified_name", ""))
            if k not in reachable and k not in unreached_keys:
                errors.append(
                    f"ORPHAN_NODE (entrypoint): {k[0]}::{k[1]!r} is in "
                    f"entrypoint_chain but unreachable from root"
                )

    return errors


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> int:
    try:
        root = _find_root()
    except RuntimeError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    try:
        errors = validate(root)
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        print(
            f"\n{len(errors)} validation error(s). Flow graph is invalid.",
            file=sys.stderr,
        )
        return 1

    node_count = len(
        [
            n
            for shard in (root / "flow_graph" / "nodes").glob("*.yml")
            for n in (ledger_io.load(shard.read_text()) or {}).get("nodes", [])
        ]
    )
    print(f"Flow graph: OK ({node_count} nodes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
