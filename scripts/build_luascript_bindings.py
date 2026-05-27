#!/usr/bin/env python3
"""Build the luascript bindings manifest from luascript.cpp."""

import json
import re

SRC = "/Users/pablohpsilva/Documents/monorepo/apps/poketibia/forgottenserver/src/luascript.cpp"
OUT = "/Users/pablohpsilva/Documents/monorepo/apps/poketibia/forgottenserver-rust/manifest_parts/part_01b_luascript_bindings.json"

with open(SRC, "r", encoding="utf-8") as f:
    lines = f.readlines()

n = len(lines)
entries = []

# --- 1. Static function definitions ---
static_fn_starts = []
for i, line in enumerate(lines, start=1):
    if re.match(r"^int LuaScriptInterface::lua[A-Z]", line):
        m = re.match(r"^int LuaScriptInterface::(lua[A-Za-z0-9_]+)\(lua_State\* L\)", line)
        if m:
            static_fn_starts.append((i, m.group(1)))


def find_block_end(start_idx):
    i = start_idx - 1
    brace_count = 0
    started = False
    while i < n:
        line = lines[i]
        for ch in line:
            if ch == '{':
                brace_count += 1
                started = True
            elif ch == '}':
                brace_count -= 1
                if started and brace_count == 0:
                    return i + 1
        i += 1
    return start_idx


for line_start, fn_name in static_fn_starts:
    line_end = find_block_end(line_start)
    entries.append({
        "file": "src/luascript.cpp",
        "kind": "static_function",
        "qualified_name": f"LuaScriptInterface::{fn_name}",
        "signature": f"static int LuaScriptInterface::{fn_name}(lua_State* L)",
        "visibility": "public",
        "line_start": line_start,
        "line_end": line_end,
        "dependencies": [],
        "behavior_summary": f"Lua binding implementation for {fn_name}.",
        "risk_level": "high",
        "body_hash": "",
        "notes": []
    })

# --- 2. Lua_register ---
lua_register_re = re.compile(r'^\s*lua_register\(L,\s*"([^"]+)",\s*LuaScriptInterface::(lua[A-Za-z0-9_]+)\)')
for i, line in enumerate(lines, start=1):
    m = lua_register_re.match(line)
    if m:
        name = m.group(1)
        func = m.group(2)
        entries.append({
            "file": "src/luascript.cpp",
            "kind": "lua_binding",
            "qualified_name": f"lua:{name}",
            "signature": f'lua_register(L, "{name}", LuaScriptInterface::{func})',
            "visibility": "global",
            "line_start": i,
            "line_end": i,
            "dependencies": [f"LuaScriptInterface::{func}"],
            "behavior_summary": f"Registers global Lua function '{name}'.",
            "risk_level": "high",
            "body_hash": "",
            "notes": []
        })

# --- 3. registerClass ---
reg_class_re = re.compile(r'^\s*registerClass\(L,\s*"([^"]+)",\s*"([^"]*)"(?:,\s*(?:LuaScriptInterface::)?(\w+))?\)')
for i, line in enumerate(lines, start=1):
    if line.lstrip().startswith("//"):
        continue
    m = reg_class_re.match(line)
    if m:
        cls = m.group(1)
        base = m.group(2)
        ctor = m.group(3) or ""
        deps = [f"LuaScriptInterface::{ctor}"] if ctor and ctor.startswith("lua") else []
        sig = f'registerClass(L, "{cls}", "{base}"'
        if ctor:
            sig += f", LuaScriptInterface::{ctor}"
        sig += ")"
        entries.append({
            "file": "src/luascript.cpp",
            "kind": "lua_binding",
            "qualified_name": f"lua:{cls}",
            "signature": sig,
            "visibility": "global",
            "line_start": i,
            "line_end": i,
            "dependencies": deps,
            "behavior_summary": f"Registers Lua class '{cls}'" + (f" extending '{base}'." if base else "."),
            "risk_level": "high",
            "body_hash": "",
            "notes": []
        })

