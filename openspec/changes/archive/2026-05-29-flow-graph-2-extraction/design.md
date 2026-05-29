## Context

Phase 1 established the YAML format and `make flow`. The C++ tree has ~100+ source files and thousands of symbols; enumerating and edge-linking them by hand is infeasible and context-hostile. `cpp_symbol_manifest.json` already holds the authoritative symbol set with `{file, qualified_name}`. This phase turns that into nodes and adds the call edges a tool can resolve, leaving only the genuinely dynamic edges for human curation.

## Goals / Non-Goals

**Goals:**
- A node per manifest symbol, no parallel symbol universe.
- Statically-resolvable call edges, tagged `static`, derived from source without modifying it.
- Idempotent rebuilds that never clobber curated dynamic edges.

**Non-Goals:**
- Dynamic edges (opcode/event/virtual/scheduled) — phases 3–5.
- A provably-complete call graph; static analysis of C++ is inherently partial.

## Decisions

### D1: Node set seeded from `cpp_symbol_manifest.json`
`bootstrap_nodes.py` reads the manifest and writes a node per symbol into the correct `nodes/<file>.yml`. The boot-spine nodes from phase 1 are merged, not overwritten.

### D2: O1 — static extractor implementation
Detect the toolchain at build time. **Prefer libclang** (accurate AST-level call resolution). **Fall back** to a tree-sitter / heuristic call-site scanner if clang is unavailable. Either way the emitted YAML is identical, so downstream phases are unaffected. The chosen path is recorded in `flow_graph/README.md`.
- *Alternative rejected:* regex-only — too noisy for an exhaustive graph; acceptable only as the documented fallback.

### D3: Merge semantics on rebuild
`build_edges.py` recomputes only `kind: static` edges and merges them in, preserving every `kind: dynamic`/`confidence: curated` edge. Re-running with no source change yields a zero diff (idempotent).

### D4: Coverage reporting
The build reports static-edge coverage (functions with ≥1 resolved out-edge vs. total) so blind spots are visible rather than silent, informing where curation is most needed.

## Risks / Trade-offs

- **clang unavailable / heavy** → documented heuristic fallback; schema unchanged.
- **Static false edges (macros/templates)** → tag confidence; the validator and later spot-checks catch egregious cases; curation overrides where needed.
- **Large diffs on first build** → expected one-time cost; subsequent builds are idempotent.
