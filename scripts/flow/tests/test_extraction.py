"""Tests for bootstrap_nodes.py and build_edges.py."""
from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).parent
sys.path.insert(0, str(HERE.parent.parent / "ledger"))
sys.path.insert(0, str(HERE.parent))

import ledger_io  # noqa: E402
import bootstrap_nodes as bootstrap  # noqa: E402
import build_edges as extractor  # noqa: E402


# ---------------------------------------------------------------------------
# Shared fixture helpers
# ---------------------------------------------------------------------------

_MANIFEST_BASE = [
    {"file": "src/otserv.cpp", "qualified_name": "mainLoader",
     "kind": "static_function", "line_start": 1, "line_end": 5},
    {"file": "src/game.cpp",   "qualified_name": "Game::loadMainMap",
     "kind": "method",         "line_start": 1, "line_end": 3},
    {"file": "src/game.cpp",   "qualified_name": "Game::setGameState",
     "kind": "method",         "line_start": 5, "line_end": 7},
    {"file": "src/game.cpp",   "qualified_name": "Game::start",
     "kind": "method",         "line_start": 9, "line_end": 11},
]


def _write_manifest(tmp: Path, extra: list[dict] | None = None) -> None:
    data = list(_MANIFEST_BASE) + (extra or [])
    (tmp / "cpp_symbol_manifest.json").write_text(json.dumps(data))


def _write_source(tmp: Path, rel_path: str, content: str) -> None:
    p = tmp / "forgottenserver-upstream" / "src" / rel_path
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(content)


def _write_game_source(tmp: Path) -> None:
    # Minimal game.cpp bodies matching line ranges in _MANIFEST_BASE.
    _write_source(tmp, "game.cpp",
        "void Game::loadMainMap(string name) {}\n"
        "// line 2\n"
        "// line 3\n"
        "// line 4\n"
        "void Game::setGameState(GameState s) {}\n"
        "// line 6\n"
        "// line 7\n"
        "// line 8\n"
        "void Game::start(ServiceManager* s) {}\n"
    )


def _flow_dir(tmp: Path) -> Path:
    d = tmp / "flow_graph" / "nodes"
    d.mkdir(parents=True, exist_ok=True)
    return d


def _read_nodes(tmp: Path, shard: str) -> list[dict]:
    p = tmp / "flow_graph" / "nodes" / shard
    if not p.exists():
        return []
    doc = ledger_io.load(p.read_text())
    return (doc or {}).get("nodes") or []


def _node_by_qn(nodes: list[dict], qn: str) -> dict | None:
    return next((n for n in nodes if n["qualified_name"] == qn), None)


def _edge_targets(node: dict) -> set[tuple[str, str]]:
    edges = node.get("edges") or []
    return {
        (e["target"]["file"], e["target"]["qualified_name"])
        for e in edges
        if isinstance(e.get("target"), dict)
    }


# ---------------------------------------------------------------------------
# Bootstrap tests
# ---------------------------------------------------------------------------