# --- 4. registerTable ---
reg_table_re = re.compile(r'^\s*registerTable\(L,\s*"([^"]+)"\)')
for i, line in enumerate(lines, start=1):
    if line.lstrip().startswith("//"):
        continue
    m = reg_table_re.match(line)
    if m:
        t = m.group(1)
        entries.append({
            "file": "src/luascript.cpp",
            "kind": "lua_binding",
            "qualified_name": f"lua:{t}",
            "signature": f'registerTable(L, "{t}")',
            "visibility": "global",
            "line_start": i,
            "line_end": i,
            "dependencies": [],
            "behavior_summary": f"Registers Lua table namespace '{t}'.",
            "risk_level": "high",
            "body_hash": "",
            "notes": []
        })

# --- multi-line offset lookup ---
text = "".join(lines)
offsets = [0]
for L in lines:
    offsets.append(offsets[-1] + len(L))


def offset_to_line(off):
    lo, hi = 0, len(offsets) - 1
    while lo < hi:
        mid = (lo + hi) // 2
        if offsets[mid] <= off < offsets[mid + 1]:
            return mid + 1
        elif off < offsets[mid]:
            hi = mid
        else:
            lo = mid + 1
    return lo + 1


# --- 5. registerMethod ---
multi_re = re.compile(r'registerMethod\(L,\s*"([^"]+)",\s*"([^"]+)",\s*(?:LuaScriptInterface::)?(\w+)\)', re.MULTILINE)
seen_method = set()
for m in multi_re.finditer(text):
    cls = m.group(1)
    method = m.group(2)
    func = m.group(3)
    line_no = offset_to_line(m.start())
    line_content = lines[line_no - 1].lstrip()
    if line_content.startswith("//"):
        continue
    key = (cls, method)
    if key in seen_method:
        continue
    seen_method.add(key)
    deps = [f"LuaScriptInterface::{func}"] if func.startswith("lua") else []
    entries.append({
        "file": "src/luascript.cpp",
        "kind": "lua_binding",
        "qualified_name": f"lua:{cls}.{method}",
        "signature": f'registerMethod(L, "{cls}", "{method}", LuaScriptInterface::{func})',
        "visibility": "global",
        "line_start": line_no,
        "line_end": line_no,
        "dependencies": deps,
        "behavior_summary": f"Registers Lua method '{cls}:{method}'.",
        "risk_level": "high",
        "body_hash": "",
        "notes": []
    })

# --- 6. registerMetaMethod ---
meta_re = re.compile(r'registerMetaMethod\(L,\s*"([^"]+)",\s*"([^"]+)",\s*(?:LuaScriptInterface::)?(\w+)\)', re.MULTILINE)
seen_meta = set()
for m in meta_re.finditer(text):
    cls = m.group(1)
    method = m.group(2)
    func = m.group(3)
    line_no = offset_to_line(m.start())
    line_content = lines[line_no - 1].lstrip()
    if line_content.startswith("//"):
        continue
    key = (cls, method)
    if key in seen_meta:
        continue
    seen_meta.add(key)
    deps = [f"LuaScriptInterface::{func}"] if func.startswith("lua") else []
    entries.append({
        "file": "src/luascript.cpp",
        "kind": "lua_binding",
        "qualified_name": f"lua:{cls}.{method}",
        "signature": f'registerMetaMethod(L, "{cls}", "{method}", LuaScriptInterface::{func})',
        "visibility": "global",
        "line_start": line_no,
        "line_end": line_no,
        "dependencies": deps,
        "behavior_summary": f"Registers Lua metamethod '{cls}.{method}'.",
        "risk_level": "high",
        "body_hash": "",
        "notes": []
    })

# --- 7. registerGlobalMethod ---
glob_re = re.compile(r'registerGlobalMethod\(L,\s*"([^"]+)",\s*(?:LuaScriptInterface::)?(\w+)\)', re.MULTILINE)
seen_glob = set()
for m in glob_re.finditer(text):
    name = m.group(1)
    func = m.group(2)
    line_no = offset_to_line(m.start())
    line_content = lines[line_no - 1].lstrip()
    if line_content.startswith("//"):
        continue
    if name in seen_glob:
        continue
    seen_glob.add(name)
    deps = [f"LuaScriptInterface::{func}"] if func.startswith("lua") else []
    entries.append({
        "file": "src/luascript.cpp",
        "kind": "lua_binding",
        "qualified_name": f"lua:{name}",
        "signature": f'registerGlobalMethod(L, "{name}", LuaScriptInterface::{func})',
        "visibility": "global",
        "line_start": line_no,
        "line_end": line_no,
        "dependencies": deps,
        "behavior_summary": f"Registers global Lua method '{name}'.",
        "risk_level": "high",
        "body_hash": "",
        "notes": []
    })

