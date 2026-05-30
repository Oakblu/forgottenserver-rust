#!/usr/bin/env python3
"""Generate flow_graph/FLOW_GRAPH.md from the YAML node shards.

Produces:
  1. Boot sequence — the ordered entrypoint chain from index.yml as a
     numbered list showing the call path from main through mainLoader.
  2. Per-shard node table — one section per .yml file listing every node
     with its edge count and outbound edge targets.

Usage (from repo root):
    python3 scripts/flow/render_markdown.py
"""
from __future__ import annotations

import re
import sys
from collections import defaultdict
from datetime import date
from pathlib import Path

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

_ROOT = _HERE.parent.parent
_OUT = _ROOT / "flow_graph" / "FLOW_GRAPH.md"


def _load_index(root: Path) -> dict:
    return ledger_io.load((root / "flow_graph" / "index.yml").read_text()) or {}


def _load_shard(path: Path) -> list[dict]:
    doc = ledger_io.load(path.read_text()) or {}
    return list(doc.get("nodes") or [])


def _edge_summary(edges: list[dict]) -> str:
    if not edges:
        return "—"
    static = sum(1 for e in edges if e.get("kind") == "static")
    dynamic = sum(1 for e in edges if e.get("kind") == "dynamic")
    curated = sum(1 for e in edges if e.get("confidence") == "curated")
    parts = []
    if static:
        parts.append(f"{static} static")
    if dynamic:
        parts.append(f"{dynamic} dynamic")
    if curated:
        parts.append(f"{curated} curated")
    return ", ".join(parts)


def _shard_label(shard_file: str) -> str:
    """Human-readable label from shard filename."""
    name = Path(shard_file).stem
    # Remove .h suffix for .h.yml shards
    if name.endswith(".h"):
        return f"`{name}` (header declarations)"
    return f"`{name}`"


def generate(root: Path = _ROOT) -> Path:
    out_path = root / "flow_graph" / "FLOW_GRAPH.md"
    index = _load_index(root)
    nodes_dir = root / "flow_graph" / "nodes"

    lines: list[str] = []
    lines.append("# Flow Graph")
    lines.append("")
    lines.append(f"_Generated {date.today()} — do not edit by hand. Re-run `make flow-render`._")
    lines.append("")
    lines.append(
        "This document is generated from the YAML node shards in `flow_graph/nodes/`. "
        "Each section corresponds to one shard file. "
        "Nodes are C++ functions/methods; edges encode the call paths "
        "(static calls extracted from source, dynamic/curated calls manually annotated)."
    )
    lines.append("")

    # --- Boot sequence ---
    lines.append("## Boot Sequence")
    lines.append("")
    lines.append("Entrypoint chain from `main` through initial loader:")
    lines.append("")
    chain = index.get("entrypoint_chain") or []
    for i, step in enumerate(chain, 1):
        qname = step.get("qualified_name", "?")
        src = step.get("file", "?")
        lines.append(f"{i}. `{qname}` — `{src}`")
    lines.append("")
    lines.append(f"- **Root:** `{index.get('root', {}).get('qualified_name', '?')}`")
    lines.append(f"- **Orphan policy:** `{index.get('orphan_policy', 'strict')}`")
    lines.append("")

    # --- Per-shard sections ---
    lines.append("## Node Shards")
    lines.append("")

    # Group .yml and .h.yml separately; sort by stem
    shards = sorted(nodes_dir.glob("*.yml"), key=lambda p: p.name)

    # Aggregate stats
    total_nodes = 0
    total_edges = 0

    for shard_path in shards:
        nodes = _load_shard(shard_path)
        if not nodes:
            continue

        shard_edges = sum(len(n.get("edges") or []) for n in nodes)
        total_nodes += len(nodes)
        total_edges += shard_edges

        label = _shard_label(shard_path.name)
        lines.append(f"### {label}")
        lines.append("")
        lines.append(
            f"{len(nodes)} node(s), {shard_edges} edge(s) total"
        )
        lines.append("")

        # Table header
        lines.append("| Node | Edges | Edge detail |")
        lines.append("|---|---|---|")

        for node in nodes:
            qname = node.get("qualified_name", "?")
            edges = node.get("edges") or []
            edge_sum = _edge_summary(edges)

            # Build compact edge target list (up to 5)
            targets = []
            for e in edges[:5]:
                tgt = e.get("target", {}).get("qualified_name", "?")
                kind = e.get("kind", "?")
                conf = e.get("confidence", "")
                tag = f"{kind}" + (f"/{conf}" if conf and conf != "static" else "")
                targets.append(f"`{tgt}` ({tag})")
            if len(edges) > 5:
                targets.append(f"… +{len(edges)-5} more")

            target_str = "; ".join(targets) if targets else "—"
            # Escape pipes
            target_str = target_str.replace("|", "\\|")
            lines.append(f"| `{qname}` | {len(edges)} | {target_str} |")

        lines.append("")

    # --- Footer stats ---
    lines.append("## Statistics")
    lines.append("")
    lines.append(f"| Metric | Value |")
    lines.append(f"|---|---|")
    lines.append(f"| Shards | {len(shards)} |")
    lines.append(f"| Total nodes | {total_nodes} |")
    lines.append(f"| Total edges | {total_edges} |")
    lines.append("")

    out_path.write_text("\n".join(lines) + "\n")
    return out_path


def main() -> int:
    try:
        out = generate()
        print(f"FLOW_GRAPH.md written to {out}")
        return 0
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        import traceback; traceback.print_exc(file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
