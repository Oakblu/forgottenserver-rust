#!/usr/bin/env python3
"""
Symbol manifest extractor for the forgottenserver-rust entity crate.
Extracts all Rust symbols (structs, enums, impls, fns, consts, traits, etc.)
with line ranges, signatures, visibility, dependencies, body hashes and
heuristic risk levels.
"""
import os
import re
import json
import hashlib
from pathlib import Path

SRC_DIR = Path("/Users/pablohpsilva/Documents/monorepo/apps/poketibia/forgottenserver-rust/crates/entity/src")
OUT_PATH = Path("/Users/pablohpsilva/Documents/monorepo/apps/poketibia/forgottenserver-rust/manifest_shards/entity.json")

def body_hash(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:16]


def find_matching_brace(src: str, open_idx: int) -> int:
    depth = 0
    i = open_idx
    in_string = False
    in_line_comment = False
    in_block_comment = False
    string_quote = None
    n = len(src)
    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        if in_line_comment:
            if c == "\n":
                in_line_comment = False
            i += 1
            continue
        if in_block_comment:
            if c == "*" and nxt == "/":
                in_block_comment = False
                i += 2
                continue
            i += 1
            continue
        if in_string:
            if c == "\\":
                i += 2
                continue
            if c == string_quote:
                in_string = False
                string_quote = None
            i += 1
            continue
        if c == "/" and nxt == "/":
            in_line_comment = True
            i += 2
            continue
        if c == "/" and nxt == "*":
            in_block_comment = True
            i += 2
            continue
        if c == '"':
            in_string = True
            string_quote = '"'
            i += 1
            continue
        if c == "'":
            # heuristic: lifetime starts with apostrophe + alpha/_
            if nxt.isalpha() or nxt == "_":
                i += 1
                continue
            # char literal
            i += 1
            # skip until matching '
            while i < n and src[i] != "'":
                if src[i] == "\\":
                    i += 1
                i += 1
            if i < n:
                i += 1
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def compute_line_starts(src: str):
    starts = [0]
    for i, c in enumerate(src):
        if c == "\n":
            starts.append(i + 1)
    return starts


def line_of_offset(line_starts, offset):
    lo, hi = 0, len(line_starts) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        if line_starts[mid] <= offset:
            lo = mid + 1
        else:
            hi = mid - 1
    return hi + 1


def extract_deps(text: str):
    cleaned = re.sub(r'//.*', '', text)
    cleaned = re.sub(r'/\*.*?\*/', '', cleaned, flags=re.DOTALL)
    cleaned = re.sub(r'"(?:\\.|[^"\\])*"', '', cleaned)
    ids = re.findall(r'\b[A-Z][A-Za-z0-9_]*\b', cleaned)
    skip = {"Self", "Some", "None", "Option", "Result", "Ok", "Err", "Vec",
            "String", "Box", "Default", "Debug", "Clone", "Copy",
            "PartialEq", "Eq", "Hash", "Display", "Ord", "PartialOrd",
            "Send", "Sync", "Sized", "Drop", "From", "Into", "TryFrom",
            "TryInto", "Iterator", "IntoIterator", "AsRef", "AsMut", "Borrow"}
    deps = sorted(set(i for i in ids if i not in skip))
    return deps[:30]


PLAYER_CRIT = [r"\bsave", r"\bload", r"\bdb\b", r"\bsql\b",
               r"add_health", r"change_health", r"set_health", r"set_mana",
               r"die\b", r"on_death", r"on_kill", r"on_attack",
               r"add_condition", r"remove_condition", r"experience",
               r"set_skill", r"add_skill", r"\bdrop", r"corpse",
               r"set_capacity", r"set_vocation", r"add_item", r"remove_item"]

COMBAT = [r"\battack", r"\bdamage", r"\bblock", r"\bcombat", r"\bhit",
          r"deal_damage", r"absorb", r"resist"]

HIGH = [r"spawn", r"dialog", r"join", r"leave", r"member", r"invite",
        r"keyword", r"queue", r"npc_response", r"response"]