# --- 8. registerGlobalVariable ---
gvar_re = re.compile(r'^\s*registerGlobalVariable\(L,\s*"([^"]+)",\s*(\S+?)\);')
for i, line in enumerate(lines, start=1):
    if line.lstrip().startswith("//"):
        continue
    m = gvar_re.match(line)
    if m:
        nm = m.group(1)
        v = m.group(2)
        entries.append({
            "file": "src/luascript.cpp",
            "kind": "lua_binding",
            "qualified_name": f"lua:{nm}",
            "signature": f'registerGlobalVariable(L, "{nm}", {v})',
            "visibility": "global",
            "line_start": i,
            "line_end": i,
            "dependencies": [],
            "behavior_summary": f"Registers global Lua variable '{nm}' = {v}.",
            "risk_level": "high",
            "body_hash": "",
            "notes": []
        })

# --- 9. registerGlobalBoolean ---
gbool_re = re.compile(r'^\s*registerGlobalBoolean\(L,\s*"([^"]+)",\s*(\S+?)\);')
for i, line in enumerate(lines, start=1):
    if line.lstrip().startswith("//"):
        continue
    m = gbool_re.match(line)
    if m:
        nm = m.group(1)
        v = m.group(2)
        entries.append({
            "file": "src/luascript.cpp",
            "kind": "lua_binding",
            "qualified_name": f"lua:{nm}",
            "signature": f'registerGlobalBoolean(L, "{nm}", {v})',
            "visibility": "global",
            "line_start": i,
            "line_end": i,
            "dependencies": [],
            "behavior_summary": f"Registers global Lua boolean '{nm}' = {v}.",
            "risk_level": "high",
            "body_hash": "",
            "notes": []
        })

# --- 10. registerEnum / registerEnumIn ---
enum_re = re.compile(r'^\s*registerEnum\(L,\s*([A-Za-z0-9_]+)\)')
enumin_re = re.compile(r'^\s*registerEnumIn\(L,\s*"([^"]+)",\s*([A-Za-z0-9_]+)\)')

for i, line in enumerate(lines, start=1):
    if line.lstrip().startswith("//"):
        continue
    m = enum_re.match(line)
    if m:
        name = m.group(1)
        entries.append({
            "file": "src/luascript.cpp",
            "kind": "lua_binding",
            "qualified_name": f"lua:{name}",
            "signature": f'registerEnum(L, {name})',
            "visibility": "global",
            "line_start": i,
            "line_end": i,
            "dependencies": [],
            "behavior_summary": f"Registers global Lua enum constant '{name}'.",
            "risk_level": "high",
            "body_hash": "",
            "notes": []
        })
        continue
    m = enumin_re.match(line)
    if m:
        t = m.group(1)
        name = m.group(2)
        entries.append({
            "file": "src/luascript.cpp",
            "kind": "lua_binding",
            "qualified_name": f"lua:{t}.{name}",
            "signature": f'registerEnumIn(L, "{t}", {name})',
            "visibility": "global",
            "line_start": i,
            "line_end": i,
            "dependencies": [],
            "behavior_summary": f"Registers Lua enum constant '{t}.{name}'.",
            "risk_level": "high",
            "body_hash": "",
            "notes": []
        })

with open(OUT, "w", encoding="utf-8") as f:
    json.dump(entries, f, indent=1)

counts = {"static_function": 0, "lua_binding": 0}
for e in entries:
    counts[e["kind"]] = counts.get(e["kind"], 0) + 1

print(f"Total: {len(entries)}")
print(f"static_function: {counts['static_function']}")
print(f"lua_binding: {counts['lua_binding']}")
