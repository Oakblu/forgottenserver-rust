#!/usr/bin/env python3
"""
sync_check.py — tooling for the upstream-sync-check skill.

Must be run from the project root (where MIGRATION_LEDGER.yml lives).

Subcommands:
  ensure-upstream              Clone or fetch forgottenserver-upstream/
  get-commits [--since DATE]   List commits + changed .cpp/.h files (default: 3 months ago)
  map-symbols --files F...     Map changed files to C++ symbols via cpp_symbol_manifest.json
  check-ledger --symbols S...  Query MIGRATION_LEDGER.yml for each symbol (stream-parsed)
  check-intentional --syms S.. Check intentional_differences.yml for each symbol
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

UPSTREAM_DIR = "forgottenserver-upstream"
UPSTREAM_URL = "https://github.com/otland/forgottenserver.git"
CPP_MANIFEST = "cpp_symbol_manifest.json"
LEDGER_FILE = "MIGRATION_LEDGER.yml"
INTENTIONAL_FILE = "intentional_differences.yml"


# ---------------------------------------------------------------------------
# ensure-upstream
# ---------------------------------------------------------------------------

def cmd_ensure_upstream(_args):
    upstream = Path(UPSTREAM_DIR)
    if not (upstream / ".git").exists():
        print(f"Cloning {UPSTREAM_URL} → {UPSTREAM_DIR}/", flush=True)
        result = subprocess.run(
            ["git", "clone", UPSTREAM_URL, UPSTREAM_DIR],
            capture_output=True, text=True,
        )
    else:
        print(f"Fetching origin in {UPSTREAM_DIR}/", flush=True)
        result = subprocess.run(
            ["git", "-C", UPSTREAM_DIR, "fetch", "origin"],
            capture_output=True, text=True,
        )

    if result.returncode != 0:
        print("ERROR:", result.stderr, file=sys.stderr)
        sys.exit(1)

    print("OK — upstream is up to date.")


# ---------------------------------------------------------------------------
# get-commits
# ---------------------------------------------------------------------------

def cmd_get_commits(args):
    since = getattr(args, "since", "3 months ago")
    upstream = Path(UPSTREAM_DIR)

    if not (upstream / ".git").exists():
        print("ERROR: forgottenserver-upstream/ not found. Run ensure-upstream first.", file=sys.stderr)
        sys.exit(1)

    # Try master, fall back to main (only if master branch doesn't exist)
    result = None
    for branch in ("origin/master", "origin/main"):
        r = subprocess.run(
            [
                "git", "-C", UPSTREAM_DIR, "log",
                f"--since={since}",
                "--oneline",
                "--name-only",
                "--diff-filter=ACDMR",
                branch,
            ],
            capture_output=True, text=True,
        )
        if r.returncode == 0:
            result = r
            break  # branch exists — use it even if output is empty
        # branch doesn't exist (returncode != 0); try next

    if result is None or result.returncode != 0:
        print("ERROR:", (result.stderr if result else "no branch found"), file=sys.stderr)
        sys.exit(1)

    raw = result.stdout.strip()
    if not raw:
        print("NO_COMMITS")
        print(f"No upstream commits in the past {since}.")
        return

    # Parse: commit lines vs file lines
    commits = []
    current_commit = None
    current_files = []

    for line in raw.splitlines():
        line = line.strip()
        if not line:
            if current_commit:
                commits.append((current_commit, current_files))
                current_commit = None
                current_files = []
            continue

        commit_match = re.match(r'^([0-9a-f]{7,})\s+(.+)$', line)
        if commit_match:
            if current_commit:
                commits.append((current_commit, current_files))
            current_commit = (commit_match.group(1), commit_match.group(2))
            current_files = []
        else:
            current_files.append(line)

    if current_commit:
        commits.append((current_commit, current_files))

    # Collect unique changed C++ files
    all_files = set()
    print(f"=== {len(commits)} commit(s) since '{since}' ===\n")
    for (sha, msg), files in commits:
        cpp_files = [f for f in files if f.endswith((".cpp", ".h", ".hpp"))]
        print(f"  {sha}  {msg}")
        for f in cpp_files:
            print(f"    {f}")
            all_files.add(f)
        if not cpp_files:
            print("    (no .cpp/.h changes)")

    print(f"\n=== Unique changed C++ files ({len(all_files)}) ===")
    for f in sorted(all_files):
        print(f"  {f}")

    print(f"\nCHANGED_FILES: {json.dumps(sorted(all_files))}")


# ---------------------------------------------------------------------------
# map-symbols
# ---------------------------------------------------------------------------

def cmd_map_symbols(args):
    changed_files = set(args.files)

    manifest_path = Path(CPP_MANIFEST)
    if not manifest_path.exists():
        print(f"ERROR: {CPP_MANIFEST} not found.", file=sys.stderr)
        sys.exit(1)

    data = json.loads(manifest_path.read_text())
    hits = []
    new_files = set()

    for s in data:
        f = s.get("file", "")
        if any(cf in f or f.endswith(cf) for cf in changed_files):
            hits.append((s["kind"], s["qualified_name"], f))

    # Detect changed files not in the manifest at all
    manifest_files = {s.get("file", "") for s in data}
    for cf in changed_files:
        if not any(cf in mf or mf.endswith(cf) for mf in manifest_files):
            new_files.add(cf)

    if hits:
        print(f"=== {len(hits)} symbol(s) found in changed files ===\n")
        for kind, name, file in sorted(hits, key=lambda x: x[2]):
            print(f"  {kind:<12} {name:<60} {file}")
    else:
        print("No symbols found in cpp_symbol_manifest for the changed files.")

    if new_files:
        print(f"\n=== {len(new_files)} file(s) NOT in manifest — manual review needed ===")
        for f in sorted(new_files):
            print(f"  NEW_FILE: {f}")

    # Machine-readable summary for next phase
    symbols = [name for _, name, _ in hits]
    print(f"\nSYMBOLS: {json.dumps(symbols)}")


# ---------------------------------------------------------------------------
# check-ledger
# ---------------------------------------------------------------------------

def cmd_check_ledger(args):
    wanted = set(args.symbols)

    ledger_path = Path(LEDGER_FILE)
    if not ledger_path.exists():
        print(f"ERROR: {LEDGER_FILE} not found.", file=sys.stderr)
        sys.exit(1)

    results = {}
    notes_map = {}
    current_symbol = None
    current_status = None
    current_notes = None

    # Stream-parse — never load the full 148k-line file into memory at once
    with open(ledger_path) as f:
        for line in f:
            m = re.match(r'^\s{2}qualified_name:\s+"?([^"]+)"?', line)
            if m:
                current_symbol = m.group(1).strip()
                current_status = None
                current_notes = None
                continue

            s = re.match(r'^\s{2}status:\s+(\w+)', line)
            if s and current_symbol:
                current_status = s.group(1)

            n = re.match(r'^\s{2}notes:\s+"?(.+)"?$', line)
            if n and current_symbol:
                current_notes = n.group(1).strip().strip('"')

            # Flush when we hit the next top-level entry or end of block
            if re.match(r'^- ', line) and current_symbol and current_status:
                if current_symbol in wanted:
                    results[current_symbol] = current_status
                    notes_map[current_symbol] = current_notes
                current_symbol = None
                current_status = None
                current_notes = None

        # Final flush
        if current_symbol and current_status and current_symbol in wanted:
            results[current_symbol] = current_status
            notes_map[current_symbol] = current_notes

    missing = wanted - set(results.keys())

    print(f"=== Ledger results for {len(wanted)} symbol(s) ===\n")
    for sym in sorted(results.keys()):
        status = results[sym]
        notes = notes_map.get(sym)
        note_str = f"  notes: {notes}" if notes else ""
        print(f"  {status:<10} {sym}{note_str}")

    for sym in sorted(missing):
        print(f"  MISSING    {sym}")

    print(f"\nSUMMARY: {len(results)} found, {len(missing)} missing from ledger")


# ---------------------------------------------------------------------------
# check-intentional
# ---------------------------------------------------------------------------

def cmd_check_intentional(args):
    wanted = set(args.symbols)

    intentional_path = Path(INTENTIONAL_FILE)
    if not intentional_path.exists():
        print(f"ERROR: {INTENTIONAL_FILE} not found.", file=sys.stderr)
        sys.exit(1)

    try:
        import yaml
        data = yaml.safe_load(intentional_path.read_text()) or []
    except ImportError:
        # Fallback: simple regex parse without PyYAML
        data = _parse_intentional_simple(intentional_path.read_text())

    found = []
    for entry in data:
        cpp = entry.get("cpp_symbol", "")
        if cpp in wanted:
            found.append(entry)

    if found:
        print(f"=== {len(found)} intentional divergence(s) found ===\n")
        for e in found:
            print(f"  cpp_symbol : {e.get('cpp_symbol')}")
            print(f"  rust_symbol: {e.get('rust_symbol', 'n/a')}")
            print(f"  divergence : {e.get('divergence', '')}")
            print(f"  reason     : {e.get('reason', '')}")
            print(f"  date       : {e.get('date', '')}")
            print()
    else:
        not_found = sorted(wanted)
        print(f"No intentional_differences entries for: {not_found}")

    print(f"INTENTIONAL_SYMBOLS: {json.dumps([e.get('cpp_symbol') for e in found])}")


def _parse_intentional_simple(text):
    entries, current = [], {}
    for line in text.splitlines():
        if line.startswith("- ") or line.startswith("  - "):
            key_match = re.match(r'\s*-?\s*(\w+):\s*(.*)', line)
            if key_match:
                key, val = key_match.group(1), key_match.group(2).strip().strip('"')
                if key == "cpp_symbol" and current:
                    entries.append(current)
                    current = {}
                current[key] = val
        else:
            kv = re.match(r'\s+(\w+):\s*(.*)', line)
            if kv:
                current[kv.group(1)] = kv.group(2).strip().strip('"')
    if current:
        entries.append(current)
    return entries


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="upstream-sync-check tooling — run from project root"
    )
    sub = parser.add_subparsers(dest="cmd", required=True)

    sub.add_parser("ensure-upstream", help="Clone or fetch forgottenserver-upstream/")

    p_commits = sub.add_parser("get-commits", help="List recent upstream commits + changed files")
    p_commits.add_argument("--since", default="3 months ago", help="Time window (default: '3 months ago')")

    p_symbols = sub.add_parser("map-symbols", help="Map changed files → C++ symbols")
    p_symbols.add_argument("--files", nargs="+", required=True, metavar="FILE")

    p_ledger = sub.add_parser("check-ledger", help="Query MIGRATION_LEDGER.yml for symbols")
    p_ledger.add_argument("--symbols", nargs="+", required=True, metavar="SYMBOL")

    p_intentional = sub.add_parser("check-intentional", help="Check intentional_differences.yml")
    p_intentional.add_argument("--symbols", nargs="+", required=True, metavar="SYMBOL")

    args = parser.parse_args()
    dispatch = {
        "ensure-upstream": cmd_ensure_upstream,
        "get-commits": cmd_get_commits,
        "map-symbols": cmd_map_symbols,
        "check-ledger": cmd_check_ledger,
        "check-intentional": cmd_check_intentional,
    }
    dispatch[args.cmd](args)


if __name__ == "__main__":
    main()