def estimate_risk(name, kind, file, body):
    text = (name + " " + body[:1000]).lower()
    if "player.rs" in file:
        if any(re.search(p, text) for p in PLAYER_CRIT):
            return "critical"
    if "creature.rs" in file or "monster.rs" in file:
        if any(re.search(p, text) for p in COMBAT):
            return "critical"
    if file.endswith(("spawn.rs", "npc.rs", "party.rs", "guild.rs",
                      "monsters.rs", "monster.rs")):
        if any(re.search(p, text) for p in HIGH):
            return "high"
    if kind in ("field", "enum_variant", "constant", "static"):
        return "low"
    if kind in ("struct", "enum", "trait"):
        return "medium"
    if "test_" in name.lower():
        return "low"
    return "medium"


# Try to match Lua binding pattern
LUA_PATTERN = re.compile(r'register(?:Method|GlobalMethod|Class|Constant|UserdataMethod)')


def detect_kind_override(name, body):
    """Detect special kinds (lua_binding, scheduler_event, database_mapping)."""
    notes = []
    if LUA_PATTERN.search(body) or "lua_state" in body.lower() or "luaapi" in body.lower():
        notes.append("possibly-lua-binding")
    if "schedule_event" in body.lower() or "addtask" in body.lower() or "scheduler::" in body.lower():
        notes.append("possible-scheduler-event")
    if "sqlx::" in body.lower() or "query!" in body or "row.get(" in body.lower():
        notes.append("possible-database-mapping")
    return notes


def skip_ws_and_comments_attrs(src, pos, end):
    while pos < end:
        c = src[pos]
        if c.isspace():
            pos += 1
            continue
        if src[pos:pos+2] == "//":
            nl = src.find("\n", pos)
            if nl == -1 or nl >= end:
                return end
            pos = nl + 1
            continue
        if src[pos:pos+2] == "/*":
            eb = src.find("*/", pos+2)
            if eb == -1 or eb >= end:
                return end
            pos = eb + 2
            continue
        if c == "#":
            # attribute or inner attribute
            if pos+1 < end and src[pos+1] in "[!":
                if src[pos+1] == "!" and pos+2 < end and src[pos+2] == "[":
                    open_b = pos + 2
                elif src[pos+1] == "[":
                    open_b = pos + 1
                else:
                    return pos
                depth = 0
                j = open_b
                while j < end:
                    if src[j] == "[":
                        depth += 1
                    elif src[j] == "]":
                        depth -= 1
                        if depth == 0:
                            pos = j + 1
                            break
                    j += 1
                else:
                    return end
                continue
        return pos
    return end


def find_signature_end(src, start, region_end):
    """Find end of fn signature - either ';' or '{'."""
    i = start
    depth = 0
    in_string = False
    in_line_comment = False
    in_block_comment = False
    while i < region_end:
        c = src[i]
        nxt = src[i+1] if i+1 < region_end else ""
        if in_line_comment:
            if c == "\n":
                in_line_comment = False
            i += 1
            continue
        if in_block_comment:
            if c == "*" and nxt == "/":
                in_block_comment = False
                i += 2
                continue
            i += 1
            continue
        if in_string:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_string = False
            i += 1
            continue
        if c == "/" and nxt == "/":
            in_line_comment = True
            i += 2
            continue
        if c == "/" and nxt == "*":
            in_block_comment = True
            i += 2
            continue
        if c == '"':
            in_string = True
            i += 1
            continue
        if c in "<([":
            depth += 1
        elif c in ">)]":
            depth -= 1
        elif depth == 0 and (c == "{" or c == ";"):
            return i
        i += 1
    return -1


def parse_impl_target(header):
    h = header.strip()
    h = re.sub(r'^impl\s*', '', h)
    h = re.sub(r'^<[^>]*>\s*', '', h)
    if ' for ' in h:
        parts = h.split(' for ', 1)
        target = parts[1].strip()
    else:
        target = h.strip()
    target = re.sub(r'<.*', '', target).strip()
    target = re.sub(r'\s+where\s+.*', '', target).strip()
    return target


def parse_impl_trait(header):
    h = header.strip()
    h = re.sub(r'^impl\s*', '', h)
    h = re.sub(r'^<[^>]*>\s*', '', h)
    if ' for ' in h:
        parts = h.split(' for ', 1)
        trait_name = re.sub(r'<.*', '', parts[0]).strip()
        return trait_name
    return None


