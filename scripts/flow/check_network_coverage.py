#!/usr/bin/env python3
"""Opcode coverage check for ProtocolGame::parsePacket.

Parses the switch (recvbyte) arms in protocolgame.cpp and verifies:
  1. Every active case has at least one curated edge in the graph.
  2. Every curated edge condition's opcode matches an active case (no phantoms).
  3. Every commented-out case is listed in disabled_opcodes or noop_opcodes.

Exits 0 on success, 1 on any violation.

Usage (from repo root):
    python3 scripts/flow/check_network_coverage.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

_ROOT = _HERE.parent.parent
_CPP = _ROOT / "forgottenserver-upstream" / "src" / "protocolgame.cpp"
_SHARD = _ROOT / "flow_graph" / "nodes" / "protocolgame.yml"

# Matches active cases: "case 0x64:"
_ACTIVE_CASE = re.compile(r"^\s*case\s+(0x[0-9A-Fa-f]+)\s*:", re.MULTILINE)

# Matches disabled cases in comments: "// case 0x2A: ..."
_DISABLED_CASE = re.compile(r"//\s*case\s+(0x[0-9A-Fa-f]+)\s*:", re.MULTILINE)

# Extract opcode from curated edge condition: "recvbyte == 0x64 ..."
_CONDITION_OPCODE = re.compile(r"recvbyte\s*==\s*(0x[0-9A-Fa-f]+)")


def _load_switch_opcodes(cpp_text: str) -> tuple[set[str], set[str]]:
    """Return (active_opcodes, disabled_opcodes) extracted from protocolgame.cpp.

    Only considers the region from 'switch (recvbyte)' to the end of the function.
    """
    switch_start = cpp_text.find("switch (recvbyte)")
    if switch_start == -1:
        raise ValueError("Could not locate 'switch (recvbyte)' in protocolgame.cpp")
    region = cpp_text[switch_start:]

    active = {m.group(1).upper() for m in _ACTIVE_CASE.finditer(region)}
    disabled = {m.group(1).upper() for m in _DISABLED_CASE.finditer(region)}
    # Disabled cases that are also 'active' means a real case exists alongside a comment;
    # remove from disabled in that case.
    disabled -= active
    return active, disabled


def _load_graph_node() -> dict:
    doc = ledger_io.load(_SHARD.read_text()) or {}
    nodes = doc.get("nodes") or []
    for n in nodes:
        if n.get("qualified_name") == "ProtocolGame::parsePacket":
            return n
    raise ValueError("ProtocolGame::parsePacket node not found in protocolgame.yml")


def _curated_opcodes(node: dict) -> dict[str, list[dict]]:
    """Map OPCODE (upper) → list of curated edges claiming that opcode."""
    result: dict[str, list[dict]] = {}
    for edge in node.get("edges") or []:
        if edge.get("confidence") != "curated":
            continue
        cond = edge.get("condition", "")
        m = _CONDITION_OPCODE.search(cond)
        if m:
            op = m.group(1).upper()
            result.setdefault(op, []).append(edge)
    return result


def _annotated_opcodes(node: dict, key: str) -> set[str]:
    return {
        entry["opcode"].upper()
        for entry in (node.get(key) or [])
        if isinstance(entry, dict) and "opcode" in entry
    }


def check(root: Path = _ROOT) -> list[str]:
    errors: list[str] = []

    cpp_text = (root / "forgottenserver-upstream" / "src" / "protocolgame.cpp").read_text()
    shard_path = root / "flow_graph" / "nodes" / "protocolgame.yml"

    active_cases, disabled_cases = _load_switch_opcodes(cpp_text)

    doc = ledger_io.load(shard_path.read_text()) or {}
    node: dict | None = None
    for n in (doc.get("nodes") or []):
        if n.get("qualified_name") == "ProtocolGame::parsePacket":
            node = n
            break
    if node is None:
        return ["MISSING_NODE: ProtocolGame::parsePacket not in protocolgame.yml"]

    curated = _curated_opcodes(node)
    annotated_disabled = _annotated_opcodes(node, "disabled_opcodes")
    annotated_noop = _annotated_opcodes(node, "noop_opcodes")
    known_inactive = annotated_disabled | annotated_noop

    # 1. Every active, non-noop case must have at least one curated edge.
    for op in sorted(active_cases):
        if op not in curated and op not in annotated_noop:
            errors.append(
                f"MISSING_EDGE: active case {op} in parsePacket has no curated edge"
            )

    # 2. Every curated edge opcode must match an active, noop, or known-inactive case.
    for op, edges in sorted(curated.items()):
        if op not in active_cases and op not in known_inactive:
            errors.append(
                f"PHANTOM_EDGE: curated edge opcode {op} not found in switch arms"
            )

    # 3. Every C++ disabled case should be annotated in the graph.
    for op in sorted(disabled_cases):
        if op not in known_inactive:
            errors.append(
                f"UNANNOTATED_DISABLED: commented-out case {op} not in "
                f"disabled_opcodes or noop_opcodes"
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
        print(f"\n{len(errors)} opcode coverage error(s).", file=sys.stderr)
        return 1

    print("Network coverage: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
