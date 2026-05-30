"""Tests for scripts/flow/check_virtual_coverage.py."""
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
import check_virtual_coverage as cvc  # noqa: E402
from curate_virtual import (  # noqa: E402
    TRIVIAL_ACCESSOR_ALLOWLIST,
    parse_creature_virtuals,
    parse_overrides,
    in_scope_virtuals,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_FAKE_CREATURE_H = """
class Creature {
public:
    virtual const std::string& getName() const = 0;
    virtual const std::string& getDescription() const = 0;
    virtual Player* getPlayer() { return nullptr; }
    virtual void onThink(uint32_t interval);
    virtual bool isAttackable() const { return true; }
};
"""

_FAKE_PLAYER_H = """
class Player final : public Creature {
public:
    const std::string& getName() const override;
    const std::string& getDescription() const override;
    Player* getPlayer() override { return this; }
    void onThink(uint32_t interval) override;
    bool isAttackable() const override;
};
"""

_FAKE_MONSTER_H = """
class Monster final : public Creature {
public:
    const std::string& getName() const override;
    const std::string& getDescription() const override;
    void onThink(uint32_t interval) override;
};
"""

_FAKE_NPC_H = """
class Npc final : public Creature {
public:
    const std::string& getName() const override;
};
"""


def _curated_virtual_edge(class_prefix: str, method: str, cpp_file: str) -> dict:
    return {
        "target": {"file": cpp_file, "qualified_name": f"{class_prefix}::{method}"},
        "kind": "dynamic",
        "confidence": "curated",
        "condition": f"dyntype == {class_prefix}",
        "order": 1,
    }


def _run_check(
    creature_h: str,
    subclass_headers: dict[str, tuple[str, str]],  # class -> (cpp_file, header_text)
    cpp_nodes: dict[str, dict],
    hdr_nodes: dict[str, dict],
) -> list[str]:
    """Run check logic against synthetic data."""
    all_v, _pure = parse_creature_virtuals(creature_h)
    scope = in_scope_virtuals(all_v)

    cpp_map: dict[str, list[dict]] = {k: [v] for k, v in cpp_nodes.items()}
    hdr_map: dict[str, list[dict]] = {k: [v] for k, v in hdr_nodes.items()}

    errors: list[str] = []
    for class_prefix, (cpp_file, header_text) in subclass_headers.items():
        overrides = parse_overrides(header_text, class_prefix)
        required = scope & overrides
        for method in sorted(required):
            base_qname = f"Creature::{method}"
            override_qname = f"{class_prefix}::{method}"
            condition_prefix = f"dyntype == {class_prefix}"
            nodes = cpp_map.get(base_qname) or hdr_map.get(base_qname)
            if not nodes:
                errors.append(f"MISSING_BASE_NODE: {base_qname}")
                continue
            if not cvc._any_has_curated_edge(nodes, override_qname, condition_prefix):
                errors.append(
                    f"MISSING_VIRTUAL_EDGE: {base_qname} → {override_qname}"
                )
    return errors


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

class TestTrivialAccessorExcluded(unittest.TestCase):
    def test_getPlayer_not_required(self):
        self.assertIn("getPlayer", TRIVIAL_ACCESSOR_ALLOWLIST)

    def test_in_scope_excludes_accessors(self):
        all_v, _ = parse_creature_virtuals(_FAKE_CREATURE_H)
        scope = in_scope_virtuals(all_v)
        self.assertNotIn("getPlayer", scope)
        self.assertIn("getName", scope)
        self.assertIn("onThink", scope)


class TestMissingVirtualEdgeFails(unittest.TestCase):
    def test_missing_edge_to_player_override_fails(self):
        subclasses = {
            "Player": ("src/player.cpp", _FAKE_PLAYER_H),
        }
        # Node for Creature::getName exists but has no edge to Player::getName
        hdr_nodes = {
            "Creature::getName": {"qualified_name": "Creature::getName", "edges": []},
            "Creature::getDescription": {"qualified_name": "Creature::getDescription",
                                          "edges": [_curated_virtual_edge("Player", "getDescription", "src/player.cpp")]},
            "Creature::onThink": {"qualified_name": "Creature::onThink",
                                   "edges": [_curated_virtual_edge("Player", "onThink", "src/player.cpp")]},
            "Creature::isAttackable": {"qualified_name": "Creature::isAttackable",
                                        "edges": [_curated_virtual_edge("Player", "isAttackable", "src/player.cpp")]},
        }
        errors = _run_check(_FAKE_CREATURE_H, subclasses, {}, hdr_nodes)
        self.assertTrue(
            any("MISSING_VIRTUAL_EDGE" in e and "getName" in e and "Player" in e for e in errors),
            errors,
        )

    def test_missing_base_node_fails(self):
        subclasses = {
            "Player": ("src/player.cpp", _FAKE_PLAYER_H),
        }
        # No node for Creature::getName at all
        hdr_nodes: dict = {}
        errors = _run_check(_FAKE_CREATURE_H, subclasses, {}, hdr_nodes)
        self.assertTrue(
            any("MISSING_BASE_NODE" in e and "getName" in e for e in errors),
            errors,
        )


class TestAllowlistedAccessorNotRequired(unittest.TestCase):
    def test_getPlayer_override_needs_no_edge(self):
        subclasses = {
            "Player": ("src/player.cpp", _FAKE_PLAYER_H),
        }
        # Provide edges for all in-scope methods but not getPlayer
        hdr_nodes = {
            "Creature::getName": {"qualified_name": "Creature::getName",
                                   "edges": [_curated_virtual_edge("Player", "getName", "src/player.cpp")]},
            "Creature::getDescription": {"qualified_name": "Creature::getDescription",
                                          "edges": [_curated_virtual_edge("Player", "getDescription", "src/player.cpp")]},
            "Creature::onThink": {"qualified_name": "Creature::onThink",
                                   "edges": [_curated_virtual_edge("Player", "onThink", "src/player.cpp")]},
            "Creature::isAttackable": {"qualified_name": "Creature::isAttackable",
                                        "edges": [_curated_virtual_edge("Player", "isAttackable", "src/player.cpp")]},
        }
        errors = _run_check(_FAKE_CREATURE_H, subclasses, {}, hdr_nodes)
        self.assertFalse(errors, errors)


class TestFullCoveragePasses(unittest.TestCase):
    def test_all_overrides_covered_passes(self):
        subclasses = {
            "Player":  ("src/player.cpp",  _FAKE_PLAYER_H),
            "Monster": ("src/monster.cpp", _FAKE_MONSTER_H),
            "Npc":     ("src/npc.cpp",     _FAKE_NPC_H),
        }
        # Player overrides: getName, getDescription, onThink, isAttackable
        # Monster overrides: getName, getDescription, onThink
        # Npc overrides: getName
        hdr_nodes = {
            "Creature::getName": {
                "qualified_name": "Creature::getName",
                "edges": [
                    _curated_virtual_edge("Player", "getName", "src/player.cpp"),
                    _curated_virtual_edge("Monster", "getName", "src/monster.cpp"),
                    _curated_virtual_edge("Npc", "getName", "src/npc.cpp"),
                ],
            },
            "Creature::getDescription": {
                "qualified_name": "Creature::getDescription",
                "edges": [
                    _curated_virtual_edge("Player", "getDescription", "src/player.cpp"),
                    _curated_virtual_edge("Monster", "getDescription", "src/monster.cpp"),
                ],
            },
            "Creature::onThink": {
                "qualified_name": "Creature::onThink",
                "edges": [
                    _curated_virtual_edge("Player", "onThink", "src/player.cpp"),
                    _curated_virtual_edge("Monster", "onThink", "src/monster.cpp"),
                ],
            },
            "Creature::isAttackable": {
                "qualified_name": "Creature::isAttackable",
                "edges": [
                    _curated_virtual_edge("Player", "isAttackable", "src/player.cpp"),
                ],
            },
        }
        errors = _run_check(_FAKE_CREATURE_H, subclasses, {}, hdr_nodes)
        self.assertFalse(errors, errors)

    def test_cpp_node_takes_priority_over_hdr_node(self):
        """Edge on cpp node satisfies the check even when hdr node also exists."""
        subclasses = {
            "Player": ("src/player.cpp", _FAKE_PLAYER_H),
        }
        cpp_nodes = {
            "Creature::onThink": {
                "qualified_name": "Creature::onThink",
                "edges": [_curated_virtual_edge("Player", "onThink", "src/player.cpp")],
            },
        }
        hdr_nodes = {
            "Creature::getName": {"qualified_name": "Creature::getName",
                                   "edges": [_curated_virtual_edge("Player", "getName", "src/player.cpp")]},
            "Creature::getDescription": {"qualified_name": "Creature::getDescription",
                                          "edges": [_curated_virtual_edge("Player", "getDescription", "src/player.cpp")]},
            "Creature::isAttackable": {"qualified_name": "Creature::isAttackable",
                                        "edges": [_curated_virtual_edge("Player", "isAttackable", "src/player.cpp")]},
        }
        errors = _run_check(_FAKE_CREATURE_H, subclasses, cpp_nodes, hdr_nodes)
        self.assertFalse(errors, errors)


class TestRealGraph(unittest.TestCase):
    """Integration test: real creature.h, real shards must pass."""

    def test_real_graph_passes(self):
        errors = cvc.check()
        self.assertFalse(
            errors,
            msg="Virtual coverage check failed:\n" + "\n".join(errors),
        )


if __name__ == "__main__":
    unittest.main()