def parse_attribute_context(src, pos):
    """Look backwards from pos for any nearby #[cfg(test)] attr."""
    start = max(0, pos - 200)
    snippet = src[start:pos]
    return "#[cfg(test)]" in snippet or "#[test]" in snippet


def extract_struct_fields(body, body_start_offset, line_starts, qual_struct, file_name, symbols):
    i = 0
    n = len(body)
    while i < n:
        # skip ws/comments/attrs/commas
        while i < n:
            c = body[i]
            if c.isspace() or c == ",":
                i += 1
                continue
            if body[i:i+2] == "//":
                nl = body.find("\n", i)
                if nl == -1:
                    return
                i = nl + 1
                continue
            if body[i:i+2] == "/*":
                eb = body.find("*/", i+2)
                if eb == -1:
                    return
                i = eb + 2
                continue
            if c == "#":
                if i+1 < n and body[i+1] in "[!":
                    if body[i+1] == "!" and i+2 < n and body[i+2] == "[":
                        open_b = i + 2
                    elif body[i+1] == "[":
                        open_b = i + 1
                    else:
                        break
                    depth = 0
                    j = open_b
                    while j < n:
                        if body[j] == "[":
                            depth += 1
                        elif body[j] == "]":
                            depth -= 1
                            if depth == 0:
                                i = j + 1
                                break
                        j += 1
                    else:
                        return
                    continue
            break
        if i >= n:
            break
        m = re.match(r'(pub(?:\([^)]+\))?\s+)?(\w+)\s*:\s*', body[i:])
        if not m:
            comma = body.find(",", i)
            if comma == -1:
                break
            i = comma + 1
            continue
        vis = m.group(1).strip() if m.group(1) else "private"
        name = m.group(2)
        field_start = i
        depth = 0
        j = i + m.end()
        while j < n:
            c = body[j]
            if c in "<([":
                depth += 1
            elif c in ">)]":
                depth -= 1
            elif c == "," and depth == 0:
                break
            j += 1
        end = j
        sig = body[field_start:end].strip()
        ls = line_of_offset(line_starts, body_start_offset + field_start)
        le = line_of_offset(line_starts, body_start_offset + end)
        symbols.append({
            "file": "crates/entity/src/" + file_name,
            "kind": "field",
            "qualified_name": f"{qual_struct}::{name}",
            "signature": sig,
            "visibility": vis,
            "line_start": ls,
            "line_end": le,
            "dependencies": extract_deps(sig),
            "behavior_summary": f"Field `{name}` of `{qual_struct.split('::')[-1]}`.",
            "risk_level": "low",
            "body_hash": body_hash(sig),
            "notes": [],
        })
        i = end + 1


def extract_enum_variants(body, body_start_offset, line_starts, qual_enum, file_name, symbols):
    i = 0
    n = len(body)
    while i < n:
        while i < n:
            c = body[i]
            if c.isspace() or c == ",":
                i += 1
                continue
            if body[i:i+2] == "//":
                nl = body.find("\n", i)
                if nl == -1:
                    return
                i = nl + 1
                continue
            if body[i:i+2] == "/*":
                eb = body.find("*/", i+2)
                if eb == -1:
                    return
                i = eb + 2
                continue
            if c == "#":
                if i+1 < n and body[i+1] in "[!":
                    if body[i+1] == "!" and i+2 < n and body[i+2] == "[":
                        open_b = i + 2
                    elif body[i+1] == "[":
                        open_b = i + 1
                    else:
                        break
                    depth = 0
                    j = open_b
                    while j < n:
                        if body[j] == "[":
                            depth += 1
                        elif body[j] == "]":
                            depth -= 1
                            if depth == 0:
                                i = j + 1
                                break
                        j += 1
                    else:
                        return
                    continue
            break
        if i >= n:
            break
        m = re.match(r'(\w+)', body[i:])
        if not m:
            i += 1
            continue
        name = m.group(1)
        start = i
        depth = 0
        j = i + m.end()
        # Skip past tuple/struct variant data and discriminant
        while j < n:
            c = body[j]
            if c in "<([{":
                depth += 1
            elif c in ">)]}":
                depth -= 1
            elif c == "," and depth == 0:
                break
            j += 1
        end = j
        sig = body[start:end].strip()
        ls = line_of_offset(line_starts, body_start_offset + start)
        le = line_of_offset(line_starts, body_start_offset + end)
        symbols.append({
            "file": "crates/entity/src/" + file_name,
            "kind": "enum_variant",
            "qualified_name": f"{qual_enum}::{name}",
            "signature": sig,
            "visibility": "pub",
            "line_start": ls,
            "line_end": le,
            "dependencies": extract_deps(sig),
            "behavior_summary": f"Variant `{name}` of `{qual_enum.split('::')[-1]}`.",
            "risk_level": "low",
            "body_hash": body_hash(sig),
            "notes": [],
        })
        i = end + 1


