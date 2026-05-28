#!/usr/bin/env python3
"""Patches shield.io badge URLs in README.md with fresh test/coverage metrics."""

import re
import sys


def coverage_color(pct: float) -> str:
    if pct >= 90:
        return "brightgreen"
    if pct >= 80:
        return "green"
    if pct >= 70:
        return "yellow"
    if pct >= 60:
        return "orange"
    return "red"


def patch_readme(
    path: str,
    test_count: int,
    coverage_pct: float,
    loc_k: str,
    crates_count: int,
) -> None:
    with open(path) as f:
        content = f.read()

    tests_color = "brightgreen" if test_count > 0 else "red"

    patches = [
        (
            r"badge/tests-[^?]+(?=\?style=flat-square)",
            f"badge/tests-{test_count}%20passing-{tests_color}",
            "tests",
        ),
        (
            r"badge/coverage-[^?]+(?=\?style=flat-square)",
            f"badge/coverage-{coverage_pct:.2f}%25-{coverage_color(coverage_pct)}",
            "coverage",
        ),
        (
            r"badge/rust%20LOC-[^?]+(?=\?style=flat-square)",
            f"badge/rust%20LOC-{loc_k}-lightgrey",
            "loc",
        ),
        (
            r"badge/crates-[^?]+(?=\?style=flat-square&logo=rust&logoColor=white)",
            f"badge/crates-{crates_count}-orange",
            "crates",
        ),
    ]

    for pattern, replacement, label in patches:
        m = re.search(pattern, content)
        if not m:
            sys.exit(f"ERROR: badge '{label}' pattern not found in {path}")
        old = m.group(0)
        content = re.sub(pattern, replacement, content)
        if old != replacement:
            print(f"{label}: {old} -> {replacement}")

    with open(path, "w") as f:
        f.write(content)


if __name__ == "__main__":
    if len(sys.argv) != 5:
        sys.exit(f"Usage: {sys.argv[0]} <test_count> <coverage_pct> <loc_k> <crates_count>")

    patch_readme(
        "README.md",
        int(sys.argv[1]),
        float(sys.argv[2]),
        sys.argv[3],
        int(sys.argv[4]),
    )
