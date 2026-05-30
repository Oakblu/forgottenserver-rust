#!/usr/bin/env python3
"""Event/scheduler coverage check.

Asserts:
  1. Every non-NONE CREATURE_EVENT_* type in creatureevent.h has a curated edge
     from its executor to LuaScriptInterface::callFunction.
  2. Every known periodic scheduler task (checkCreatures, updateCreaturesPath,
     checkDecay) has a curated edge from Game::start.

Exits 0 on success, 1 on any violation.

Usage (from repo root):
    python3 scripts/flow/check_event_coverage.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

_ROOT = _HERE.parent.parent

# Executor method for each event type (must match curate_events.py)
_CREATURE_EVENT_EXECUTORS: dict[str, str] = {
    "CREATURE_EVENT_LOGIN":           "CreatureEvent::executeOnLogin",
    "CREATURE_EVENT_LOGOUT":          "CreatureEvent::executeOnLogout",
    "CREATURE_EVENT_RECONNECT":       "CreatureEvent::executeOnReconnect",
    "CREATURE_EVENT_THINK":           "CreatureEvent::executeOnThink",
    "CREATURE_EVENT_PREPAREDEATH":    "CreatureEvent::executeOnPrepareDeath",
    "CREATURE_EVENT_DEATH":           "CreatureEvent::executeOnDeath",
    "CREATURE_EVENT_KILL":            "CreatureEvent::executeOnKill",
    "CREATURE_EVENT_ADVANCE":         "CreatureEvent::executeAdvance",
    "CREATURE_EVENT_MODALWINDOW":     "CreatureEvent::executeModalWindow",
    "CREATURE_EVENT_TEXTEDIT":        "CreatureEvent::executeTextEdit",
    "CREATURE_EVENT_HEALTHCHANGE":    "CreatureEvent::executeHealthChange",
    "CREATURE_EVENT_MANACHANGE":      "CreatureEvent::executeManaChange",
    "CREATURE_EVENT_EXTENDED_OPCODE": "CreatureEvent::executeExtendedOpcode",
}

_PERIODIC_TASKS = ["Game::checkCreatures", "Game::updateCreaturesPath", "Game::checkDecay"]
_LUA_QNAME = "LuaScriptInterface::callFunction"

_ENUM_VALUE = re.compile(r"\b(CREATURE_EVENT_\w+)\b")


def _parse_creature_event_types(header_text: str) -> set[str]:
    """Extract all CREATURE_EVENT_* names from the header, excluding NONE."""
    types = _ENUM_VALUE.findall(header_text)
    return {t for t in types if t != "CREATURE_EVENT_NONE"}


def _load_nodes(shard_path: Path) -> dict[str, dict]:
    """Return {qualified_name: node} from a shard."""
    doc = ledger_io.load(shard_path.read_text()) if shard_path.exists() else {}
    return {n["qualified_name"]: n for n in (doc or {}).get("nodes") or []}


def _has_curated_edge_to(node: dict, target_qname: str) -> bool:
    for e in node.get("edges") or []:
        if (e.get("confidence") == "curated"
                and e.get("target", {}).get("qualified_name") == target_qname):
            return True
    return False


def check(root: Path = _ROOT) -> list[str]:
    errors: list[str] = []
    nodes_dir = root / "flow_graph" / "nodes"

    # --- 1. Creature event executors → LuaScriptInterface::callFunction ---
    header = (root / "forgottenserver-upstream" / "src" / "creatureevent.h").read_text()
    declared_types = _parse_creature_event_types(header)

    creature_nodes = _load_nodes(nodes_dir / "creatureevent.yml")

    for event_type in sorted(declared_types):
        executor = _CREATURE_EVENT_EXECUTORS.get(event_type)
        if executor is None:
            errors.append(
                f"UNMAPPED_EVENT_TYPE: {event_type} has no executor mapping in "
                f"_CREATURE_EVENT_EXECUTORS"
            )
            continue
        node = creature_nodes.get(executor)
        if node is None:
            errors.append(
                f"MISSING_NODE: executor {executor!r} for {event_type} "
                f"not found in creatureevent.yml"
            )
            continue
        if not _has_curated_edge_to(node, _LUA_QNAME):
            errors.append(
                f"MISSING_LUA_EDGE: {executor} ({event_type}) has no curated edge "
                f"to {_LUA_QNAME}"
            )

    # --- 2. Periodic scheduler tasks reachable from Game::start ---
    game_nodes = _load_nodes(nodes_dir / "game.yml")
    start_node = game_nodes.get("Game::start")
    if start_node is None:
        errors.append("MISSING_NODE: Game::start not found in game.yml")
    else:
        for task_qname in _PERIODIC_TASKS:
            if not _has_curated_edge_to(start_node, task_qname):
                errors.append(
                    f"MISSING_PERIODIC_EDGE: Game::start has no curated edge to {task_qname}"
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
        print(f"\n{len(errors)} event coverage error(s).", file=sys.stderr)
        return 1

    print("Event coverage: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