def extract_methods_in_block(src, brace_idx, close_idx, qual_target, is_trait_impl, file_name, symbols, line_starts):
    pos = brace_idx + 1
    end = close_idx
    while pos < end:
        pos = skip_ws_and_comments_attrs(src, pos, end)
        if pos >= end:
            break
        line_end = src.find("\n", pos)
        if line_end == -1 or line_end > end:
            line_end = end
        line = src[pos:line_end]
        # fn
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?(?:extern\s+(?:"[^"]+"\s+)?)?fn\s+(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            sig_end = find_signature_end(src, pos, end)
            if sig_end == -1:
                pos = line_end + 1
                continue
            if src[sig_end] == ";":
                ls = line_of_offset(line_starts, pos)
                le = line_of_offset(line_starts, sig_end)
                sig = src[pos:sig_end+1].strip()
                body = sig
                params_text = sig.split("(", 1)[1].rsplit(")", 1)[0] if "(" in sig else ""
                has_self = re.search(r'\bself\b', params_text) is not None
                if is_trait_impl:
                    kind = "trait_method"
                elif has_self:
                    kind = "impl_method"
                else:
                    kind = "associated_function"
                extra_notes = ["declaration-only"]
                extra_notes.extend(detect_kind_override(name, body))
                symbols.append({
                    "file": "crates/entity/src/" + file_name,
                    "kind": kind,
                    "qualified_name": f"{qual_target}::{name}",
                    "signature": sig,
                    "visibility": vis,
                    "line_start": ls,
                    "line_end": le,
                    "dependencies": extract_deps(sig),
                    "behavior_summary": f"{kind.replace('_',' ').title()} `{name}`.",
                    "risk_level": estimate_risk(name, kind, file_name, body),
                    "body_hash": body_hash(body),
                    "notes": extra_notes,
                })
                pos = sig_end + 1
                continue
            close_brace = find_matching_brace(src, sig_end)
            if close_brace == -1:
                pos = line_end + 1
                continue
            ls = line_of_offset(line_starts, pos)
            le = line_of_offset(line_starts, close_brace)
            sig = src[pos:sig_end].strip()
            body = src[sig_end+1:close_brace]
            params_text = sig.split("(", 1)[1].rsplit(")", 1)[0] if "(" in sig else ""
            has_self = re.search(r'\bself\b', params_text) is not None
            if is_trait_impl:
                kind = "trait_method"
            elif has_self:
                kind = "impl_method"
            else:
                kind = "associated_function"
            extra_notes = []
            extra_notes.extend(detect_kind_override(name, body))
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": kind,
                "qualified_name": f"{qual_target}::{name}",
                "signature": sig + " { ... }",
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(body[:2000]),
                "behavior_summary": f"{kind.replace('_',' ').title()} `{name}`.",
                "risk_level": estimate_risk(name, kind, file_name, body),
                "body_hash": body_hash(body),
                "notes": extra_notes,
            })
            pos = close_brace + 1
            continue
        # const
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?const\s+(\w+)\s*:', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            semi = src.find(";", pos)
            if semi == -1 or semi > end:
                pos = line_end + 1
                continue
            ls = line_of_offset(line_starts, pos)
            le = line_of_offset(line_starts, semi)
            sig = src[pos:semi+1].strip()
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "constant",
                "qualified_name": f"{qual_target}::{name}",
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(sig),
                "behavior_summary": f"Associated constant `{name}`.",
                "risk_level": "low",
                "body_hash": body_hash(sig),
                "notes": [],
            })
            pos = semi + 1
            continue
        # type
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?type\s+(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            semi = src.find(";", pos)
            ls = line_of_offset(line_starts, pos)
            le = line_of_offset(line_starts, semi)
            sig = src[pos:semi+1].strip()
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "type_alias",
                "qualified_name": f"{qual_target}::{name}",
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(sig),
                "behavior_summary": f"Associated type `{name}`.",
                "risk_level": "low",
                "body_hash": body_hash(sig),
                "notes": [],
            })
            pos = semi + 1
            continue
        pos = line_end + 1


