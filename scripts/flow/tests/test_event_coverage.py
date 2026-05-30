"""Tests for scripts/flow/check_event_coverage.py."""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

_TESTS_DIR = Path(__file__).parent
_FLOW_DIR = _TESTS_DIR.parent
_LEDGER_DIR = _FLOW_DIR.parent / "ledger"
sys.path.insert(0, str(_LEDGER_DIR))
sys.path.insert(0, str(_FLOW_DIR))

import ledger_io  # noqa: E402
import check_event_coverage as cec  # noqa: E402


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_FAKE_HEADER = """
enum CreatureEventType {
    CREATURE_EVENT_NONE,
    CREATURE_EVENT_LOGIN,
    CREATURE_EVENT_LOGOUT,
    CREATURE_EVENT_THINK,
};
"""

_FAKE_EXECUTORS: dict[str, str] = {
    "CREATURE_EVENT_LOGIN":  "CreatureEvent::executeOnLogin",
    "CREATURE_EVENT_LOGOUT": "CreatureEvent::executeOnLogout",
    "CREATURE_EVENT_THINK":  "CreatureEvent::executeOnThink",
}


def _curated_lua_edge() -> dict:
    return {
        "target": {"file": "src/luascript.cpp", "qualified_name": "LuaScriptInterface::callFunction"},
        "kind": "dynamic",
        "confidence": "curated",
        "condition": "Lua boundary",
        "order": 1,
    }


def _curated_task_edge(qname: str) -> dict:
    return {
        "target": {"file": "src/game.cpp", "qualified_name": qname},
        "kind": "dynamic",
        "confidence": "curated",
        "condition": "scheduled",
        "order": 1,
    }


def _run_check(header: str, executor_map: dict[str, str],
               creature_nodes: dict[str, dict],
               game_start_node: dict | None) -> list[str]:
    """Run the core check logic against synthetic data."""
    errors: list[str] = []
    declared = cec._parse_creature_event_types(header)

    for event_type in sorted(declared):
        executor = executor_map.get(event_type)
        if executor is None:
            errors.append(f"UNMAPPED_EVENT_TYPE: {event_type}")
            continue
        node = creature_nodes.get(executor)
        if node is None:
            errors.append(f"MISSING_NODE: executor {executor!r} for {event_type}")
            continue
        if not cec._has_curated_edge_to(node, cec._LUA_QNAME):
            errors.append(f"MISSING_LUA_EDGE: {executor} ({event_type})")

    if game_start_node is None:
        errors.append("MISSING_NODE: Game::start not found in game.yml")
    else:
        for task in cec._PERIODIC_TASKS:
            if not cec._has_curated_edge_to(game_start_node, task):
                errors.append(f"MISSING_PERIODIC_EDGE: Game::start has no curated edge to {task}")

    return errors


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

class TestMissingEventEdgeFails(unittest.TestCase):
    def test_executor_without_lua_edge_fails(self):
        creature_nodes = {
            "CreatureEvent::executeOnLogin": {"qualified_name": "CreatureEvent::executeOnLogin", "edges": []},
            "CreatureEvent::executeOnLogout": {"qualified_name": "CreatureEvent::executeOnLogout", "edges": [_curated_lua_edge()]},
            "CreatureEvent::executeOnThink": {"qualified_name": "CreatureEvent::executeOnThink", "edges": [_curated_lua_edge()]},
        }
        game_start = {"qualified_name": "Game::start", "edges": [_curated_task_edge(t) for t in cec._PERIODIC_TASKS]}
        errors = _run_check(_FAKE_HEADER, _FAKE_EXECUTORS, creature_nodes, game_start)
        self.assertTrue(any("MISSING_LUA_EDGE" in e and "executeOnLogin" in e for e in errors), errors)

    def test_missing_node_fails(self):
        creature_nodes = {
            # executeOnLogin missing entirely
            "CreatureEvent::executeOnLogout": {"qualified_name": "CreatureEvent::executeOnLogout", "edges": [_curated_lua_edge()]},
            "CreatureEvent::executeOnThink": {"qualified_name": "CreatureEvent::executeOnThink", "edges": [_curated_lua_edge()]},
        }
        game_start = {"qualified_name": "Game::start", "edges": [_curated_task_edge(t) for t in cec._PERIODIC_TASKS]}
        errors = _run_check(_FAKE_HEADER, _FAKE_EXECUTORS, creature_nodes, game_start)
        self.assertTrue(any("MISSING_NODE" in e for e in errors), errors)


class TestMissingPeriodicEdgeFails(unittest.TestCase):
    def test_missing_periodic_task_edge_fails(self):
        creature_nodes = {
            e: {"qualified_name": e, "edges": [_curated_lua_edge()]}
            for e in _FAKE_EXECUTORS.values()
        }
        # Game::start missing checkDecay edge
        game_start = {
            "qualified_name": "Game::start",
            "edges": [_curated_task_edge("Game::checkCreatures"), _curated_task_edge("Game::updateCreaturesPath")],
        }
        errors = _run_check(_FAKE_HEADER, _FAKE_EXECUTORS, creature_nodes, game_start)
        self.assertTrue(any("MISSING_PERIODIC_EDGE" in e and "checkDecay" in e for e in errors), errors)


class TestFullCoveragePasses(unittest.TestCase):
    def test_all_events_and_tasks_covered_passes(self):
        creature_nodes = {
            e: {"qualified_name": e, "edges": [_curated_lua_edge()]}
            for e in _FAKE_EXECUTORS.values()
        }
        game_start = {
            "qualified_name": "Game::start",
            "edges": [_curated_task_edge(t) for t in cec._PERIODIC_TASKS],
        }
        errors = _run_check(_FAKE_HEADER, _FAKE_EXECUTORS, creature_nodes, game_start)
        self.assertFalse(errors, errors)


class TestRealGraph(unittest.TestCase):
    """Integration test: the real creatureevent.h and real shards must pass."""

    def test_real_graph_passes(self):
        errors = cec.check()
        self.assertFalse(
            errors,
            msg="Event coverage check failed:\n" + "\n".join(errors),
        )


if __name__ == "__main__":
    unittest.main()
