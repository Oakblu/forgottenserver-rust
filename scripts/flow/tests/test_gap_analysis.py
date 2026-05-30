"""Tests for scripts/flow/gap_analysis.py."""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

_TESTS_DIR = Path(__file__).parent
_FLOW_DIR = _TESTS_DIR.parent
_LEDGER_DIR = _FLOW_DIR.parent / "ledger"
sys.path.insert(0, str(_LEDGER_DIR))
sys.path.insert(0, str(_FLOW_DIR))

from gap_analysis import (  # noqa: E402
    Finding,
    NodeKey,
    _is_stub,
    _priority,
    _rust_symbol_for,
    analyze,
    bfs,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _node(file: str, qname: str, edges: list[dict] | None = None) -> dict:
    return {"file": file, "qualified_name": qname, "edges": edges or []}


def _static_edge(tfile: str, tqname: str, condition: str = "") -> dict:
    e: dict = {
        "target": {"file": tfile, "qualified_name": tqname},
        "kind": "static",
        "confidence": "static",
    }
    if condition:
        e["condition"] = condition
    return e


def _dynamic_edge(tfile: str, tqname: str, condition: str = "") -> dict:
    e: dict = {
        "target": {"file": tfile, "qualified_name": tqname},
        "kind": "dynamic",
        "confidence": "curated",
    }
    if condition:
        e["condition"] = condition
    return e


def _ledger_entry(symbol: str, status: str, rust_qname: str = "") -> dict:
    entry: dict = {
        "cpp": {"file": "src/test.cpp", "symbol": symbol},
        "status": status,
        "rust": [],
    }
    if rust_qname:
        entry["rust"] = [{"qualified_name": rust_qname, "symbol": rust_qname}]
    return entry


def _manifest_entry(qname: str, body_hash: str = "abc123",
                    kind: str = "impl_method") -> dict:
    return {"qualified_name": qname, "body_hash": body_hash, "kind": kind}


# ---------------------------------------------------------------------------
# Unit tests for helpers
# ---------------------------------------------------------------------------

class TestIsStub(unittest.TestCase):
    def test_empty_qname_is_stub(self):
        self.assertTrue(_is_stub("", {}))

    def test_absent_from_manifest_is_stub(self):
        self.assertTrue(_is_stub("foo::bar", {}))

    def test_non_code_kind_not_stub(self):
        manifest = {"foo::Bar": _manifest_entry("foo::Bar", body_hash="", kind="struct")}
        self.assertFalse(_is_stub("foo::Bar", manifest))

    def test_empty_body_hash_is_stub(self):
        manifest = {"foo::bar": _manifest_entry("foo::bar", body_hash="")}
        self.assertTrue(_is_stub("foo::bar", manifest))

    def test_nonempty_body_hash_not_stub(self):
        manifest = {"foo::bar": _manifest_entry("foo::bar", body_hash="deadbeef")}
        self.assertFalse(_is_stub("foo::bar", manifest))


class TestRustSymbolFor(unittest.TestCase):
    def test_none_entry_returns_empty(self):
        self.assertEqual(_rust_symbol_for(None), "")

    def test_empty_rust_array_returns_empty(self):
        self.assertEqual(_rust_symbol_for({"rust": []}), "")

    def test_qualified_name_preferred(self):
        entry = {"rust": [{"qualified_name": "foo::Bar", "symbol": "foo::baz"}]}
        self.assertEqual(_rust_symbol_for(entry), "foo::Bar")

    def test_symbol_fallback(self):
        entry = {"rust": [{"qualified_name": "", "symbol": "foo::baz"}]}
        self.assertEqual(_rust_symbol_for(entry), "foo::baz")


# ---------------------------------------------------------------------------
# Core logic scenarios (using pure-Python synthetic data)
# ---------------------------------------------------------------------------

def _run_analysis(
    nodes: dict[NodeKey, dict],
    root_key: NodeKey,
    ledger: dict[str, dict],
    rust_manifest: dict[str, dict],
    intentional_syms: frozenset[str] = frozenset(),
) -> tuple[list[Finding], list[Finding]]:
    """Run the gap analysis logic against synthetic in-memory data."""
    from gap_analysis import (
        _is_stub as _is_stub_fn,
        _priority as _priority_fn,
        _rust_symbol_for as _rsf,
        _STATUS_WEIGHT,
        _criticality,
    )
    from collections import deque

    # BFS
    visited: dict[NodeKey, int] = {}
    queue: deque = deque([(root_key, 0)])
    while queue:
        key, depth = queue.popleft()
        if key in visited:
            continue
        visited[key] = depth
        node = nodes.get(key)
        if not node:
            continue
        for edge in (node.get("edges") or []):
            tgt = edge.get("target", {})
            tkey: NodeKey = (tgt.get("file", ""), tgt.get("qualified_name", ""))
            if tkey not in visited and tkey[1]:
                queue.append((tkey, depth + 1))

    findings: list[Finding] = []
    suppressed: list[Finding] = []
    seen: set = set()

    def _add(f: Finding) -> None:
        key = (f.category, f.cpp_file, f.qualified_name)
        if key in seen:
            return
        seen.add(key)
        sym = f.qualified_name
        if any(sym in idiff or idiff in sym for idiff in intentional_syms if idiff):
            suppressed.append(f)
        else:
            findings.append(f)

    for node_key, depth in visited.items():
        cpp_file, qname = node_key
        node = nodes.get(node_key)
        ledger_entry = ledger.get(qname)
        ledger_status = ledger_entry["status"] if ledger_entry else "missing"
        rust_qname = _rsf(ledger_entry)

        if ledger_status == "intentionally_removed":
            continue

        if ledger_status == "missing":
            _add(Finding(
                category="MISSING_FLOW",
                priority=_priority_fn(depth, _STATUS_WEIGHT["missing"], qname),
                cpp_file=cpp_file, qualified_name=qname,
                ledger_status=ledger_status, rust_symbol="",
                depth=depth, condition="", note="not in ledger",
            ))
        elif _is_stub_fn(rust_qname, rust_manifest):
            _add(Finding(
                category="MISSING_FLOW",
                priority=_priority_fn(depth, _STATUS_WEIGHT["stub"], qname),
                cpp_file=cpp_file, qualified_name=qname,
                ledger_status=ledger_status, rust_symbol=rust_qname,
                depth=depth, condition="", note="stub impl",
            ))

        if not node:
            continue

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
            tgt_status = tgt_ledger["status"] if tgt_ledger else "missing"
            if tgt_status == "intentionally_removed":
                continue
            tgt_rust = _rsf(tgt_ledger)
            tgt_gap = tgt_status == "missing" or _is_stub_fn(tgt_rust, rust_manifest)
            if not tgt_gap:
                continue
            if kind == "dynamic" and confidence == "curated":
                _add(Finding(
                    category="DYNAMIC_GAP",
                    priority=_priority_fn(depth, _STATUS_WEIGHT["dynamic_target"], qname),
                    cpp_file=cpp_file, qualified_name=qname,
                    ledger_status=ledger_status, rust_symbol=rust_qname,
                    depth=depth, condition=condition, note=f"-> {tqname}",
                ))
            elif condition and kind == "static":
                _add(Finding(
                    category="BRANCH_GAP",
                    priority=_priority_fn(depth, _STATUS_WEIGHT["branch_target"], qname),
                    cpp_file=cpp_file, qualified_name=qname,
                    ledger_status=ledger_status, rust_symbol=rust_qname,
                    depth=depth, condition=condition, note=f"-> {tqname}",
                ))

    findings.sort(key=lambda f: f.priority, reverse=True)
    suppressed.sort(key=lambda f: f.priority, reverse=True)
    return findings, suppressed


# ---------------------------------------------------------------------------
# Scenario tests
# ---------------------------------------------------------------------------

class TestReachablePendingIsMissingFlow(unittest.TestCase):
    """Reachable + not in ledger → MISSING_FLOW."""

    def test_missing_ledger_entry_is_missing_flow(self):
        root: NodeKey = ("src/main.cpp", "main")
        nodes = {root: _node("src/main.cpp", "main")}
        ledger: dict = {}  # main not in ledger
        manifest: dict = {}
        findings, _ = _run_analysis(nodes, root, ledger, manifest)
        self.assertTrue(
            any(f.category == "MISSING_FLOW" and f.qualified_name == "main"
                for f in findings),
            findings,
        )

    def test_stub_rust_is_missing_flow(self):
        root: NodeKey = ("src/main.cpp", "main")
        nodes = {root: _node("src/main.cpp", "main")}
        ledger = {"main": _ledger_entry("main", "migrated", "server::main_fn")}
        manifest = {"server::main_fn": _manifest_entry("server::main_fn", body_hash="")}
        findings, _ = _run_analysis(nodes, root, ledger, manifest)
        self.assertTrue(
            any(f.category == "MISSING_FLOW" and f.qualified_name == "main"
                for f in findings),
            findings,
        )


class TestReachableDoneNoFinding(unittest.TestCase):
    """Reachable + DONE (migrated + non-stub Rust) → no finding."""

    def test_migrated_with_body_hash_no_finding(self):
        root: NodeKey = ("src/main.cpp", "main")
        nodes = {root: _node("src/main.cpp", "main")}
        ledger = {"main": _ledger_entry("main", "migrated", "server::main_fn")}
        manifest = {"server::main_fn": _manifest_entry("server::main_fn", body_hash="deadbeef")}
        findings, _ = _run_analysis(nodes, root, ledger, manifest)
        self.assertFalse(
            any(f.qualified_name == "main" for f in findings),
            findings,
        )


class TestIntentionalDifferenceSuppressed(unittest.TestCase):
    """Symbol in intentional_differences.yml → suppressed, not actionable."""

    def test_intentional_symbol_goes_to_suppressed(self):
        root: NodeKey = ("src/main.cpp", "main")
        nodes = {root: _node("src/main.cpp", "main")}
        ledger: dict = {}  # main not in ledger → would be MISSING_FLOW
        manifest: dict = {}
        intentional = frozenset({"main"})
        findings, suppressed = _run_analysis(nodes, root, ledger, manifest, intentional)
        # Should not appear in actionable
        self.assertFalse(
            any(f.qualified_name == "main" for f in findings),
            findings,
        )
        # Should appear in suppressed
        self.assertTrue(
            any(f.qualified_name == "main" for f in suppressed),
            suppressed,
        )


class TestDynamicGap(unittest.TestCase):
    """Dynamic edge without Rust handler → DYNAMIC_GAP."""

    def test_dynamic_edge_to_missing_handler_is_dynamic_gap(self):
        root: NodeKey = ("src/protocolgame.cpp", "ProtocolGame::parsePacket")
        handler: NodeKey = ("src/protocolgame.cpp", "ProtocolGame::parseSomeOpcode")
        nodes = {
            root: _node(
                "src/protocolgame.cpp", "ProtocolGame::parsePacket",
                edges=[_dynamic_edge("src/protocolgame.cpp",
                                     "ProtocolGame::parseSomeOpcode",
                                     "recvbyte == 0x64")],
            ),
            handler: _node("src/protocolgame.cpp", "ProtocolGame::parseSomeOpcode"),
        }
        ledger = {
            "ProtocolGame::parsePacket": _ledger_entry(
                "ProtocolGame::parsePacket", "migrated", "network::ProtocolGame::parse_packet"),
            # handler NOT in ledger → missing
        }
        manifest = {
            "network::ProtocolGame::parse_packet": _manifest_entry(
                "network::ProtocolGame::parse_packet", body_hash="aaa"),
        }
        findings, _ = _run_analysis(nodes, root, ledger, manifest)
        self.assertTrue(
            any(f.category == "DYNAMIC_GAP" and "parsePacket" in f.qualified_name
                for f in findings),
            findings,
        )


class TestAllDoneEmptyReport(unittest.TestCase):
    """All reachable nodes DONE → empty actionable report."""

    def test_all_done_no_findings(self):
        root: NodeKey = ("src/main.cpp", "main")
        child: NodeKey = ("src/game.cpp", "Game::start")
        nodes = {
            root: _node("src/main.cpp", "main",
                        edges=[_static_edge("src/game.cpp", "Game::start")]),
            child: _node("src/game.cpp", "Game::start"),
        }
        ledger = {
            "main": _ledger_entry("main", "migrated", "server::main_fn"),
            "Game::start": _ledger_entry("Game::start", "migrated", "game::Game::start"),
        }
        manifest = {
            "server::main_fn": _manifest_entry("server::main_fn", body_hash="aaa"),
            "game::Game::start": _manifest_entry("game::Game::start", body_hash="bbb"),
        }
        findings, suppressed = _run_analysis(nodes, root, ledger, manifest)
        self.assertFalse(findings, findings)
        self.assertFalse(suppressed, suppressed)


# ---------------------------------------------------------------------------
# Priority ordering test (task 3.2)
# ---------------------------------------------------------------------------

class TestBootSpineOutranksDeepBranch(unittest.TestCase):
    """Boot-spine finding outranks a deep-branch finding."""

    def test_shallow_outranks_deep(self):
        # Boot-spine at depth 0; deep branch at depth 5
        boot_finding = Finding(
            category="MISSING_FLOW",
            priority=_priority(0, 0.9, "main"),
            cpp_file="src/main.cpp", qualified_name="main",
            ledger_status="missing", rust_symbol="",
            depth=0, condition="", note="",
        )
        deep_finding = Finding(
            category="MISSING_FLOW",
            priority=_priority(5, 0.9, "SomeDeep::obscureMethod"),
            cpp_file="src/some.cpp", qualified_name="SomeDeep::obscureMethod",
            ledger_status="missing", rust_symbol="",
            depth=5, condition="", note="",
        )
        self.assertGreater(boot_finding.priority, deep_finding.priority)

    def test_priority_decreases_with_depth(self):
        p0 = _priority(0, 0.9, "X")
        p3 = _priority(3, 0.9, "X")
        p10 = _priority(10, 0.9, "X")
        self.assertGreater(p0, p3)
        self.assertGreater(p3, p10)


# ---------------------------------------------------------------------------
# Real graph integration test
# ---------------------------------------------------------------------------

class TestRealGraphAnalysis(unittest.TestCase):
    """Integration test: real graph + real ledger must run without error."""

    def test_real_graph_runs(self):
        findings, suppressed = analyze()
        # No assertion on count — just verify it runs and returns lists
        self.assertIsInstance(findings, list)
        self.assertIsInstance(suppressed, list)
        # All findings must have the expected fields
        for f in findings:
            self.assertIn(f.category,
                          ("MISSING_FLOW", "DYNAMIC_GAP", "BRANCH_GAP", "ORDER_MISMATCH"))
            self.assertGreaterEqual(f.priority, 0.0)
            self.assertLessEqual(f.priority, 1.0)
            self.assertGreaterEqual(f.depth, 0)


if __name__ == "__main__":
    unittest.main()