def scan_region(src, region_start, region_end, scope_qualifier, file_name, mod_name, symbols, line_starts, in_test_mod=False):
    i = region_start
    while i < region_end:
        i = skip_ws_and_comments_attrs(src, i, region_end)
        if i >= region_end:
            break
        # capture nearby cfg(test) marker for inner items
        is_test_ctx = in_test_mod or parse_attribute_context(src, i)
        line_end = src.find("\n", i)
        if line_end == -1 or line_end > region_end:
            line_end = region_end
        line = src[i:line_end]

        # struct
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?struct\s+(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            semicolon = src.find(";", i)
            brace = src.find("{", i)
            if brace == -1 or (semicolon != -1 and semicolon < brace):
                end_idx = semicolon if semicolon != -1 else line_end
                ls = line_of_offset(line_starts, i)
                le = line_of_offset(line_starts, end_idx)
                sig = src[i:end_idx+1].strip()
                qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
                symbols.append({
                    "file": "crates/entity/src/" + file_name,
                    "kind": "struct",
                    "qualified_name": qual,
                    "signature": sig,
                    "visibility": vis,
                    "line_start": ls,
                    "line_end": le,
                    "dependencies": extract_deps(sig),
                    "behavior_summary": f"Struct `{name}` (tuple/unit form).",
                    "risk_level": estimate_risk(name, "struct", file_name, sig),
                    "body_hash": body_hash(sig),
                    "notes": ["cfg(test)"] if is_test_ctx else [],
                })
                i = end_idx + 1
                continue
            close = find_matching_brace(src, brace)
            if close == -1:
                i = line_end + 1
                continue
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, close)
            sig = src[i:brace].strip()
            body = src[brace+1:close]
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "struct",
                "qualified_name": qual,
                "signature": sig + " { ... }",
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(body[:3000]),
                "behavior_summary": f"Struct `{name}`.",
                "risk_level": estimate_risk(name, "struct", file_name, body),
                "body_hash": body_hash(body),
                "notes": ["cfg(test)"] if is_test_ctx else [],
            })
            extract_struct_fields(body, brace+1, line_starts, qual, file_name, symbols)
            i = close + 1
            continue

        # enum
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?enum\s+(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            brace = src.find("{", i)
            if brace == -1 or brace > region_end:
                i = line_end + 1
                continue
            close = find_matching_brace(src, brace)
            if close == -1:
                i = line_end + 1
                continue
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, close)
            sig = src[i:brace].strip()
            body = src[brace+1:close]
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "enum",
                "qualified_name": qual,
                "signature": sig + " { ... }",
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(body[:3000]),
                "behavior_summary": f"Enum `{name}`.",
                "risk_level": estimate_risk(name, "enum", file_name, body),
                "body_hash": body_hash(body),
                "notes": ["cfg(test)"] if is_test_ctx else [],
            })
            extract_enum_variants(body, brace+1, line_starts, qual, file_name, symbols)
            i = close + 1
            continue

        # trait
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?(?:unsafe\s+)?trait\s+(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            brace = src.find("{", i)
            if brace == -1 or brace > region_end:
                i = line_end + 1
                continue
            close = find_matching_brace(src, brace)
            if close == -1:
                i = line_end + 1
                continue
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, close)
            sig = src[i:brace].strip()
            body = src[brace+1:close]
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "trait",
                "qualified_name": qual,
                "signature": sig + " { ... }",
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(body[:3000]),
                "behavior_summary": f"Trait `{name}`.",
                "risk_level": "medium",
                "body_hash": body_hash(body),
                "notes": ["cfg(test)"] if is_test_ctx else [],
            })
            extract_methods_in_block(src, brace, close, qual, True, file_name, symbols, line_starts)
            i = close + 1
            continue

        # impl
        m = re.match(r'^impl\b', line)
        if m:
            brace = src.find("{", i)
            if brace == -1 or brace > region_end:
                i = line_end + 1
                continue
            close = find_matching_brace(src, brace)
            if close == -1:
                i = line_end + 1
                continue
            header = src[i:brace].strip()
            target = parse_impl_target(header)
            trait_name = parse_impl_trait(header)
            qual = f"entity::{scope_qualifier}::{target}" if scope_qualifier else f"entity::{mod_name}::{target}"
            extract_methods_in_block(src, brace, close, qual, trait_name is not None, file_name, symbols, line_starts)
            i = close + 1
            continue

        # free fn
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?(?:extern\s+(?:"[^"]+"\s+)?)?fn\s+(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            sig_end = find_signature_end(src, i, region_end)
            if sig_end == -1:
                i = line_end + 1
                continue
            if src[sig_end] == ";":
                ls = line_of_offset(line_starts, i)
                le = line_of_offset(line_starts, sig_end)
                sig = src[i:sig_end+1].strip()
                qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
                symbols.append({
                    "file": "crates/entity/src/" + file_name,
                    "kind": "free_function",
                    "qualified_name": qual,
                    "signature": sig,
                    "visibility": vis,
                    "line_start": ls,
                    "line_end": le,
                    "dependencies": extract_deps(sig),
                    "behavior_summary": f"Function `{name}` declaration.",
                    "risk_level": estimate_risk(name, "free_function", file_name, sig),
                    "body_hash": body_hash(sig),
                    "notes": ["declaration-only"] + (["cfg(test)"] if is_test_ctx else []),
                })
                i = sig_end + 1
                continue
            close_brace = find_matching_brace(src, sig_end)
            if close_brace == -1:
                i = line_end + 1
                continue
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, close_brace)
            sig = src[i:sig_end].strip()
            body = src[sig_end+1:close_brace]
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            notes = ["cfg(test)"] if is_test_ctx else []
            notes.extend(detect_kind_override(name, body))
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "free_function",
                "qualified_name": qual,
                "signature": sig + " { ... }",
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(body[:2000]),
                "behavior_summary": f"Function `{name}`{' (test)' if is_test_ctx else ''}.",
                "risk_level": "low" if is_test_ctx else estimate_risk(name, "free_function", file_name, body),
                "body_hash": body_hash(body),
                "notes": notes,
            })
            i = close_brace + 1
            continue

        # const
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?const\s+(\w+)\s*:', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            semi = src.find(";", i)
            if semi == -1 or semi > region_end:
                i = line_end + 1
                continue
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, semi)
            sig = src[i:semi+1].strip()
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "constant",
                "qualified_name": qual,
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(sig),
                "behavior_summary": f"Constant `{name}`.",
                "risk_level": "low",
                "body_hash": body_hash(sig),
                "notes": ["cfg(test)"] if is_test_ctx else [],
            })
            i = semi + 1
            continue

        # static
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?static\s+(?:mut\s+)?(\w+)\s*:', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            semi = src.find(";", i)
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, semi)
            sig = src[i:semi+1].strip()
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "static",
                "qualified_name": qual,
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(sig),
                "behavior_summary": f"Static `{name}`.",
                "risk_level": "low",
                "body_hash": body_hash(sig),
                "notes": ["cfg(test)"] if is_test_ctx else [],
            })
            i = semi + 1
            continue

        # type alias
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?type\s+(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            semi = src.find(";", i)
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, semi)
            sig = src[i:semi+1].strip()
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "type_alias",
                "qualified_name": qual,
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(sig),
                "behavior_summary": f"Type alias `{name}`.",
                "risk_level": "low",
                "body_hash": body_hash(sig),
                "notes": ["cfg(test)"] if is_test_ctx else [],
            })
            i = semi + 1
            continue

        # macro_rules
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?macro_rules!\s*(\w+)', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            brace = src.find("{", i)
            close = find_matching_brace(src, brace) if brace != -1 else -1
            if close == -1:
                i = line_end + 1
                continue
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, close)
            sig = src[i:brace].strip() + " { ... }"
            body = src[brace+1:close]
            qual = f"entity::{scope_qualifier}::{name}" if scope_qualifier else f"entity::{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "macro",
                "qualified_name": qual,
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": extract_deps(body[:2000]),
                "behavior_summary": f"Macro `{name}`.",
                "risk_level": "medium",
                "body_hash": body_hash(body),
                "notes": ["cfg(test)"] if is_test_ctx else [],
            })
            i = close + 1
            continue

        # inline mod
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?mod\s+(\w+)\s*\{', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            brace = src.find("{", i)
            close = find_matching_brace(src, brace) if brace != -1 else -1
            if close == -1:
                i = line_end + 1
                continue
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, close)
            sig = src[i:brace].strip() + " { ... }"
            body = src[brace+1:close]
            inner_is_test = is_test_ctx or "tests" == name
            qual_path = f"{scope_qualifier}::{name}" if scope_qualifier else f"{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "module",
                "qualified_name": f"entity::{qual_path}",
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": [],
                "behavior_summary": f"Inline module `{name}`{' (test)' if inner_is_test else ''}.",
                "risk_level": "low",
                "body_hash": body_hash(body),
                "notes": ["cfg(test)"] if inner_is_test else [],
            })
            scan_region(src, brace + 1, close, qual_path, file_name, mod_name, symbols, line_starts, inner_is_test)
            i = close + 1
            continue

        # mod decl `mod x;`
        m = re.match(r'^(pub(?:\([^)]+\))?\s+)?mod\s+(\w+)\s*;', line)
        if m:
            vis = m.group(1).strip() if m.group(1) else "private"
            name = m.group(2)
            semi = src.find(";", i)
            ls = line_of_offset(line_starts, i)
            le = line_of_offset(line_starts, semi)
            sig = src[i:semi+1].strip()
            qual_path = f"{scope_qualifier}::{name}" if scope_qualifier else f"{mod_name}::{name}"
            symbols.append({
                "file": "crates/entity/src/" + file_name,
                "kind": "module",
                "qualified_name": f"entity::{qual_path}",
                "signature": sig,
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": [],
                "behavior_summary": f"Module declaration `{name}`.",
                "risk_level": "low",
                "body_hash": body_hash(sig),
                "notes": [],
            })
            i = semi + 1
            continue

        # No match — advance one line
        i = line_end + 1


