#!/usr/bin/env python3
"""Sync MIGRATION.md's per-file status column to the rollup.

Per design.md §11, when MIGRATION.md and the symbol-level ledger
disagree, the symbols win and the narrative is updated. This script
does that surgically:

  - Only the `status` column (column 5) of each table row is rewritten.
  - All other columns (rust_module, crate, layer, cov%, owner, notes)
    are preserved byte-for-byte.
  - Rows referencing `foo.cpp/h` are matched against BOTH `src/foo.cpp`
    and `src/foo.h` in the rollup; the worst status wins.

Severity (worst → best): MISSING > PENDING > PARTIAL > DONE > NON_GOAL.

Usage:
    python3 scripts/ledger/sync_migration_md.py            # writes in place
    python3 scripts/ledger/sync_migration_md.py --check    # exit 1 if drift
"""
from __future__ import annotations

import argparse
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import ledger_io  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))


SEVERITY = {"MISSING": 5, "PENDING": 4, "PARTIAL": 3, "DONE": 2, "NON_GOAL": 1, "UNCERTAIN": 4}
STATUSES = "|".join(SEVERITY.keys())

# Match a MIGRATION.md row whose first column names a cpp file
# (foo.cpp, foo.h, or foo.cpp/h) and whose 5th column is a status.
ROW_RE = re.compile(
    r"^(?P<head>\|\s*(?P<cpp>[A-Za-z0-9_./\-]+\.(?:cpp/h|cpp|h))\s*"
    r"\|[^|]*\|[^|]*\|[^|]*\|\s*)"
    r"(?P<status>" + STATUSES + r")"
    r"(?P<tail>\s*\|.*)$"
)


def _worst(a: str | None, b: str | None) -> str | None:
    candidates = [s for s in (a, b) if s is not None]
    if not candidates:
        return None
    return max(candidates, key=lambda s: SEVERITY.get(s, 0))


def _status_for(cpp_token: str, rollup: dict[str, str]) -> str | None:
    """Resolve the MIGRATION.md cpp-token to a rollup status.

    `foo.cpp/h` → max severity of foo.cpp + foo.h.
    `foo.cpp` or `foo.h` → lookup direct.
    """
    if cpp_token.endswith(".cpp/h"):
        stem = cpp_token[: -len(".cpp/h")]
        a = rollup.get(f"src/{stem}.cpp")
        b = rollup.get(f"src/{stem}.h")
        return _worst(a, b)
    return rollup.get(f"src/{cpp_token}")


def sync(md_text: str, rollup: dict[str, str]) -> tuple[str, int, int]:
    out_lines: list[str] = []
    changed = 0
    matched = 0
    for raw in md_text.splitlines():
        m = ROW_RE.match(raw)
        if not m:
            out_lines.append(raw)
            continue
        matched += 1
        cpp = m.group("cpp").strip()
        old = m.group("status")
        new = _status_for(cpp, rollup)
        if new is None or new == old:
            out_lines.append(raw)
            continue
        # Preserve original column width by padding to len(old).
        replacement = new
        if len(replacement) < len(old):
            replacement = replacement + " " * (len(old) - len(replacement))
        elif len(replacement) > len(old):
            # widen — drop trailing spaces in head/tail to compensate so
            # the table grid stays aligned. Simplest safe option: rewrite
            # the row without trying to repad the next column. Risk:
            # table column becomes slightly off; readers tolerate it.
            pass
        new_line = m.group("head") + replacement + m.group("tail")
        out_lines.append(new_line)
        changed += 1
    return "\n".join(out_lines) + ("\n" if md_text.endswith("\n") else ""), matched, changed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Sync MIGRATION.md to rollup")
    parser.add_argument("--md", default=os.path.join(ROOT, "MIGRATION.md"))
    parser.add_argument("--ledger", default=os.path.join(ROOT, "MIGRATION_LEDGER.yml"))
    parser.add_argument("--check", action="store_true", help="exit 1 if updates would happen; don't write")
    args = parser.parse_args(argv)

    doc = ledger_io.load(open(args.ledger, "r", encoding="utf-8").read())
    rollup = {f["cpp"]: f["status"] for f in doc.get("files") or []}

    with open(args.md, "r", encoding="utf-8") as f:
        original = f.read()
    updated, matched, changed = sync(original, rollup)

    print(f"Matched rows: {matched}")
    print(f"Status updates: {changed}")
    if args.check:
        if changed:
            print(f"MIGRATION.md is out of sync with the rollup. Run without --check to fix.")
            return 1
        print("MIGRATION.md is in sync.")
        return 0
    if changed:
        with open(args.md, "w", encoding="utf-8") as f:
            f.write(updated)
        print(f"Wrote {args.md}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
