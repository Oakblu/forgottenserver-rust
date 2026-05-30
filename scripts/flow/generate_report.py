#!/usr/bin/env python3
"""Generate flow_graph/GAP_REPORT.md from gap_analysis results.

Usage (from repo root):
    python3 scripts/flow/generate_report.py
"""
from __future__ import annotations

import sys
from datetime import date
from pathlib import Path

_HERE = Path(__file__).parent
sys.path.insert(0, str(_HERE.parent / "ledger"))
sys.path.insert(0, str(_HERE))

from gap_analysis import analyze  # noqa: E402

_ROOT = _HERE.parent.parent
_OUT = _ROOT / "flow_graph" / "GAP_REPORT.md"


def _badge(cat: str) -> str:
    return {
        "MISSING_FLOW":  "🔴",
        "DYNAMIC_GAP":   "🟠",
        "BRANCH_GAP":    "🟡",
        "ORDER_MISMATCH": "🔵",
    }.get(cat, "⚪")


def generate(root: Path = _ROOT) -> Path:
    out_path = root / "flow_graph" / "GAP_REPORT.md"
    findings, suppressed = analyze(root)

    lines: list[str] = []
    lines.append("# Flow Graph Gap Report")
    lines.append("")
    lines.append(f"_Generated {date.today()} — do not edit by hand. Re-run `make flow-gap`._")
    lines.append("")

    # Summary
    lines.append("## Summary")
    lines.append("")
    by_cat: dict[str, int] = {}
    for f in findings:
        by_cat[f.category] = by_cat.get(f.category, 0) + 1
    lines.append(f"| Category | Count |")
    lines.append(f"|---|---|")
    for cat in ("MISSING_FLOW", "DYNAMIC_GAP", "BRANCH_GAP", "ORDER_MISMATCH"):
        n = by_cat.get(cat, 0)
        lines.append(f"| {_badge(cat)} {cat} | {n} |")
    lines.append(f"| Suppressed (intentional) | {len(suppressed)} |")
    lines.append(f"| **Total actionable** | **{len(findings)}** |")
    lines.append("")

    if not findings:
        lines.append("> ✅ **No actionable gaps found.** "
                     "All reachable C++ paths have verified Rust counterparts.")
        lines.append("")
    else:
        # Actionable findings table
        lines.append("## Actionable Findings")
        lines.append("")
        lines.append("Sorted by priority (descending). "
                     "Higher priority = shallower in call graph + more incomplete.")
        lines.append("")
        lines.append("| # | Cat | Pri | Depth | Symbol | Ledger | Rust Symbol | Note |")
        lines.append("|---|---|---|---|---|---|---|---|")
        for i, f in enumerate(findings, 1):
            badge = _badge(f.category)
            note = f.note.replace("|", "\\|")
            rust = (f.rust_symbol or "—").replace("|", "\\|")
            condition = f" `{f.condition}`" if f.condition else ""
            lines.append(
                f"| {i} | {badge} {f.category} | {f.priority:.3f} | {f.depth} | "
                f"`{f.qualified_name}` | {f.ledger_status} | "
                f"`{rust}` | {note}{condition} |"
            )
        lines.append("")

        # Per-file breakdown
        by_file: dict[str, list] = {}
        for f in findings:
            by_file.setdefault(f.cpp_file, []).append(f)
        lines.append("## By Source File")
        lines.append("")
        for cfile, flist in sorted(by_file.items(), key=lambda x: -len(x[1])):
            lines.append(f"### `{cfile}` ({len(flist)} finding(s))")
            lines.append("")
            for f in flist:
                cond = f" — `{f.condition}`" if f.condition else ""
                lines.append(
                    f"- {_badge(f.category)} **{f.category}** "
                    f"`{f.qualified_name}` (depth={f.depth}){cond}"
                )
                lines.append(f"  - {f.note}")
            lines.append("")

    # Suppressed appendix
    lines.append("## Suppressed Findings (Intentional Differences)")
    lines.append("")
    if not suppressed:
        lines.append("_None._")
    else:
        lines.append(
            "These findings were suppressed because the symbol appears in "
            "`intentional_differences.yml`."
        )
        lines.append("")
        lines.append("| Cat | Symbol | Ledger | Note |")
        lines.append("|---|---|---|---|")
        for f in suppressed:
            note = f.note.replace("|", "\\|")
            lines.append(
                f"| {_badge(f.category)} {f.category} | "
                f"`{f.qualified_name}` | {f.ledger_status} | {note} |"
            )
    lines.append("")

    out_path.write_text("\n".join(lines) + "\n")
    return out_path


def main() -> int:
    try:
        out = generate()
        findings, suppressed = analyze()
        print(f"GAP_REPORT.md written to {out} "
              f"({len(findings)} actionable, {len(suppressed)} suppressed)")
        return 0
    except Exception as exc:  # noqa: BLE001
        print(f"ERROR: {exc}", file=sys.stderr)
        import traceback; traceback.print_exc(file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
