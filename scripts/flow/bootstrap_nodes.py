#!/usr/bin/env python3
"""Bootstrap flow-graph nodes from cpp_symbol_manifest.json.

Seeds one node per manifest symbol into the correct shard under
flow_graph/nodes/<shard>.yml, merging with any existing nodes and
preserving all curated edges.  Running twice is a no-op.

Usage (from repo root):
    python3 scripts/flow/bootstrap_nodes.py
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402 (path must be set first)

_ROOT = _HERE.parent.parent


# ---------------------------------------------------------------------------
# Shard naming
# ---------------------------------------------------------------------------

def shard_name(manifest_file: str) -> str:
    """Convert a manifest file path to its shard filename.

    src/game.cpp      → game.yml
    src/game.h        → game.h.yml
    src/http/http.cpp → http__http.yml
    src/http/http.h   → http__http.h.yml
    """
    rel = manifest_file.removeprefix("src/")
    sanitized = rel.replace("/", "__")
    if sanitized.endswith(".cpp") or sanitized.endswith(".c"):
        return sanitized.rsplit(".", 1)[0] + ".yml"
    return sanitized + ".yml"


# ---------------------------------------------------------------------------
# Shard I/O
# ---------------------------------------------------------------------------

def load_shard(shard_path: Path) -> list[dict]:
    """Return nodes list from an existing shard, or [] if it does not exist."""
    if not shard_path.exists():
        return []
    try:
        doc = ledger_io.load(shard_path.read_text())
    except Exception:
        return []
    return (doc or {}).get("nodes") or []


def write_shard_if_changed(shard_path: Path, nodes: list[dict]) -> bool:
    """Serialize *nodes* to *shard_path* only if the parsed content differs.

    Returns True when the file was (re-)written.
    """
    if shard_path.exists():
        try:
            existing = ledger_io.load(shard_path.read_text())
            if (existing or {}).get("nodes") == nodes:
                return False
        except Exception:
            pass  # fall through to write
    shard_path.parent.mkdir(parents=True, exist_ok=True)
    shard_path.write_text(ledger_io.dump({"nodes": nodes}))
    return True


# ---------------------------------------------------------------------------
# Bootstrap
# ---------------------------------------------------------------------------

def _update_index_policy(root: Path) -> None:
    """Set orphan_policy: entrypoint-only in index.yml (idempotent)."""
    index_path = root / "flow_graph" / "index.yml"
    if not index_path.exists():
        return
    index = ledger_io.load(index_path.read_text()) or {}
    if index.get("orphan_policy") == "entrypoint-only":
        return  # Already set; nothing to write.
    index["orphan_policy"] = "entrypoint-only"
    index_path.write_text(ledger_io.dump(index))


def bootstrap(root: Path = _ROOT) -> None:
    """Seed all manifest symbols as nodes, preserving existing edges."""
    manifest: list[dict] = json.loads(
        (root / "cpp_symbol_manifest.json").read_text()
    )
    nodes_dir = root / "flow_graph" / "nodes"

    # Group manifest symbols by their shard file.
    by_shard: dict[str, list[dict]] = {}
    for sym in manifest:
        fname = shard_name(sym["file"])
        by_shard.setdefault(fname, []).append(sym)

    added = written = 0

    for shard_fname, symbols in sorted(by_shard.items()):
        shard_path = nodes_dir / shard_fname
        existing = load_shard(shard_path)
        existing_keys = {(n["file"], n["qualified_name"]) for n in existing}

        merged = list(existing)
        for sym in symbols:
            key = (sym["file"], sym["qualified_name"])
            if key not in existing_keys:
                merged.append({
                    "file": sym["file"],
                    "qualified_name": sym["qualified_name"],
                    "edges": [],
                })
                added += 1

        if write_shard_if_changed(shard_path, merged):
            written += 1

    # Switch orphan policy once we have more than the hand-authored boot spine.
    _update_index_policy(root)

    total = len(manifest)
    already = total - added
    print(
        f"Bootstrap: {added} nodes added, {already} already present "
        f"({total} total, {written} shard(s) written)"
    )


if __name__ == "__main__":
    bootstrap()
