"""Tests for scripts/flow/check_network_coverage.py."""
from __future__ import annotations

import sys
import textwrap
import unittest
from pathlib import Path
from unittest.mock import patch

# Insert ledger path first, then flow path so flow/ modules take priority.
_TESTS_DIR = Path(__file__).parent
_FLOW_DIR = _TESTS_DIR.parent
_LEDGER_DIR = _FLOW_DIR.parent / "ledger"
sys.path.insert(0, str(_LEDGER_DIR))
sys.path.insert(0, str(_FLOW_DIR))

import ledger_io  # noqa: E402
import check_network_coverage as cnc  # noqa: E402


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_cpp(cases: list[str]) -> str:
    """Build a minimal protocolgame.cpp fragment with a switch block."""
    body = "\n".join(f"\t\tcase {c}: break;" for c in cases)
    return f"switch (recvbyte) {{\n{body}\n}}"


def _make_cpp_with_disabled(active: list[str], disabled: list[str]) -> str:
    active_lines = "\n".join(f"\t\tcase {c}: break;" for c in active)
    disabled_lines = "\n".join(f"\t\t// case {c}: break; // disabled" for c in disabled)
    return f"switch (recvbyte) {{\n{active_lines}\n{disabled_lines}\n}}"


def _make_node(edges: list[dict], disabled_opcodes: list[dict] | None = None,
               noop_opcodes: list[dict] | None = None) -> dict:
    node: dict = {
        "file": "src/protocolgame.cpp",
        "qualified_name": "ProtocolGame::parsePacket",
        "edges": edges,
    }
    if disabled_opcodes is not None:
        node["disabled_opcodes"] = disabled_opcodes
    if noop_opcodes is not None:
        node["noop_opcodes"] = noop_opcodes
    return node


def _curated_edge(opcode: str, qname: str = "ProtocolGame::parseSomething") -> dict:
    return {
        "target": {"file": "src/protocolgame.cpp", "qualified_name": qname},
        "kind": "dynamic",
        "confidence": "curated",
        "condition": f"recvbyte == {opcode}",
        "order": 1,
    }


def _run_check(cpp: str, node: dict) -> list[str]:
    """Invoke check logic against synthetic cpp text and node, without touching filesystem."""
    active, disabled_from_cpp = cnc._load_switch_opcodes(cpp)
    curated = cnc._curated_opcodes(node)
    annotated_disabled = cnc._annotated_opcodes(node, "disabled_opcodes")
    annotated_noop = cnc._annotated_opcodes(node, "noop_opcodes")
    known_inactive = annotated_disabled | annotated_noop

    errors: list[str] = []

    for op in sorted(active):
        if op not in curated and op not in annotated_noop:
            errors.append(f"MISSING_EDGE: active case {op} in parsePacket has no curated edge")

    for op in sorted(curated):
        if op not in active and op not in known_inactive:
            errors.append(f"PHANTOM_EDGE: curated edge opcode {op} not found in switch arms")

    for op in sorted(disabled_from_cpp):
        if op not in known_inactive:
            errors.append(
                f"UNANNOTATED_DISABLED: commented-out case {op} not in "
                f"disabled_opcodes or noop_opcodes"
            )

    return errors


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

class TestMissingEdgeFails(unittest.TestCase):
    def test_active_case_without_curated_edge_fails(self):
        cpp = _make_cpp(["0x64"])
        node = _make_node(edges=[])  # no curated edge for 0x64
        errors = _run_check(cpp, node)
        self.assertTrue(any("MISSING_EDGE" in e and "0X64" in e for e in errors), errors)

    def test_active_case_with_curated_edge_passes(self):
        cpp = _make_cpp(["0x64"])
        node = _make_node(edges=[_curated_edge("0x64")])
        errors = _run_check(cpp, node)
        self.assertFalse(errors, errors)


class TestOpcodeMismatchFails(unittest.TestCase):
    def test_phantom_curated_opcode_fails(self):
        cpp = _make_cpp(["0x64"])
        # Edge claims 0x99 but no case 0x99 exists
        node = _make_node(edges=[_curated_edge("0x64"), _curated_edge("0x99")])
        errors = _run_check(cpp, node)
        self.assertTrue(any("PHANTOM_EDGE" in e and "0X99" in e for e in errors), errors)


class TestFullCoveragePasses(unittest.TestCase):
    def test_all_active_covered_passes(self):
        cpp = _make_cpp(["0x64", "0x65"])
        node = _make_node(edges=[_curated_edge("0x64"), _curated_edge("0x65")])
        errors = _run_check(cpp, node)
        self.assertFalse(errors, errors)

    def test_noop_opcode_requires_no_edge(self):
        cpp = _make_cpp(["0x64", "0x8E"])  # 0x8E is noop
        node = _make_node(
            edges=[_curated_edge("0x64")],
            noop_opcodes=[{"opcode": "0x8E", "comment": "join aggression"}],
        )
        errors = _run_check(cpp, node)
        self.assertFalse(errors, errors)

    def test_disabled_opcode_must_be_annotated(self):
        cpp = _make_cpp_with_disabled(active=["0x64"], disabled=["0x2A"])
        node = _make_node(
            edges=[_curated_edge("0x64")],
            disabled_opcodes=[{"opcode": "0x2A", "comment": "bestiary tracker"}],
        )
        errors = _run_check(cpp, node)
        self.assertFalse(errors, errors)

    def test_unannotated_disabled_fails(self):
        cpp = _make_cpp_with_disabled(active=["0x64"], disabled=["0x2A"])
        node = _make_node(
            edges=[_curated_edge("0x64")],
            # no disabled_opcodes annotation
        )
        errors = _run_check(cpp, node)
        self.assertTrue(any("UNANNOTATED_DISABLED" in e and "0X2A" in e for e in errors), errors)


class TestRealGraph(unittest.TestCase):
    """Integration test: the real protocolgame.cpp and real shard must pass."""

    def test_real_graph_passes(self):
        errors = cnc.check()
        self.assertFalse(
            errors,
            msg=f"Network coverage check failed on real graph:\n" + "\n".join(errors),
        )


if __name__ == "__main__":
    unittest.main()
