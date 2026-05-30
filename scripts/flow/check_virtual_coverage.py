#!/usr/bin/env python3
"""Virtual-dispatch coverage check.

Asserts that for every in-scope Creature virtual method (all_virtuals minus
trivial accessors) that is overridden in Player, Monster, or Npc, the
corresponding Creature::<method> node has a curated dynamic edge to the
concrete override with condition "dyntype == <Subclass>".

Looks up nodes in creature.yml (src/creature.cpp implementations) first,
then creature.h.yml (pure-virtual / inline declarations).

Exits 0 on success, 1 on any violation.

Usage (from repo root):
    python3 scripts/flow/check_virtual_coverage.py
"""
from __future__ import annotations

import sys
from pathlib import Path

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

from curate_virtual import (  # noqa: E402
    _SUBCLASSES,
    _load_manifest,
    parse_creature_virtuals,
    parse_overrides,
    in_scope_virtuals,
)

_ROOT = _HERE.parent.parent


def _load_nodes(shard_path: Path) -> dict[str, list[dict]]:
    """Return {qualified_name: [nodes]} from a shard (keeps all overloads)."""
    doc = ledger_io.load(shard_path.read_text()) if shard_path.exists() else {}
    result: dict[str, list[dict]] = {}
    for n in (doc or {}).get("nodes") or []:
        result.setdefault(n["qualified_name"], []).append(n)
    return result


def _any_has_curated_edge(nodes: list[dict], target_qname: str,
                           condition_prefix: str) -> bool:
    """True if ANY of the nodes (overloads) has the required curated edge."""
    for node in nodes:
        for e in node.get("edges") or []:
            if (e.get("confidence") == "curated"
                    and e.get("kind") == "dynamic"
                    and e.get("target", {}).get("qualified_name") == target_qname
                    and str(e.get("condition", "")).startswith(condition_prefix)):
                return True
    return False


def check(root: Path = _ROOT) -> list[str]:
    errors: list[str] = []
    nodes_dir = root / "flow_graph" / "nodes"

    creature_h = (root / "forgottenserver-upstream" / "src" / "creature.h").read_text()
    all_virtuals, _pure = parse_creature_virtuals(creature_h)
    scope = in_scope_virtuals(all_virtuals)

    manifest = _load_manifest(root)
    cpp_nodes = _load_nodes(nodes_dir / "creature.yml")
    hdr_nodes = _load_nodes(nodes_dir / "creature.h.yml")

    for class_prefix, _cpp, header_file in _SUBCLASSES:
        hdr = (root / "forgottenserver-upstream" / header_file).read_text()
        overrides = parse_overrides(hdr, class_prefix)
        # Only require edges for overrides that have a .cpp definition in the manifest.
        # Inline-only header overrides are excluded (no flow-graph node to point to).
        required = {
            m for m in scope & overrides
            if f"{class_prefix}::{m}" in manifest
        }

        for method in sorted(required):
            base_qname = f"Creature::{method}"
            override_qname = f"{class_prefix}::{method}"
            condition_prefix = f"dyntype == {class_prefix}"

            nodes = cpp_nodes.get(base_qname) or hdr_nodes.get(base_qname)
            if not nodes:
                errors.append(
                    f"MISSING_BASE_NODE: {base_qname} not found in "
                    f"creature.yml or creature.h.yml"
                )
                continue

            if not _any_has_curated_edge(nodes, override_qname, condition_prefix):
                errors.append(
                    f"MISSING_VIRTUAL_EDGE: {base_qname} has no curated edge "
                    f"to {override_qname} (condition: {condition_prefix!r})"
                )

    return errors


def main() -> int:
    try:
        errors = check()
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1

    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        print(f"\n{len(errors)} virtual coverage error(s).", file=sys.stderr)
        return 1

    print("Virtual coverage: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