def extract_symbols_from_file(file_path: Path):
    src = file_path.read_text()
    line_starts = compute_line_starts(src)
    mod_name = file_path.stem
    symbols = []
    if file_path.name == "lib.rs":
        # parse module declarations
        for m in re.finditer(r'^\s*(pub(?:\([^)]+\))?\s+)?mod\s+(\w+)\s*;', src, re.MULTILINE):
            vis = m.group(1).strip() if m.group(1) else "private"
            modn = m.group(2)
            ls = line_of_offset(line_starts, m.start())
            le = line_of_offset(line_starts, m.end())
            symbols.append({
                "file": "crates/entity/src/" + file_path.name,
                "kind": "module",
                "qualified_name": f"entity::{modn}",
                "signature": m.group(0).strip(),
                "visibility": vis,
                "line_start": ls,
                "line_end": le,
                "dependencies": [],
                "behavior_summary": f"Re-exports module `{modn}` from entity crate.",
                "risk_level": "low",
                "body_hash": body_hash(m.group(0)),
                "notes": [],
            })
        return symbols
    scan_region(src, 0, len(src), "", file_path.name, mod_name, symbols, line_starts)
    return symbols


def main():
    OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    all_symbols = []
    files = sorted(SRC_DIR.glob("*.rs"))
    for f in files:
        syms = extract_symbols_from_file(f)
        print(f"{f.name}: {len(syms)} symbols")
        all_symbols.extend(syms)
    print(f"TOTAL: {len(all_symbols)} symbols")
    OUT_PATH.write_text(json.dumps(all_symbols, indent=2))
    print(f"Wrote: {OUT_PATH}")


if __name__ == "__main__":
    main()
