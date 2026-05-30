#!/usr/bin/env python3
"""Curate dynamic event/scheduler edges in the flow graph.

Adds/updates curated edges for:
  - Game::start → periodic scheduler ticks (checkCreatures, updateCreaturesPath, checkDecay)
  - Periodic tick functions → self (re-arm pattern)
  - Game::setGameState → Game::shutdown (deferred via addTask on GAME_STATE_SHUTDOWN)
  - Game::playerAutoWalk → Game::internalMoveCreature (deferred via scheduler)
  - CreatureEvent::executeOnXxx → LuaScriptInterface::callFunction (one per event type)
  - GlobalEvent::executeEvent/executeRecord → LuaScriptInterface::callFunction
  - TalkAction::executeSay → LuaScriptInterface::callFunction
  - MoveEvent::executeStep/executeEquip/executeAddRemItem → LuaScriptInterface::callFunction

Idempotent: running twice produces no diff.

Usage (from repo root):
    python3 scripts/flow/curate_events.py
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

_ROOT = _HERE.parent.parent
_LUA_DISPATCH_FILE = "src/luascript.cpp"
_LUA_DISPATCH_QNAME = "LuaScriptInterface::callFunction"

# (event_type_constant, executor_qname)
_CREATURE_EVENT_EXECUTORS: list[tuple[str, str]] = [
    ("CREATURE_EVENT_LOGIN",           "CreatureEvent::executeOnLogin"),
    ("CREATURE_EVENT_LOGOUT",          "CreatureEvent::executeOnLogout"),
    ("CREATURE_EVENT_RECONNECT",       "CreatureEvent::executeOnReconnect"),
    ("CREATURE_EVENT_THINK",           "CreatureEvent::executeOnThink"),
    ("CREATURE_EVENT_PREPAREDEATH",    "CreatureEvent::executeOnPrepareDeath"),
    ("CREATURE_EVENT_DEATH",           "CreatureEvent::executeOnDeath"),
    ("CREATURE_EVENT_KILL",            "CreatureEvent::executeOnKill"),
    ("CREATURE_EVENT_ADVANCE",         "CreatureEvent::executeAdvance"),
    ("CREATURE_EVENT_MODALWINDOW",     "CreatureEvent::executeModalWindow"),
    ("CREATURE_EVENT_TEXTEDIT",        "CreatureEvent::executeTextEdit"),
    ("CREATURE_EVENT_HEALTHCHANGE",    "CreatureEvent::executeHealthChange"),
    ("CREATURE_EVENT_MANACHANGE",      "CreatureEvent::executeManaChange"),
    ("CREATURE_EVENT_EXTENDED_OPCODE", "CreatureEvent::executeExtendedOpcode"),
]

# ---------------------------------------------------------------------------
# Helpers (same pattern as curate_network.py)
# ---------------------------------------------------------------------------

EdgeKey = tuple[str, str, str]  # (file, qname, condition)


def _edge_key(edge: dict) -> EdgeKey:
    tgt = edge["target"]
    return (tgt["file"], tgt["qualified_name"], edge.get("condition", ""))


def _make_edge(file: str, qname: str, condition: str, order: int,
               kind: str = "dynamic") -> dict:
    e: dict[str, Any] = {
        "target": {"file": file, "qualified_name": qname},
        "kind": kind,
        "confidence": "curated",
        "order": order,
    }
    if condition:
        e["condition"] = condition
    return e


def _merge(existing: list[dict], new_curated: list[dict]) -> list[dict]:
    keys = {_edge_key(e) for e in new_curated}
    kept = [e for e in existing if _edge_key(e) not in keys]
    return kept + new_curated


def _load(path: Path) -> list[dict]:
    doc = ledger_io.load(path.read_text()) or {}
    return list(doc.get("nodes") or [])


def _write(path: Path, nodes: list[dict]) -> bool:
    existing = ledger_io.load(path.read_text()) if path.exists() else {}
    if (existing or {}).get("nodes") == nodes:
        return False
    path.write_text(ledger_io.dump({"nodes": nodes}))
    return True


def _update(nodes: list[dict], file: str, qname: str, new_edges: list[dict]) -> bool:
    for n in nodes:
        if n["file"] == file and n["qualified_name"] == qname:
            old = dict(n)
            n["edges"] = _merge(list(n.get("edges") or []), new_edges)
            return n != old
    return False


# ---------------------------------------------------------------------------
# game.yml — tasks 1.2 + 1.3
# ---------------------------------------------------------------------------

def _update_game(nodes_dir: Path) -> bool:
    path = nodes_dir / "game.yml"
    nodes = _load(path)

    # Game::start → periodic tick schedulers (task 1.2)
    _update(nodes, "src/game.cpp", "Game::start", [
        _make_edge("src/game.cpp", "Game::checkCreatures",
                   "scheduled every EVENT_CREATURE_THINK_INTERVAL ms via g_scheduler.addEvent", 1),
        _make_edge("src/game.cpp", "Game::updateCreaturesPath",
                   "scheduled every PATHFINDING_INTERVAL ms via g_scheduler.addEvent", 2),
        _make_edge("src/game.cpp", "Game::checkDecay",
                   "scheduled every EVENT_DECAYINTERVAL ms via g_scheduler.addEvent", 3),
    ])

    # Periodic ticks self-re-arm (task 1.3)
    _update(nodes, "src/game.cpp", "Game::checkCreatures", [
        _make_edge("src/game.cpp", "Game::checkCreatures",
                   "periodic: re-arms via g_scheduler.addEvent(EVENT_CREATURE_THINK_INTERVAL)", 1),
    ])
    _update(nodes, "src/game.cpp", "Game::updateCreaturesPath", [
        _make_edge("src/game.cpp", "Game::updateCreaturesPath",
                   "periodic: re-arms via g_scheduler.addEvent(PATHFINDING_INTERVAL)", 1),
    ])
    _update(nodes, "src/game.cpp", "Game::checkDecay", [
        _make_edge("src/game.cpp", "Game::checkDecay",
                   "periodic: re-arms via g_scheduler.addEvent(EVENT_DECAYINTERVAL)", 1),
    ])

    # Game::setGameState → Game::shutdown deferred (task 1.3)
    _update(nodes, "src/game.cpp", "Game::setGameState", [
        _make_edge("src/game.cpp", "Game::shutdown",
                   "deferred via g_dispatcher.addTask when newState == GAME_STATE_SHUTDOWN", 1),
    ])

    # Game::playerAutoWalk → Game::internalMoveCreature deferred (task 1.3)
    _update(nodes, "src/game.cpp", "Game::playerAutoWalk", [
        _make_edge("src/game.cpp", "Game::internalMoveCreature",
                   "deferred via g_scheduler.addEvent(RANGE_MOVE_CREATURE_INTERVAL)", 1),
    ])

    return _write(path, nodes)


# ---------------------------------------------------------------------------
# creatureevent.yml — task 2.1
# ---------------------------------------------------------------------------

def _update_creatureevent(nodes_dir: Path) -> bool:
    path = nodes_dir / "creatureevent.yml"
    nodes = _load(path)
    changed = False
    for event_type, executor_qname in _CREATURE_EVENT_EXECUTORS:
        changed |= _update(nodes, "src/creatureevent.cpp", executor_qname, [
            _make_edge(
                _LUA_DISPATCH_FILE, _LUA_DISPATCH_QNAME,
                f"Lua boundary — scriptInterface->callFunction ({event_type})", 1,
            ),
        ])
    if _write(path, nodes):
        return True
    return changed


# ---------------------------------------------------------------------------
# globalevent.yml — task 2.2
# ---------------------------------------------------------------------------

def _update_globalevent(nodes_dir: Path) -> bool:
    path = nodes_dir / "globalevent.yml"
    nodes = _load(path)
    _update(nodes, "src/globalevent.cpp", "GlobalEvent::executeEvent", [
        _make_edge(_LUA_DISPATCH_FILE, _LUA_DISPATCH_QNAME,
                   "Lua boundary — scriptInterface->callFunction (GlobalEvent)", 1),
    ])
    _update(nodes, "src/globalevent.cpp", "GlobalEvent::executeRecord", [
        _make_edge(_LUA_DISPATCH_FILE, _LUA_DISPATCH_QNAME,
                   "Lua boundary — scriptInterface->callFunction (GlobalEvent record)", 1),
    ])
    return _write(path, nodes)


# ---------------------------------------------------------------------------
# talkaction.yml — task 2.2
# ---------------------------------------------------------------------------

def _update_talkaction(nodes_dir: Path) -> bool:
    path = nodes_dir / "talkaction.yml"
    nodes = _load(path)
    _update(nodes, "src/talkaction.cpp", "TalkAction::executeSay", [
        _make_edge(_LUA_DISPATCH_FILE, _LUA_DISPATCH_QNAME,
                   "Lua boundary — scriptInterface->callFunction (TalkAction)", 1),
    ])
    return _write(path, nodes)


# ---------------------------------------------------------------------------
# movement.yml — task 2.2
# ---------------------------------------------------------------------------

def _update_movement(nodes_dir: Path) -> bool:
    path = nodes_dir / "movement.yml"
    nodes = _load(path)
    for executor_qname, label in [
        ("MoveEvent::executeStep",        "MoveEvent step"),
        ("MoveEvent::executeEquip",       "MoveEvent equip"),
        ("MoveEvent::executeAddRemItem",  "MoveEvent add/rem item"),
    ]:
        _update(nodes, "src/movement.cpp", executor_qname, [
            _make_edge(_LUA_DISPATCH_FILE, _LUA_DISPATCH_QNAME,
                       f"Lua boundary — scriptInterface->callFunction ({label})", 1),
        ])
    return _write(path, nodes)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def curate(root: Path = _ROOT) -> None:
    nodes_dir = root / "flow_graph" / "nodes"
    results = {
        "game":          _update_game(nodes_dir),
        "creatureevent": _update_creatureevent(nodes_dir),
        "globalevent":   _update_globalevent(nodes_dir),
        "talkaction":    _update_talkaction(nodes_dir),
        "movement":      _update_movement(nodes_dir),
    }
    written = sum(1 for v in results.values() if v)
    n_creature = len(_CREATURE_EVENT_EXECUTORS)
    print(
        f"Event curation: {n_creature} creature-event edges, "
        f"3 scheduler ticks, 3 periodic re-arms, "
        f"2 global-event + 1 talk-action + 3 move-event Lua boundary edges — "
        f"{written} shard(s) written"
    )


if __name__ == "__main__":
    curate()
