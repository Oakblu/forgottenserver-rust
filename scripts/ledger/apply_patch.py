#!/usr/bin/env python3
"""Merge per-layer audit patches into MIGRATION_LEDGER.yml.

A patch file is JSON with shape:

    {
      "layer": 7,
      "scope_files": ["src/combat.cpp", "src/combat.h", ...],
      "audited_at": "2026-05-26T...",
      "rows": [
        {"id": "<row-id>", "rust": [...], "status": "...", "migration_type": "...", "confidence": "...", "evidence": [...], "semantic_differences": [...], "risks": [...], "tests_required": [...], "differential_scenarios": [...], "notes": "..."},
        ...
      ]
    }

A checkpoint file (used with --checkpoint) is JSON with shape:

    {
      "phase": 1,
      "layer": 0,
      "completed_at": "2026-05-26",
      "files_passed": 16,
      "files_blocked": 0,
      "blockers": [],
      "commands_run": ["rollup.py", "validate.py", "..."],
      "verification_status": "PASSED"
    }

Constraints enforced by the merger:

- Every `rows[*].id` must already exist in the ledger (created by
  build_seed.py). New rows cannot be introduced via patches.
- Every `rows[*]` must belong to one of the patch's `scope_files`
  (its current `cpp.file` matches one of them). A patch cannot edit
  rows outside its declared scope.
- `cpp.*` fields are NEVER overwritten by a patch (they are owned by
  the manifest). If a patch row contains a `cpp` key it is ignored.
- After merging all patches, the result must pass `validate.py`. If
  not, the merge is rolled back and the offending patch is reported.

Usage:
    python3 scripts/ledger/apply_patch.py audit_patches/layer-0.json [...]
    python3 scripts/ledger/apply_patch.py --dry-run audit_patches/layer-7.json
    python3 scripts/ledger/apply_patch.py --all audit_patches/
    python3 scripts/ledger/apply_patch.py --checkpoint /tmp/phase1_checkpoint.json
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import shutil
import sys
from typing import Any

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import ledger_io  # noqa: E402

DEFAULT_ROOT = os.path.abspath(os.path.join(HERE, os.pardir, os.pardir))


PATCH_KEYS = {
    "rust",
    "status",
    "migration_type",
    "confidence",
    "evidence",
    "semantic_differences",
    "risks",
    "tests_required",
    "differential_scenarios",
    "notes",
}


class PatchError(Exception):
    pass


def load_patch(path: str) -> dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        patch = json.load(f)
    for required in ("layer", "scope_files", "rows"):
        if required not in patch:
            raise PatchError(f"{path}: missing required key {required!r}")
    return patch


def apply_patch_to_doc(doc: dict[str, Any], patch: dict[str, Any], patch_path: str) -> dict[str, str]:
    rows_by_id = {r["id"]: r for r in doc.get("symbols", [])}
    scope = set(patch["scope_files"])
    stats = {"updated": 0, "skipped_not_in_scope": 0, "skipped_unknown_id": 0}

    for entry in patch["rows"]:
        rid = entry.get("id")
        if not rid:
            raise PatchError(f"{patch_path}: row has no id")
        target = rows_by_id.get(rid)
        if target is None:
            stats["skipped_unknown_id"] += 1
            raise PatchError(f"{patch_path}: id={rid} not in ledger")
        if target["cpp"]["file"] not in scope:
            stats["skipped_not_in_scope"] += 1
            raise PatchError(
                f"{patch_path}: id={rid} cpp.file={target['cpp']['file']} outside declared scope"
            )
        for k, v in entry.items():
            if k == "id":
                continue
            if k == "cpp":
                # cpp.* is owned by the manifest, ignore.
                continue
            if k not in PATCH_KEYS:
                raise PatchError(f"{patch_path}: id={rid} unknown key {k!r}")
            target[k] = v
        stats["updated"] += 1
    return stats


def apply_checkpoint(doc: dict[str, Any], checkpoint: dict[str, Any]) -> None:
    required = {"phase", "layer", "completed_at", "verification_status"}
    missing = required - set(checkpoint.keys())
    if missing:
        raise PatchError(f"checkpoint missing keys: {missing}")
    if "checkpoints" not in doc:
        doc["checkpoints"] = []
    doc["checkpoints"].append(checkpoint)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Apply audit patches to MIGRATION_LEDGER.yml")
    parser.add_argument("patches", nargs="*", help="Patch JSON files (or directories with --all)")
    parser.add_argument("--ledger", default=os.path.join(DEFAULT_ROOT, "MIGRATION_LEDGER.yml"))
    parser.add_argument("--all", action="store_true", help="Treat positional args as directories and apply every *.json")
    parser.add_argument("--dry-run", action="store_true", help="Don't write the ledger; just verify")
    parser.add_argument("--skip-validate", action="store_true", help="Skip running validate.py at the end (faster)")
    parser.add_argument("--checkpoint", metavar="FILE", help="Append a phase checkpoint record to the ledger (JSON file)")
    args = parser.parse_args(argv)

    if args.checkpoint and not args.patches and not args.all:
        # Checkpoint-only mode: append checkpoint record, no symbol patches.
        backup_path = args.ledger + ".pre-merge"
        shutil.copy(args.ledger, backup_path)
        try:
            with open(args.checkpoint, "r", encoding="utf-8") as f:
                cp = json.load(f)
            with open(args.ledger, "r", encoding="utf-8") as f:
                doc = ledger_io.load(f.read())
            apply_checkpoint(doc, cp)
            if not args.dry_run:
                with open(args.ledger, "w", encoding="utf-8") as f:
                    f.write(ledger_io.dump(doc))
                os.remove(backup_path)
                print(f"Checkpoint phase={cp['phase']} layer={cp['layer']} appended to ledger.")
            else:
                os.remove(backup_path)
                print(f"--dry-run: would append checkpoint phase={cp['phase']} layer={cp['layer']}")
            return 0
        except (PatchError, KeyError) as e:
            shutil.copy(backup_path, args.ledger)
            os.remove(backup_path)
            print(f"ERROR: {e}")
            return 1
        except Exception:
            shutil.copy(backup_path, args.ledger)
            os.remove(backup_path)
            raise

    if args.all:
        patch_paths: list[str] = []
        for d in args.patches or [os.path.join(DEFAULT_ROOT, "audit_patches")]:
            # Patch files follow the `layer-<N>.json` naming convention.
            # `layer_scopes.json` and other metadata files in the same
            # directory are intentionally excluded.
            patch_paths.extend(sorted(glob.glob(os.path.join(d, "layer-*.json"))))
    else:
        patch_paths = list(args.patches)

    if not patch_paths:
        print("No patch files supplied; pass paths or use --all.")
        return 2

    backup_path = args.ledger + ".pre-merge"
    shutil.copy(args.ledger, backup_path)

    try:
        with open(args.ledger, "r", encoding="utf-8") as f:
            doc = ledger_io.load(f.read())

        total_updated = 0
        for p in patch_paths:
            patch = load_patch(p)
            stats = apply_patch_to_doc(doc, patch, p)
            print(f"{os.path.basename(p)}: updated={stats['updated']}")
            total_updated += stats["updated"]

        if args.dry_run:
            if not args.skip_validate:
                # Validate the in-memory merged state by writing to a
                # temp file. The on-disk ledger is unchanged.
                tmp = args.ledger + ".dry-run"
                with open(tmp, "w", encoding="utf-8") as f:
                    f.write(ledger_io.dump(doc))
                try:
                    import validate as validator
                    rc = validator.main(["--ledger", tmp])
                finally:
                    if os.path.exists(tmp):
                        os.remove(tmp)
                if rc != 0:
                    print("\nERROR: dry-run validation rejected the merged ledger.")
                    return 1
            print(f"--dry-run: would update {total_updated} rows across {len(patch_paths)} patches")
            os.remove(backup_path)
            return 0

        if args.checkpoint:
            with open(args.checkpoint, "r", encoding="utf-8") as f:
                cp = json.load(f)
            apply_checkpoint(doc, cp)

        with open(args.ledger, "w", encoding="utf-8") as f:
            f.write(ledger_io.dump(doc))

        if not args.skip_validate:
            import validate as validator
            rc = validator.main(["--ledger", args.ledger])
            if rc != 0:
                # Roll back.
                shutil.copy(backup_path, args.ledger)
                print(f"\nERROR: validate.py rejected the merged ledger. Restored from {backup_path}.")
                return 1

        os.remove(backup_path)
        print(f"\nApplied {total_updated} row updates from {len(patch_paths)} patches.")
        if args.checkpoint:
            print(f"Checkpoint phase={cp['phase']} layer={cp['layer']} appended.")
        return 0
    except PatchError as e:
        shutil.copy(backup_path, args.ledger)
        os.remove(backup_path)
        print(f"ERROR: {e}")
        return 1
    except Exception:
        shutil.copy(backup_path, args.ledger)
        os.remove(backup_path)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
