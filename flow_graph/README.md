# Flow Graph

The flow graph encodes every C++ execution path reachable from `main()`, sharded by source file.
It is the source of truth for understanding which C++ flows exist and how control reaches them.

## File layout

```
flow_graph/
  README.md             — this file
  index.yml             — root declaration and entrypoint chain
  FLOW_GRAPH.md         — generated Markdown view (do not hand-edit; produced by render_markdown.py)
  nodes/
    main.yml            — nodes for src/main.cpp
    otserv.yml          — nodes for src/otserv.cpp
    <file>.yml          — one shard per C++ source file
```

## Node schema

Each shard file under `flow_graph/nodes/` contains a top-level `nodes:` list.
Every node identifies its C++ symbol by the same `{file, qualified_name}` key used in
`cpp_symbol_manifest.json` and `MIGRATION_LEDGER.yml`.

```yaml
nodes:
  - file: "src/example.cpp"
    qualified_name: "ClassName::methodName"
    edges:
      - target:
          file: "src/other.cpp"
          qualified_name: "Other::symbol"
        kind: static        # static | dynamic
        confidence: curated # static | curated
        order: 1            # optional: call sequence within caller
        condition: "guard"  # optional: branch guard text
```

### Node fields

| Field | Required | Description |
|-------|----------|-------------|
| `file` | yes | C++ source file path (relative to repo root, e.g. `"src/game.cpp"`) |
| `qualified_name` | yes | C++ qualified name matching `cpp_symbol_manifest.json` |
| `edges` | yes | Out-edges (adjacency list); use `[]` for none |

### Edge fields

| Field | Required | Values | Description |
|-------|----------|--------|-------------|
| `target` | yes | `{file, qualified_name}` | Target symbol — must exist in `cpp_symbol_manifest.json` |
| `kind` | yes | `static` \| `dynamic` | Static call-site or dynamic dispatch (opcode/event/virtual) |
| `confidence` | yes | `static` \| `curated` | `static` = extracted by tool; `curated` = hand-authored |
| `order` | no | integer | Call sequence within the caller (disambiguates multiple edges to same target) |
| `condition` | no | string | Branch guard under which this edge fires |

## index.yml

Declares the graph root and the default traversal entrypoint chain:

```yaml
root:
  file: "src/main.cpp"
  qualified_name: main

entrypoint_chain:
  - file: "src/main.cpp"
    qualified_name: main
  - file: "src/otserv.cpp"
    qualified_name: startServer
  - file: "src/otserv.cpp"
    qualified_name: mainLoader

unreached: []
```

`unreached` lists nodes intentionally excluded from the reachability check, each with a `reason`.

## YAML encoding rules

The graph YAML follows the same canonical-subset rules as `MIGRATION_LEDGER.yml`
(see `scripts/ledger/ledger_io.py`):

- Block style only — no inline `{...}` flow mappings.
- Strings with `/`, `.`, `::`, spaces, or other non-identifier characters use JSON quoting:
  `"src/game.cpp"`, `"Game::setGameState"`.
- Simple identifiers are bare: `static`, `curated`, `main`.
- Integers are bare: `1`, `42`.
- Empty lists are `[]`.

## Generated view

`flow_graph/FLOW_GRAPH.md` is produced by `scripts/flow/render_markdown.py` (phase 6).
**Do not hand-edit it.**

## Static extractor — O1 decision

`scripts/flow/build_edges.py` uses a **heuristic regex extractor** (the fallback path).

**Detection result:** neither `libclang` (Python `clang.cindex`) nor `tree-sitter` is
available on this host (PEP-668 externally-managed Python prevents pip installs).

**What the heuristic covers:**
1. *Qualified calls* — `ClassName::method(` and `ns::func(` patterns matched against
   `cpp_symbol_manifest.json`; tagged `kind: static, confidence: static`.
2. *Member calls via known globals* — `g_foo.method(` and `g_foo->method(` resolved
   through file-scope `Type g_foo;` declarations (e.g. `Game g_game` → `Game::start`).

**What it misses (low-confidence or skipped):**
- Calls via local variables of unknown type.
- Function-pointer and lambda calls.
- Macro-expanded calls.
- Virtual dispatch (covered by phase 5 curated edges).

Edges extracted by the heuristic carry `confidence: static`.
Hand-authored edges carry `confidence: curated` and are never overwritten.

## Validator

`make flow` runs `scripts/flow/validate.py`, which checks:

1. Every node `{file, qualified_name}` key resolves in `cpp_symbol_manifest.json`.
2. No edge targets a missing manifest key (dangling edge).
3. Every node is reachable from the root via out-edges, or explicitly listed in `unreached`.

Exit non-zero with the offending item named on any violation.
