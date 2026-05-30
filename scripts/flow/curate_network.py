#!/usr/bin/env python3
"""Curate dynamic network-protocol edges in the flow graph.

Adds/updates curated edges for:
  - ProtocolGame::parsePacket opcode dispatch table
  - ProtocolGame::onRecvFirstMessage → ProtocolGame::login
  - ProtocolStatus::onRecvFirstMessage → sendStatusString / sendInfo
  - Connection::parsePacket → onRecvFirstMessage (virtual dispatch)
  - ServiceManager::run → Connection::parsePacket (async accept)
  - startServer → ServiceManager::run

Idempotent: running twice produces no diff.

Usage (from repo root):
    python3 scripts/flow/curate_network.py
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
import ledger_io  # noqa: E402

_ROOT = _HERE.parent.parent

# ---------------------------------------------------------------------------
# Active opcodes: (opcode_hex, handler_file, handler_qname, via_addTask)
# Order matches the switch statement in protocolgame.cpp:531
# ---------------------------------------------------------------------------
_PARSE_PACKET_OPCODES: list[tuple[str, str, str, bool]] = [
    ("0x14", "src/protocolgame.cpp", "ProtocolGame::logout", True),
    ("0x1D", "src/game.cpp", "Game::playerReceivePingBack", True),
    ("0x1E", "src/game.cpp", "Game::playerReceivePing", True),
    ("0x32", "src/protocolgame.cpp", "ProtocolGame::parseExtendedOpcode", False),
    ("0x64", "src/protocolgame.cpp", "ProtocolGame::parseAutoWalk", False),
    ("0x65", "src/game.cpp", "Game::playerMove", True),    # NORTH
    ("0x66", "src/game.cpp", "Game::playerMove", True),    # EAST
    ("0x67", "src/game.cpp", "Game::playerMove", True),    # SOUTH
    ("0x68", "src/game.cpp", "Game::playerMove", True),    # WEST
    ("0x69", "src/game.cpp", "Game::playerStopAutoWalk", True),
    ("0x6A", "src/game.cpp", "Game::playerMove", True),    # NORTHEAST
    ("0x6B", "src/game.cpp", "Game::playerMove", True),    # SOUTHEAST
    ("0x6C", "src/game.cpp", "Game::playerMove", True),    # SOUTHWEST
    ("0x6D", "src/game.cpp", "Game::playerMove", True),    # NORTHWEST
    ("0x6F", "src/game.cpp", "Game::playerTurn", True),    # NORTH
    ("0x70", "src/game.cpp", "Game::playerTurn", True),    # EAST
    ("0x71", "src/game.cpp", "Game::playerTurn", True),    # SOUTH
    ("0x72", "src/game.cpp", "Game::playerTurn", True),    # WEST
    ("0x77", "src/protocolgame.cpp", "ProtocolGame::parseEquipObject", False),
    ("0x78", "src/protocolgame.cpp", "ProtocolGame::parseThrow", False),
    ("0x79", "src/protocolgame.cpp", "ProtocolGame::parseLookInShop", False),
    ("0x7A", "src/protocolgame.cpp", "ProtocolGame::parsePlayerPurchase", False),
    ("0x7B", "src/protocolgame.cpp", "ProtocolGame::parsePlayerSale", False),
    ("0x7C", "src/game.cpp", "Game::playerCloseShop", True),
    ("0x7D", "src/protocolgame.cpp", "ProtocolGame::parseRequestTrade", False),
    ("0x7E", "src/protocolgame.cpp", "ProtocolGame::parseLookInTrade", False),
    ("0x7F", "src/game.cpp", "Game::playerAcceptTrade", True),
    ("0x80", "src/game.cpp", "Game::playerCloseTrade", True),
    ("0x82", "src/protocolgame.cpp", "ProtocolGame::parseUseItem", False),
    ("0x83", "src/protocolgame.cpp", "ProtocolGame::parseUseItemEx", False),
    ("0x84", "src/protocolgame.cpp", "ProtocolGame::parseUseWithCreature", False),
    ("0x85", "src/protocolgame.cpp", "ProtocolGame::parseRotateItem", False),
    ("0x86", "src/protocolgame.cpp", "ProtocolGame::parseEditPodiumRequest", False),
    ("0x87", "src/protocolgame.cpp", "ProtocolGame::parseCloseContainer", False),
    ("0x88", "src/protocolgame.cpp", "ProtocolGame::parseUpArrowContainer", False),
    ("0x89", "src/protocolgame.cpp", "ProtocolGame::parseTextWindow", False),
    ("0x8A", "src/protocolgame.cpp", "ProtocolGame::parseHouseWindow", False),
    ("0x8B", "src/protocolgame.cpp", "ProtocolGame::parseWrapItem", False),
    ("0x8C", "src/protocolgame.cpp", "ProtocolGame::parseLookAt", False),
    ("0x8D", "src/protocolgame.cpp", "ProtocolGame::parseLookInBattleList", False),
    # 0x8E noop
    ("0x96", "src/protocolgame.cpp", "ProtocolGame::parseSay", False),
    ("0x97", "src/game.cpp", "Game::playerRequestChannels", True),
    ("0x98", "src/protocolgame.cpp", "ProtocolGame::parseOpenChannel", False),
    ("0x99", "src/protocolgame.cpp", "ProtocolGame::parseCloseChannel", False),
    ("0x9A", "src/protocolgame.cpp", "ProtocolGame::parseOpenPrivateChannel", False),
    ("0x9E", "src/game.cpp", "Game::playerCloseNpcChannel", True),
    ("0xA0", "src/protocolgame.cpp", "ProtocolGame::parseFightModes", False),
    ("0xA1", "src/protocolgame.cpp", "ProtocolGame::parseAttack", False),
    ("0xA2", "src/protocolgame.cpp", "ProtocolGame::parseFollow", False),
    ("0xA3", "src/protocolgame.cpp", "ProtocolGame::parseInviteToParty", False),
    ("0xA4", "src/protocolgame.cpp", "ProtocolGame::parseJoinParty", False),
    ("0xA5", "src/protocolgame.cpp", "ProtocolGame::parseRevokePartyInvite", False),
    ("0xA6", "src/protocolgame.cpp", "ProtocolGame::parsePassPartyLeadership", False),
    ("0xA7", "src/game.cpp", "Game::playerLeaveParty", True),
    ("0xA8", "src/protocolgame.cpp", "ProtocolGame::parseEnableSharedPartyExperience", False),
    ("0xAA", "src/game.cpp", "Game::playerCreatePrivateChannel", True),
    ("0xAB", "src/protocolgame.cpp", "ProtocolGame::parseChannelInvite", False),
    ("0xAC", "src/protocolgame.cpp", "ProtocolGame::parseChannelExclude", False),
    ("0xBE", "src/game.cpp", "Game::playerCancelAttackAndFollow", True),
    # 0xC9 noop
    ("0xCA", "src/protocolgame.cpp", "ProtocolGame::parseUpdateContainer", False),
    ("0xCB", "src/protocolgame.cpp", "ProtocolGame::parseBrowseField", False),
    ("0xCC", "src/protocolgame.cpp", "ProtocolGame::parseSeekInContainer", False),
    ("0xD2", "src/game.cpp", "Game::playerRequestOutfit", True),
    ("0xD3", "src/protocolgame.cpp", "ProtocolGame::parseSetOutfit", False),
    ("0xDC", "src/protocolgame.cpp", "ProtocolGame::parseAddVip", False),
    ("0xDD", "src/protocolgame.cpp", "ProtocolGame::parseRemoveVip", False),
    ("0xDE", "src/protocolgame.cpp", "ProtocolGame::parseEditVip", False),
    # 0xE7 noop
    ("0xE8", "src/protocolgame.cpp", "ProtocolGame::parseDebugAssert", False),
    ("0xF2", "src/protocolgame.cpp", "ProtocolGame::parseRuleViolationReport", False),
    # 0xF3 noop
    ("0xF4", "src/protocolgame.cpp", "ProtocolGame::parseMarketLeave", False),
    ("0xF5", "src/protocolgame.cpp", "ProtocolGame::parseMarketBrowse", False),
    ("0xF6", "src/protocolgame.cpp", "ProtocolGame::parseMarketCreateOffer", False),
    ("0xF7", "src/protocolgame.cpp", "ProtocolGame::parseMarketCancelOffer", False),
    ("0xF8", "src/protocolgame.cpp", "ProtocolGame::parseMarketAcceptOffer", False),
    ("0xF9", "src/protocolgame.cpp", "ProtocolGame::parseModalWindowAnswer", False),
]

_DISABLED_OPCODES: list[dict] = [
    {"opcode": "0x28", "comment": "stash withdraw"},
    {"opcode": "0x2A", "comment": "bestiary tracker"},
    {"opcode": "0x2C", "comment": "team finder (leader)"},
    {"opcode": "0x2D", "comment": "team finder (member)"},
    {"opcode": "0x8F", "comment": "quick loot"},
    {"opcode": "0x90", "comment": "loot container"},
    {"opcode": "0x91", "comment": "update loot whitelist"},
    {"opcode": "0x92", "comment": "request locker items"},
    {"opcode": "0xB1", "comment": "request highscores"},
    {"opcode": "0xC7", "comment": "request tournament leaderboard"},
    {"opcode": "0xCD", "comment": "request inspect window"},
    {"opcode": "0xD5", "comment": "apply imbuement"},
    {"opcode": "0xD6", "comment": "clear imbuement"},
    {"opcode": "0xD7", "comment": "close imbuing window"},
    {"opcode": "0xDF", "comment": "premium shop"},
    {"opcode": "0xE0", "comment": "premium shop"},
    {"opcode": "0xE4", "comment": "buy charm rune"},
    {"opcode": "0xE5", "comment": "request character info (cyclopedia)"},
    {"opcode": "0xE6", "comment": "parse bug report"},
    {"opcode": "0xEF", "comment": "request store coins transfer"},
    {"opcode": "0xFA", "comment": "store window open"},
    {"opcode": "0xFB", "comment": "store window click"},
    {"opcode": "0xFC", "comment": "store window buy"},
    {"opcode": "0xFD", "comment": "store window history 1"},
    {"opcode": "0xFE", "comment": "store window history 2"},
]

_NOOP_OPCODES: list[dict] = [
    {"opcode": "0x8E", "comment": "join aggression — acknowledged, no action"},
    {"opcode": "0xC9", "comment": "update tile — acknowledged, no action"},
    {"opcode": "0xE7", "comment": "thank you — acknowledged, no action"},
    {"opcode": "0xF3", "comment": "get object info — acknowledged, no action"},
]

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

EdgeKey = tuple[str, str, str]  # (file, qname, condition)


def _edge_key(edge: dict) -> EdgeKey:
    tgt = edge["target"]
    return (tgt["file"], tgt["qualified_name"], edge.get("condition", ""))


def _curated_condition(opcode: str, via_task: bool) -> str:
    base = f"recvbyte == {opcode}"
    if via_task:
        return f"{base} (dispatched via g_dispatcher.addTask)"
    return base


def _make_opcode_edge(opcode: str, file: str, qname: str, via_task: bool, order: int) -> dict:
    return {
        "target": {"file": file, "qualified_name": qname},
        "kind": "dynamic",
        "confidence": "curated",
        "condition": _curated_condition(opcode, via_task),
        "order": order,
    }


def _make_curated_edge(file: str, qname: str, condition: str, order: int,
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


def _merge_edges(existing: list[dict], new_curated: list[dict]) -> list[dict]:
    """Merge curated edges, replacing existing edges for the same (file, qname, condition)."""
    curated_keys = {_edge_key(e) for e in new_curated}
    kept = [e for e in existing if _edge_key(e) not in curated_keys]
    return kept + new_curated


# ---------------------------------------------------------------------------
# Shard I/O
# ---------------------------------------------------------------------------

def _load(path: Path) -> list[dict]:
    doc = ledger_io.load(path.read_text()) or {}
    return list(doc.get("nodes") or [])


def _write(path: Path, nodes: list[dict]) -> bool:
    existing = ledger_io.load(path.read_text()) if path.exists() else {}
    if (existing or {}).get("nodes") == nodes:
        return False
    path.write_text(ledger_io.dump({"nodes": nodes}))
    return True


def _update_node(nodes: list[dict], file: str, qname: str,
                 new_edges: list[dict], extra: dict | None = None) -> bool:
    """Update the node (file, qname) in place; return True if changed."""
    for n in nodes:
        if n["file"] == file and n["qualified_name"] == qname:
            old = dict(n)
            n["edges"] = _merge_edges(list(n.get("edges") or []), new_edges)
            if extra:
                n.update(extra)
            return n != old
    return False


# ---------------------------------------------------------------------------
# Per-shard updates
# ---------------------------------------------------------------------------

def _update_protocolgame(nodes_dir: Path) -> bool:
    path = nodes_dir / "protocolgame.yml"
    nodes = _load(path)

    # Build parsePacket curated edges
    opcode_edges = [
        _make_opcode_edge(op, f, qn, via, i + 1)
        for i, (op, f, qn, via) in enumerate(_PARSE_PACKET_OPCODES)
    ]
    _update_node(
        nodes,
        file="src/protocolgame.cpp",
        qname="ProtocolGame::parsePacket",
        new_edges=opcode_edges,
        extra={
            "disabled_opcodes": _DISABLED_OPCODES,
            "noop_opcodes": _NOOP_OPCODES,
        },
    )

    # onRecvFirstMessage → login (via addTask on success path)
    login_edges = [
        _make_curated_edge(
            "src/protocolgame.cpp", "ProtocolGame::login",
            "dispatched via g_dispatcher.addTask on successful session auth",
            order=5,
        ),
    ]
    _update_node(
        nodes,
        file="src/protocolgame.cpp",
        qname="ProtocolGame::onRecvFirstMessage",
        new_edges=login_edges,
    )

    return _write(path, nodes)


def _update_protocolstatus(nodes_dir: Path) -> bool:
    path = nodes_dir / "protocolstatus.yml"
    nodes = _load(path)

    status_edges = [
        _make_curated_edge(
            "src/protocolstatus.cpp", "ProtocolStatus::sendStatusString",
            "recvbyte == 0xFF (XML info protocol, dispatched via g_dispatcher.addTask)",
            order=1,
        ),
        _make_curated_edge(
            "src/protocolstatus.cpp", "ProtocolStatus::sendInfo",
            "recvbyte == 0x01 (server info protocol, dispatched via g_dispatcher.addTask)",
            order=2,
        ),
    ]
    _update_node(
        nodes,
        file="src/protocolstatus.cpp",
        qname="ProtocolStatus::onRecvFirstMessage",
        new_edges=status_edges,
    )

    return _write(path, nodes)


def _update_connection(nodes_dir: Path) -> bool:
    path = nodes_dir / "connection.yml"
    nodes = _load(path)

    # Connection::parsePacket dispatches to the protocol's virtual onRecvFirstMessage
    # on the first message, or to protocol->parsePacket on subsequent messages.
    dispatch_edges = [
        _make_curated_edge(
            "src/protocolgame.cpp", "ProtocolGame::onRecvFirstMessage",
            "first message on game world port — virtual dispatch protocol->onRecvFirstMessage",
            order=1,
        ),
        _make_curated_edge(
            "src/protocolstatus.cpp", "ProtocolStatus::onRecvFirstMessage",
            "first message on status port — virtual dispatch protocol->onRecvFirstMessage",
            order=2,
        ),
        _make_curated_edge(
            "src/protocolgame.cpp", "ProtocolGame::parsePacket",
            "subsequent messages — virtual dispatch protocol->parsePacket",
            order=3,
        ),
    ]
    _update_node(
        nodes,
        file="src/connection.cpp",
        qname="Connection::parsePacket",
        new_edges=dispatch_edges,
    )

    return _write(path, nodes)


def _update_server(nodes_dir: Path) -> bool:
    path = nodes_dir / "server.yml"
    nodes = _load(path)

    # ServiceManager::run → Connection::parsePacket via io_context async accept loop
    run_edges = [
        _make_curated_edge(
            "src/connection.cpp", "Connection::parsePacket",
            "io_context async accept loop — boost::asio schedules Connection::parsePacket per inbound packet",
            order=1,
        ),
    ]
    _update_node(
        nodes,
        file="src/server.cpp",
        qname="ServiceManager::run",
        new_edges=run_edges,
    )

    return _write(path, nodes)


def _update_otserv(nodes_dir: Path) -> bool:
    path = nodes_dir / "otserv.yml"
    nodes = _load(path)

    # startServer → ServiceManager::run (local variable call — missed by static extractor)
    run_edge = [
        _make_curated_edge(
            "src/server.cpp", "ServiceManager::run",
            "serviceManager.is_running() check passes after mainLoader completes",
            order=7,
            kind="static",
        ),
    ]
    _update_node(
        nodes,
        file="src/otserv.cpp",
        qname="startServer",
        new_edges=run_edge,
    )

    return _write(path, nodes)


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def curate(root: Path = _ROOT) -> None:
    nodes_dir = root / "flow_graph" / "nodes"
    results = {
        "protocolgame": _update_protocolgame(nodes_dir),
        "protocolstatus": _update_protocolstatus(nodes_dir),
        "connection": _update_connection(nodes_dir),
        "server": _update_server(nodes_dir),
        "otserv": _update_otserv(nodes_dir),
    }
    written = sum(1 for v in results.values() if v)
    opcode_count = len(_PARSE_PACKET_OPCODES)
    print(
        f"Network curation: {opcode_count} opcode edges, "
        f"{len(_DISABLED_OPCODES)} disabled, {len(_NOOP_OPCODES)} noop — "
        f"{written} shard(s) written"
    )


if __name__ == "__main__":
    curate()