class TestBootstrapNodes(unittest.TestCase):

    def test_every_manifest_symbol_gets_a_node(self):
        """Every manifest symbol must have a node after bootstrap."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            _write_manifest(tmp)
            _flow_dir(tmp)
            bootstrap.bootstrap(root=tmp)

            # Check shard for otserv.cpp
            nodes = _read_nodes(tmp, "otserv.yml")
            keys = {(n["file"], n["qualified_name"]) for n in nodes}
            self.assertIn(("src/otserv.cpp", "mainLoader"), keys)

            # Check shard for game.cpp
            nodes = _read_nodes(tmp, "game.yml")
            keys = {(n["file"], n["qualified_name"]) for n in nodes}
            self.assertIn(("src/game.cpp", "Game::loadMainMap"), keys)
            self.assertIn(("src/game.cpp", "Game::setGameState"), keys)
            self.assertIn(("src/game.cpp", "Game::start"), keys)

    def test_no_node_key_absent_from_manifest(self):
        """No node should exist whose key is not in the manifest."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            _write_manifest(tmp)
            _flow_dir(tmp)
            bootstrap.bootstrap(root=tmp)

            manifest_keys = {
                (s["file"], s["qualified_name"])
                for s in _MANIFEST_BASE
            }
            for shard in (tmp / "flow_graph" / "nodes").glob("*.yml"):
                nodes = _read_nodes(tmp, shard.name)
                for n in nodes:
                    self.assertIn(
                        (n["file"], n["qualified_name"]),
                        manifest_keys,
                        f"Node key not in manifest: {n['file']}::{n['qualified_name']}",
                    )

    def test_existing_boot_spine_nodes_preserved(self):
        """Bootstrap must not overwrite nodes that already have curated edges."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            _write_manifest(tmp)
            nodes_dir = _flow_dir(tmp)

            # Pre-write a shard with a curated edge on mainLoader.
            curated_edge = {
                "target": {"file": "src/game.cpp", "qualified_name": "Game::start"},
                "kind": "dynamic",
                "confidence": "curated",
                "order": 1,
            }
            shard_content = ledger_io.dump({
                "nodes": [
                    {
                        "file": "src/otserv.cpp",
                        "qualified_name": "mainLoader",
                        "edges": [curated_edge],
                    }
                ]
            })
            (nodes_dir / "otserv.yml").write_text(shard_content)

            bootstrap.bootstrap(root=tmp)

            nodes = _read_nodes(tmp, "otserv.yml")
            ml = _node_by_qn(nodes, "mainLoader")
            self.assertIsNotNone(ml)
            self.assertEqual(ml["edges"], [curated_edge])

    def test_bootstrap_is_idempotent(self):
        """Running bootstrap twice must not change any file."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            _write_manifest(tmp)
            _flow_dir(tmp)

            bootstrap.bootstrap(root=tmp)
            snapshots_1 = {
                p.name: p.read_text()
                for p in (tmp / "flow_graph" / "nodes").glob("*.yml")
            }
            bootstrap.bootstrap(root=tmp)
            snapshots_2 = {
                p.name: p.read_text()
                for p in (tmp / "flow_graph" / "nodes").glob("*.yml")
            }
            self.assertEqual(snapshots_1, snapshots_2)


# ---------------------------------------------------------------------------
# Extraction tests
# ---------------------------------------------------------------------------

