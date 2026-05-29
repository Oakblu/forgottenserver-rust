"""Unit tests for scripts/flow/validate.py.

Tests use a temporary directory fixture with a minimal manifest and graph to
exercise the three validation checks independently:
  - UNKNOWN_NODE_KEY  (node key absent from manifest)
  - DANGLING_EDGE     (edge target absent from manifest)
  - ORPHAN_NODE       (node unreachable from root and not in unreached)
"""
from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).parent
# Ledger path inserted first so flow/validate.py takes priority at [0].
sys.path.insert(0, str(HERE.parent.parent / "ledger"))
sys.path.insert(0, str(HERE.parent))

import validate as validator  # noqa: E402
import ledger_io  # noqa: E402


# ---------------------------------------------------------------------------
# Manifest fixture — three known symbols
# ---------------------------------------------------------------------------

MANIFEST = [
    {"file": "src/foo.cpp", "qualified_name": "foo::entry"},
    {"file": "src/foo.cpp", "qualified_name": "foo::helper"},
    {"file": "src/bar.cpp", "qualified_name": "bar::run"},
]


# ---------------------------------------------------------------------------
# Fixture writer
# ---------------------------------------------------------------------------

def _write_fixture(
    tmp: Path,
    nodes: list[dict],
    root: dict | None = None,
    unreached: list[dict] | None = None,
    extra_manifest: list[dict] | None = None,
) -> None:
    """Write a minimal flow-graph fixture under *tmp*."""
    manifest = list(MANIFEST)
    if extra_manifest:
        manifest.extend(extra_manifest)
    (tmp / "cpp_symbol_manifest.json").write_text(json.dumps(manifest))

    flow = tmp / "flow_graph"
    flow.mkdir()
    (flow / "nodes").mkdir()

    root_node = root or {"file": "src/foo.cpp", "qualified_name": "foo::entry"}
    index = ledger_io.dump({
        "root": root_node,
        "entrypoint_chain": [root_node],
        "unreached": unreached or [],
    })
    (flow / "index.yml").write_text(index)

    shard = ledger_io.dump({"nodes": nodes})
    (flow / "nodes" / "foo.yml").write_text(shard)


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

class TestDanglingEdge(unittest.TestCase):
    """An edge whose target is absent from the manifest must be flagged."""

    def test_dangling_edge_fails(self):
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            nodes = [
                {
                    "file": "src/foo.cpp",
                    "qualified_name": "foo::entry",
                    "edges": [
                        {
                            "target": {
                                "file": "src/nonexistent.cpp",
                                "qualified_name": "missing::sym",
                            },
                            "kind": "static",
                            "confidence": "curated",
                        }
                    ],
                }
            ]
            _write_fixture(tmp, nodes=nodes)
            errors = validator.validate(tmp)
            self.assertTrue(
                any("DANGLING_EDGE" in e for e in errors),
                f"Expected DANGLING_EDGE in errors, got: {errors}",
            )

    def test_edge_to_manifest_symbol_passes(self):
        """An edge whose target IS in the manifest is not a dangling edge."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            nodes = [
                {
                    "file": "src/foo.cpp",
                    "qualified_name": "foo::entry",
                    "edges": [
                        {
                            "target": {
                                "file": "src/bar.cpp",
                                "qualified_name": "bar::run",
                            },
                            "kind": "static",
                            "confidence": "curated",
                            "order": 1,
                        }
                    ],
                }
            ]
            _write_fixture(tmp, nodes=nodes)
            errors = validator.validate(tmp)
            dangling = [e for e in errors if "DANGLING_EDGE" in e]
            self.assertEqual(dangling, [], f"Unexpected DANGLING_EDGE: {dangling}")


class TestOrphanNode(unittest.TestCase):
    """A node not reachable from root and not in unreached must be flagged."""

    def test_orphan_fails(self):
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            # foo::entry (root) has no edge to foo::helper → helper is orphaned.
            nodes = [
                {"file": "src/foo.cpp", "qualified_name": "foo::entry", "edges": []},
                {"file": "src/foo.cpp", "qualified_name": "foo::helper", "edges": []},
            ]
            _write_fixture(tmp, nodes=nodes)
            errors = validator.validate(tmp)
            self.assertTrue(
                any("ORPHAN_NODE" in e for e in errors),
                f"Expected ORPHAN_NODE in errors, got: {errors}",
            )

    def test_unreached_node_passes(self):
        """A node listed in unreached is not flagged as an orphan."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            nodes = [
                {"file": "src/foo.cpp", "qualified_name": "foo::entry", "edges": []},
                {"file": "src/foo.cpp", "qualified_name": "foo::helper", "edges": []},
            ]
            unreached = [
                {
                    "file": "src/foo.cpp",
                    "qualified_name": "foo::helper",
                    "reason": "legacy dead code",
                }
            ]
            _write_fixture(tmp, nodes=nodes, unreached=unreached)
            errors = validator.validate(tmp)
            orphans = [e for e in errors if "ORPHAN_NODE" in e]
            self.assertEqual(orphans, [], f"Unexpected ORPHAN_NODE: {orphans}")


class TestCleanGraph(unittest.TestCase):
    """A well-formed, fully-reachable graph with valid edges must pass."""

    def test_boot_spine_passes(self):
        """Two-node chain from root → helper: no errors."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            nodes = [
                {
                    "file": "src/foo.cpp",
                    "qualified_name": "foo::entry",
                    "edges": [
                        {
                            "target": {
                                "file": "src/foo.cpp",
                                "qualified_name": "foo::helper",
                            },
                            "kind": "static",
                            "confidence": "curated",
                            "order": 1,
                        }
                    ],
                },
                {"file": "src/foo.cpp", "qualified_name": "foo::helper", "edges": []},
            ]
            _write_fixture(tmp, nodes=nodes)
            errors = validator.validate(tmp)
            self.assertEqual(errors, [], f"Expected no errors, got: {errors}")

    def test_edge_to_non_node_manifest_symbol_passes(self):
        """An edge to a manifest symbol that has no node entry is not an orphan."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            # bar::run is in manifest but has no node; edge from entry to it is OK.
            nodes = [
                {
                    "file": "src/foo.cpp",
                    "qualified_name": "foo::entry",
                    "edges": [
                        {
                            "target": {
                                "file": "src/bar.cpp",
                                "qualified_name": "bar::run",
                            },
                            "kind": "static",
                            "confidence": "curated",
                            "order": 1,
                        }
                    ],
                }
            ]
            _write_fixture(tmp, nodes=nodes)
            errors = validator.validate(tmp)
            self.assertEqual(errors, [], f"Expected no errors, got: {errors}")


class TestUnknownNodeKey(unittest.TestCase):
    """A node whose key is absent from the manifest must be flagged."""

    def test_unknown_node_key_fails(self):
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            nodes = [
                # This key is NOT in the MANIFEST fixture.
                {
                    "file": "src/ghost.cpp",
                    "qualified_name": "ghost::phantom",
                    "edges": [],
                }
            ]
            # Make this node reachable by making it the root too.
            root = {"file": "src/ghost.cpp", "qualified_name": "ghost::phantom"}
            _write_fixture(tmp, nodes=nodes, root=root)
            errors = validator.validate(tmp)
            self.assertTrue(
                any("UNKNOWN_NODE_KEY" in e for e in errors),
                f"Expected UNKNOWN_NODE_KEY in errors, got: {errors}",
            )


if __name__ == "__main__":
    unittest.main()
