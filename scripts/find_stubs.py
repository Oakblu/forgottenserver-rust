#!/usr/bin/env python3
"""Find stub functions in the Rust port codebase for migration ledger tracking.

Usage:
    python3 scripts/find_stubs.py
    # writes scripts/stub_report.json
"""

import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
CRATES_DIR = ROOT / "crates"
MANIFEST_PATH = ROOT / "rust_symbol_manifest.json"
OUTPUT_PATH = Path(__file__).parent / "stub_report.json"


def strip_test_blocks(src: str) -> str:
    """Replace #[cfg(test)] mod/impl { ... } blocks with spaces (newlines preserved)."""
    marker = "#[cfg(test)]"
    chars = list(src)
    i = 0
    while i < len(src):
        if src[i : i + len(marker)] == marker:
            j = i + len(marker)
            # Skip whitespace and non-brace tokens (e.g. "mod tests", "impl Foo")
            # until we find the opening '{' of the associated item
            while j < len(src) and src[j] != "{":
                # If we hit another '#' or a newline after the first non-whitespace
                # word, there's no directly attached block — bail out.
                if src[j] == "#":
                    break
                j += 1
            if j < len(src) and src[j] == "{":
                depth, k = 1, j + 1
                while k < len(src) and depth:
                    if src[k] == "{":
                        depth += 1
                    elif src[k] == "}":
                        depth -= 1
                    k += 1
                # Blank out [i, k) but keep newlines so line numbers stay valid
                for m in range(i, k):
                    if chars[m] != "\n":
                        chars[m] = " "
                i = k
                continue
        i += 1
    return "".join(chars)


def main() -> None:
    manifest = load_manifest(MANIFEST_PATH)
    stubs = []
    for rs_path in walk_rs_files(CRATES_DIR):
        stubs.extend(scan_file(rs_path, CRATES_DIR, manifest))
    OUTPUT_PATH.write_text(json.dumps(stubs, indent=2))
    print(f"Wrote {len(stubs)} stubs to {OUTPUT_PATH}", file=sys.stderr)


if __name__ == "__main__":
    main()
