"""Canonical YAML I/O for MIGRATION_LEDGER.yml.

The ledger uses a deliberately restricted subset of YAML so we can read
and write it without a `pyyaml` dependency (the host Python is PEP-668
externally-managed). Format rules:

- Block style only. No flow mappings except the literal `[]` for empty
  lists. No anchors, no tags, no multi-document streams.
- Scalars are emitted in one of three forms:
    * bareword:  ^[A-Za-z_][A-Za-z0-9_]*$ (used for enums + ids when
      they match this regex)
    * integer:   ^-?[0-9]+$
    * JSON string: anything else, written as `"..."` via `json.dumps`.
- Lists of scalars or mappings are emitted in block style with two-space
  indent steps.
- Empty lists are `[]` (so a reader can tell "audited, none" from
  "not yet present").
- Comments are allowed only before `version:` (file header). The
  in-repo reader strips them but does not preserve them on round-trip.

This is enough to represent every field in `design.md` §2 and stays
diffable.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass
from typing import Any


_BAREWORD = re.compile(r"^[A-Za-z_][A-Za-z0-9_\-]*$")
_INT = re.compile(r"^-?[0-9]+$")
_HEX = re.compile(r"^[0-9a-fA-F]+$")


# ---------------------------------------------------------------------------
# Emitter
# ---------------------------------------------------------------------------

def _emit_scalar(value: Any) -> str:
    """Emit a single scalar in canonical form."""
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int) and not isinstance(value, bool):
        return str(value)
    if isinstance(value, str):
        if value == "":
            return '""'
        if _BAREWORD.match(value) and value not in {"null", "true", "false", "yes", "no"}:
            return value
        # Numeric-looking strings (e.g. body_hash "2369646527424954") must
        # be JSON-quoted; emitting them bare would make the parser round-
        # trip them as `int`, silently losing the string identity.
        return json.dumps(value, ensure_ascii=False)
    raise TypeError(f"unsupported scalar type: {type(value).__name__}")


def _emit(value: Any, indent: int, lines: list[str], at_list_item: bool = False) -> None:
    pad = " " * indent
    if isinstance(value, list):
        if not value:
            # caller appended the key already; replace last line's trailing space with []
            if lines and lines[-1].endswith(" "):
                lines[-1] = lines[-1].rstrip() + " []"
            else:
                lines.append(pad + "[]")
            return
        for item in value:
            if isinstance(item, (dict, list)):
                if isinstance(item, dict):
                    # Emit "- " then the first key on the same line.
                    keys = list(item.keys())
                    if not keys:
                        lines.append(pad + "- {}")
                        continue
                    first = keys[0]
                    first_value = item[first]
                    if isinstance(first_value, list) and not first_value:
                        lines.append(pad + "- " + first + ": []")
                    elif isinstance(first_value, (dict, list)):
                        lines.append(pad + "- " + first + ":")
                        _emit(first_value, indent + 4, lines)
                    else:
                        lines.append(pad + "- " + first + ": " + _emit_scalar(first_value))
                    for k in keys[1:]:
                        v = item[k]
                        if isinstance(v, list) and not v:
                            lines.append(" " * (indent + 2) + k + ": []")
                        elif isinstance(v, (dict, list)):
                            lines.append(" " * (indent + 2) + k + ":")
                            _emit(v, indent + 4, lines)
                        else:
                            lines.append(" " * (indent + 2) + k + ": " + _emit_scalar(v))
                else:
                    lines.append(pad + "-")
                    _emit(item, indent + 2, lines)
            else:
                lines.append(pad + "- " + _emit_scalar(item))
        return
    if isinstance(value, dict):
        for k, v in value.items():
            if isinstance(v, (dict, list)):
                if isinstance(v, list) and not v:
                    lines.append(pad + k + ": []")
                else:
                    lines.append(pad + k + ":")
                    _emit(v, indent + 2, lines)
            else:
                lines.append(pad + k + ": " + _emit_scalar(v))
        return
    # bare scalar at top level (rare; supported for completeness)
    lines.append(pad + _emit_scalar(value))


def dump(doc: dict, header: str = "") -> str:
    """Serialize a top-level mapping to canonical YAML text."""
    lines: list[str] = []
    if header:
        for line in header.splitlines():
            lines.append("# " + line if line and not line.startswith("#") else line)
        lines.append("")
    _emit(doc, 0, lines)
    return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# Reader (strict canonical only)
# ---------------------------------------------------------------------------

@dataclass
class _Line:
    indent: int
    text: str          # without leading whitespace
    raw: str           # original
    n: int             # 1-based line number


class LedgerParseError(ValueError):
    pass


def _tokenize(text: str) -> list[_Line]:
    out: list[_Line] = []
    for i, raw in enumerate(text.splitlines(), start=1):
        stripped = raw.lstrip(" ")
        if not stripped or stripped.startswith("#"):
            continue
        indent = len(raw) - len(stripped)
        if indent % 2 != 0:
            raise LedgerParseError(f"line {i}: indent must be even, got {indent}")
        out.append(_Line(indent=indent, text=stripped, raw=raw, n=i))
    return out


def _parse_scalar(s: str, n: int) -> Any:
    if s == "" or s == '""':
        return ""
    if s == "null":
        return None
    if s == "true":
        return True
    if s == "false":
        return False
    if s == "[]":
        return []
    if s.startswith('"'):
        # JSON string
        try:
            return json.loads(s)
        except json.JSONDecodeError as e:
            raise LedgerParseError(f"line {n}: bad JSON string: {e}")
    if _INT.match(s):
        return int(s)
    if _BAREWORD.match(s) or _HEX.match(s):
        return s
    raise LedgerParseError(f"line {n}: unrecognized scalar {s!r}")


def _parse_block(lines: list[_Line], pos: int, indent: int):
    """Parse a block (mapping or sequence) starting at lines[pos],
    return (value, next_pos). Stops when a line is found at indent <
    `indent` or pos == len(lines)."""
    if pos >= len(lines):
        return None, pos
    first = lines[pos]
    if first.indent < indent:
        return None, pos
    if first.text.startswith("- "):
        return _parse_sequence(lines, pos, indent)
    return _parse_mapping(lines, pos, indent)


def _parse_mapping(lines: list[_Line], pos: int, indent: int):
    out: dict = {}
    while pos < len(lines):
        ln = lines[pos]
        if ln.indent < indent:
            break
        if ln.indent != indent:
            raise LedgerParseError(
                f"line {ln.n}: expected indent {indent}, got {ln.indent}"
            )
        if ":" not in ln.text:
            raise LedgerParseError(f"line {ln.n}: expected mapping key, got {ln.text!r}")
        key, _, rest = ln.text.partition(":")
        key = key.strip()
        rest = rest.strip()
        pos += 1
        if rest == "":
            # Nested block
            value, pos = _parse_block(lines, pos, indent + 2)
            out[key] = value if value is not None else {}
        else:
            out[key] = _parse_scalar(rest, ln.n)
    return out, pos


def _parse_sequence(lines: list[_Line], pos: int, indent: int):
    out: list = []
    while pos < len(lines):
        ln = lines[pos]
        if ln.indent < indent:
            break
        if ln.indent != indent:
            raise LedgerParseError(
                f"line {ln.n}: expected indent {indent}, got {ln.indent}"
            )
        if not ln.text.startswith("- "):
            break
        inner = ln.text[2:]
        pos += 1
        if inner == "":
            # Plain "-" with nested block beneath
            value, pos = _parse_block(lines, pos, indent + 2)
            out.append(value if value is not None else {})
            continue
        if ":" in inner and not inner.startswith('"'):
            # Mapping item: parse first key inline, then continue at indent+2.
            key, _, rest = inner.partition(":")
            key = key.strip()
            rest = rest.strip()
            item: dict = {}
            if rest == "":
                value, pos = _parse_block(lines, pos, indent + 4)
                item[key] = value if value is not None else {}
            else:
                item[key] = _parse_scalar(rest, ln.n)
            # Continue with subsequent keys at indent + 2.
            sub_indent = indent + 2
            while pos < len(lines):
                nxt = lines[pos]
                if nxt.indent < sub_indent or nxt.text.startswith("- "):
                    break
                if nxt.indent != sub_indent:
                    raise LedgerParseError(
                        f"line {nxt.n}: expected indent {sub_indent}, got {nxt.indent}"
                    )
                k2, _, r2 = nxt.text.partition(":")
                k2 = k2.strip()
                r2 = r2.strip()
                pos += 1
                if r2 == "":
                    value, pos = _parse_block(lines, pos, sub_indent + 2)
                    item[k2] = value if value is not None else {}
                else:
                    item[k2] = _parse_scalar(r2, nxt.n)
            out.append(item)
        else:
            out.append(_parse_scalar(inner, ln.n))
    return out, pos


def load(text: str) -> dict:
    """Parse canonical-YAML text into a Python dict."""
    lines = _tokenize(text)
    if not lines:
        return {}
    doc, _ = _parse_block(lines, 0, 0)
    if not isinstance(doc, dict):
        raise LedgerParseError("top-level document must be a mapping")
    return doc
