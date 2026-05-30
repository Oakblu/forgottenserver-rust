#!/usr/bin/env python3
"""Virtual-dispatch coverage check.

Asserts that for every in-scope Creature virtual method (all_virtuals minus
trivial accessors) that is overridden in Player, Monster, or Npc, there exists
a curated dynamic edge from the Creature::<method> node to the concrete
override node with condition "dyntype == <Subclass>".

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
    TRIVIAL_ACCESSOR_ALLOWLIST,
    _SUBCLASSES,
    parse_creature_virtuals,
    parse_overrides,
    in_scope_virtuals,
)

_ROOT = _HERE.parent.parent


def _load_nodes(shard_path: Path) -> dict[str, dict]:
    """Return {qualified_name: first node} from a shard (ignores overloads)."""
    doc = ledger_io.load(shard_path.read_text()) if shard_path.exists() else {}
    result: dict[str, dict] = {}
    for n in (doc or {}).get("nodes") or []:
        qn = n["qualified_name"]
        if qn not in result:
            result[qn] = n
    return result


def _has_curated_edge_to(node: dict, target_qname: str,
                          condition_prefix: str) -> bool:
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

    creature_nodes = _load_nodes(nodes_dir / "creature.yml")

    for class_prefix, _cpp, header_file in _SUBCLASSES:
        hdr = (root / "forgottenserver-upstream" / header_file).read_text()
        overrides = parse_overrides(hdr, class_prefix)
        required = scope & overrides

        for method in sorted(required):
            base_qname = f"Creature::{method}"
            override_qname = f"{class_prefix}::{method}"
            condition_prefix = f"dyntype == {class_prefix}"

            node = creature_nodes.get(base_qname)
            if node is None:
                errors.append(
                    f"MISSING_BASE_NODE: {base_qname} not found in creature.yml"
                )
                continue

            if not _has_curated_edge_to(node, override_qname, condition_prefix):
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