class TestBuildEdges(unittest.TestCase):

    def _setup(self, tmp: Path, source_body: str, extra_manifest: list[dict] | None = None) -> None:
        _write_manifest(tmp, extra=extra_manifest)
        _write_game_source(tmp)
        # Write mainLoader source with the given body.
        _write_source(tmp, "otserv.cpp",
            "Game g_game;\n"
            + source_body + "\n"
        )
        _flow_dir(tmp)
        bootstrap.bootstrap(root=tmp)

    def test_qualified_call_yields_static_edge(self):
        """A direct qualified call must produce a kind:static edge."""
        body = (
            "void mainLoader(ServiceManager* s) {\n"
            "    if (!ConfigManager::load()) { return; }\n"
            "}\n"
        )
        # Add ConfigManager::load to manifest.
        extra = [{"file": "src/configmanager.cpp", "qualified_name": "ConfigManager::load",
                  "kind": "method", "line_start": 1, "line_end": 3}]
        # Source body starts at line 2 (after 'Game g_game;').
        # Adjust manifest line range.
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            _write_manifest(tmp, extra=extra)
            _write_game_source(tmp)
            # Write mainLoader at line 2 (line_start=2, line_end=4)
            src = "Game g_game;\nvoid mainLoader(ServiceManager* s) {\n    if (!ConfigManager::load()) { return; }\n}\n"
            _write_source(tmp, "otserv.cpp", src)
            _flow_dir(tmp)
            # Update manifest line range for mainLoader
            manifest = json.loads((tmp / "cpp_symbol_manifest.json").read_text())
            for s in manifest:
                if s["qualified_name"] == "mainLoader":
                    s["line_start"], s["line_end"] = 2, 4
            (tmp / "cpp_symbol_manifest.json").write_text(json.dumps(manifest))
            bootstrap.bootstrap(root=tmp)

            extractor.build(root=tmp)
            nodes = _read_nodes(tmp, "otserv.yml")
            ml = _node_by_qn(nodes, "mainLoader")
            self.assertIsNotNone(ml)
            targets = _edge_targets(ml)
            self.assertIn(
                ("src/configmanager.cpp", "ConfigManager::load"),
                targets,
                f"Expected ConfigManager::load in edges, got: {targets}",
            )

    def test_member_call_via_global_yields_static_edge(self):
        """A g_game.loadMainMap() call must resolve to Game::loadMainMap."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            # mainLoader body calls g_game.loadMainMap(...) at lines 2-4.
            src = (
                "Game g_game;\n"
                "void mainLoader(ServiceManager* s) {\n"
                "    g_game.loadMainMap(\"test\");\n"
                "    g_game.setGameState(GAME_STATE_NORMAL);\n"
                "}\n"
            )
            _write_manifest(tmp)
            _write_game_source(tmp)
            _write_source(tmp, "otserv.cpp", src)
            _flow_dir(tmp)
            manifest = json.loads((tmp / "cpp_symbol_manifest.json").read_text())
            for s in manifest:
                if s["qualified_name"] == "mainLoader":
                    s["line_start"], s["line_end"] = 2, 5
            (tmp / "cpp_symbol_manifest.json").write_text(json.dumps(manifest))
            bootstrap.bootstrap(root=tmp)

            extractor.build(root=tmp)
            nodes = _read_nodes(tmp, "otserv.yml")
            ml = _node_by_qn(nodes, "mainLoader")
            self.assertIsNotNone(ml)
            targets = _edge_targets(ml)
            self.assertIn(
                ("src/game.cpp", "Game::loadMainMap"),
                targets,
                f"Expected Game::loadMainMap in targets, got: {targets}",
            )

    def test_curated_dynamic_edge_survives_rebuild(self):
        """A kind:dynamic curated edge must survive a static rebuild."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            src = (
                "Game g_game;\n"
                "void mainLoader(ServiceManager* s) {\n"
                "    g_game.start(s);\n"
                "}\n"
            )
            _write_manifest(tmp)
            _write_game_source(tmp)
            _write_source(tmp, "otserv.cpp", src)
            _flow_dir(tmp)

            manifest = json.loads((tmp / "cpp_symbol_manifest.json").read_text())
            for s in manifest:
                if s["qualified_name"] == "mainLoader":
                    s["line_start"], s["line_end"] = 2, 4
            (tmp / "cpp_symbol_manifest.json").write_text(json.dumps(manifest))
            bootstrap.bootstrap(root=tmp)

            # Manually insert a curated dynamic edge.
            nodes_dir = tmp / "flow_graph" / "nodes"
            nodes = _read_nodes(tmp, "otserv.yml")
            ml = _node_by_qn(nodes, "mainLoader")
            curated_edge = {
                "target": {"file": "src/game.cpp", "qualified_name": "Game::start"},
                "kind": "dynamic",
                "confidence": "curated",
                "order": 99,
            }
            ml["edges"] = [curated_edge]
            (nodes_dir / "otserv.yml").write_text(ledger_io.dump({"nodes": nodes}))

            extractor.build(root=tmp)

            nodes_after = _read_nodes(tmp, "otserv.yml")
            ml_after = _node_by_qn(nodes_after, "mainLoader")
            dynamic_edges = [
                e for e in (ml_after.get("edges") or [])
                if e.get("kind") == "dynamic" and e.get("confidence") == "curated"
            ]
            self.assertTrue(
                len(dynamic_edges) >= 1,
                f"Curated dynamic edge was lost after rebuild. Edges: {ml_after.get('edges')}",
            )
            self.assertEqual(dynamic_edges[0]["target"]["qualified_name"], "Game::start")

    def test_rebuild_is_idempotent(self):
        """Running make flow-build twice must produce identical shard files."""
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            src = (
                "Game g_game;\n"
                "void mainLoader(ServiceManager* s) {\n"
                "    g_game.loadMainMap(\"map\");\n"
                "    g_game.setGameState(NORMAL);\n"
                "}\n"
            )
            _write_manifest(tmp)
            _write_game_source(tmp)
            _write_source(tmp, "otserv.cpp", src)
            _flow_dir(tmp)
            manifest = json.loads((tmp / "cpp_symbol_manifest.json").read_text())
            for s in manifest:
                if s["qualified_name"] == "mainLoader":
                    s["line_start"], s["line_end"] = 2, 5
            (tmp / "cpp_symbol_manifest.json").write_text(json.dumps(manifest))

            # First build.
            bootstrap.bootstrap(root=tmp)
            extractor.build(root=tmp)
            snap1 = {
                p.name: p.read_text()
                for p in (tmp / "flow_graph" / "nodes").glob("*.yml")
            }

            # Second build.
            bootstrap.bootstrap(root=tmp)
            extractor.build(root=tmp)
            snap2 = {
                p.name: p.read_text()
                for p in (tmp / "flow_graph" / "nodes").glob("*.yml")
            }

            self.assertEqual(snap1, snap2, "Second build produced a diff — not idempotent")


if __name__ == "__main__":
    unittest.main()
