//! Lua binding static audit.
//!
//! Parses the C++ `luascript.cpp` for every `LuaScriptInterface::register*`
//! call (and macro expansion) and emits a list of every binding the C++
//! server exposes to Lua scripts under `data/`. Parses the Rust
//! `crates/scripting/` tree for every mlua registration site. Diffs the
//! two surfaces and produces a JSON report consumable by the harness
//! ledger writer.
//!
//! This is the static sub-lane of Phase 2 in the equivalence harness.
//! The runtime sub-lane (per-binding executable test corpus) is shipped
//! incrementally via the `forgottenserver-rust-lua-binding-corpus-<group>`
//! follow-up changes.
//!
//! ## Output shape
//!
//! ```json
//! {
//!   "lane": "lua_bindings",
//!   "sub_lane": "static",
//!   "status": "PASS" | "FAIL",
//!   "cpp_total": 547,
//!   "rust_total": 12,
//!   "missing_in_rust": ["Player:getLevel", "Game.createMonster", ...],
//!   "unexpected_in_rust": [],
//!   "by_kind": { "ClassMethod": 380, "GlobalEnum": 120, ... },
//!   "ledger_entries": [
//!     {"cpp": "src/luascript.cpp", "transition": "...", "reason": "..."}
//!   ]
//! }
//! ```

use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Hash)]
pub enum BindingKind {
    /// `registerMethod(L, "Class", "name", fn)` / `methods.add_method("name", ...)`
    ClassMethod,
    /// `add_field_method_get/set("name", ...)` (Rust-side concept; C++ uses
    /// `registerMetaMethod` for `__index`/`__newindex`)
    ClassField,
    /// `registerGlobalMethod(L, "name", fn)` / `globals.set("name", create_function(...))`
    GlobalFunction,
    /// `registerEnum(L, ENUM_VALUE)` — top-level enum constant
    GlobalEnum,
    /// `registerEnumIn(L, "table", ENUM_VALUE)` — enum constant inside a table
    TableEnum,
    /// `registerVariable(L, "table", "name", value)` — variable inside a table
    TableVariable,
    /// `registerClass(L, "ClassName", "BaseClass", newFunction)` — class metatable
    Class,
    /// `registerTable(L, "TableName")` — table init
    Table,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Binding {
    /// Fully-qualified binding name; collision-free across kinds via prefix.
    pub name: String,
    pub kind: BindingKind,
    pub source_file: String,
    pub source_line: u32,
}

impl Binding {
    pub fn key(&self) -> (BindingKind, String) {
        (self.kind, self.name.clone())
    }
}

#[derive(Debug, Default, Serialize)]
pub struct AuditReport {
    pub lane: String,
    pub sub_lane: String,
    pub status: String,
    pub cpp_total: usize,
    pub rust_total: usize,
    pub missing_in_rust: Vec<String>,
    pub unexpected_in_rust: Vec<String>,
    pub by_kind_cpp: BTreeMap<String, usize>,
    pub by_kind_rust: BTreeMap<String, usize>,
    pub ledger_entries: Vec<LedgerEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LedgerEntry {
    pub cpp: String,
    pub transition: String,
    pub reason: String,
}

// ── C++ parser ───────────────────────────────────────────────────────────────

/// Parse `forgottenserver/src/luascript.cpp` for every binding
/// registration. Returns one `Binding` per call site.
///
/// Handles single-line invocations of the `register*` family. The C++
/// codebase uses one-call-per-line consistently inside
/// `registerFunctions()`, so the line-based parser is accurate enough
/// for an audit.
pub fn parse_cpp_bindings(path: &Path) -> anyhow::Result<Vec<Binding>> {
    let content = fs::read_to_string(path)?;
    let file_label = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("<unknown>")
        .to_string();

    // Identifier patterns:
    //   * `IDENT`   = bare C identifier (function names, enum constants)
    //   * `QIDENT`  = possibly-qualified identifier
    //                 (`LuaScriptInterface::luaFoo`, `ConfigManager::IP`)
    //   * Function-arg slot uses QIDENT so qualified callbacks are captured.
    let q_ident = r"[A-Za-z_][A-Za-z0-9_:]*";
    let upper_qident = r"[A-Z][A-Za-z0-9_:]*";

    let re_method = Regex::new(&format!(
        r#"registerMethod\(\s*L\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*{q_ident}\s*\)"#
    ))?;
    let re_meta = Regex::new(&format!(
        r#"registerMetaMethod\(\s*L\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*{q_ident}\s*\)"#
    ))?;
    let re_global_method = Regex::new(&format!(
        r#"registerGlobalMethod\(\s*L\s*,\s*"([^"]+)"\s*,\s*{q_ident}\s*\)"#
    ))?;
    // 4th `newFn` arg is optional — `registerClass(L, "XMLNode", "");` is valid.
    let re_class =
        Regex::new(r#"registerClass\(\s*L\s*,\s*"([^"]+)"\s*,\s*"([^"]*)"\s*(?:,\s*[^)]+)?\)"#)?;
    let re_table = Regex::new(r#"registerTable\(\s*L\s*,\s*"([^"]+)"\s*\)"#)?;
    let re_variable =
        Regex::new(r#"registerVariable\(\s*L\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*[^)]+\)"#)?;
    let re_enum = Regex::new(&format!(
        r#"registerEnum\(\s*L\s*,\s*({upper_qident})\s*\)"#
    ))?;
    let re_enum_in = Regex::new(&format!(
        r#"registerEnumIn\(\s*L\s*,\s*"([^"]+)"\s*,\s*({upper_qident})\s*\)"#
    ))?;

    let mut out = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let trimmed = line.trim_start();
        // Skip lines inside comments / inside the helper-function definitions.
        if trimmed.starts_with("//") || trimmed.starts_with("*") || trimmed.starts_with("#define") {
            continue;
        }

        for cap in re_method.captures_iter(line) {
            out.push(Binding {
                name: format!("{}:{}", &cap[1], &cap[2]),
                kind: BindingKind::ClassMethod,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
        for cap in re_meta.captures_iter(line) {
            out.push(Binding {
                name: format!("{}:{}", &cap[1], &cap[2]),
                kind: BindingKind::ClassMethod,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
        for cap in re_global_method.captures_iter(line) {
            out.push(Binding {
                name: cap[1].to_string(),
                kind: BindingKind::GlobalFunction,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
        for cap in re_class.captures_iter(line) {
            out.push(Binding {
                name: cap[1].to_string(),
                kind: BindingKind::Class,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
        for cap in re_table.captures_iter(line) {
            out.push(Binding {
                name: cap[1].to_string(),
                kind: BindingKind::Table,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
        for cap in re_variable.captures_iter(line) {
            out.push(Binding {
                name: format!("{}.{}", &cap[1], &cap[2]),
                kind: BindingKind::TableVariable,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
        for cap in re_enum.captures_iter(line) {
            out.push(Binding {
                name: cap[1].to_string(),
                kind: BindingKind::GlobalEnum,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
        for cap in re_enum_in.captures_iter(line) {
            // Mirror the C++ `registerEnumIn` macro's behaviour: strip the
            // qualified prefix (`ConfigManager::IP` → `IP`) so the captured
            // name matches what's exposed to Lua (`configKeys.IP`).
            let enum_name = cap[2].rsplit("::").next().unwrap_or(&cap[2]);
            out.push(Binding {
                name: format!("{}.{}", &cap[1], enum_name),
                kind: BindingKind::TableEnum,
                source_file: file_label.clone(),
                source_line: line_no,
            });
        }
    }
    Ok(out)
}

// ── Rust parser ──────────────────────────────────────────────────────────────

/// Walk the `crates/scripting/` directory (and any other Rust
/// directories provided) for mlua registration sites. Identifies:
///
///   - `globals.set("name", lua.create_function(...))` → GlobalFunction
///   - `lua.globals().set("name", ...)` → GlobalFunction
///   - `methods.add_method("name", ...)` inside `impl UserData for X` → ClassMethod
///   - `methods.add_field_method_get/set("name", ...)` → ClassField
///
/// **Multi-line aware.** mlua method registrations frequently span
/// multiple lines (closure bodies + arg types push the call past
/// rustfmt's width). The parser scans the full file with a regex that
/// matches across newlines, then attributes each match to its
/// containing `impl UserData for X` block via a pre-computed range
/// map.
///
/// By convention, `LuaXxx` newtypes (required by Rust orphan rules
/// when wrapping types like `common::Position`) have their `Lua`
/// prefix stripped in the audit so binding names line up with C++.
pub fn parse_rust_bindings(root: &Path) -> anyhow::Result<Vec<Binding>> {
    // `\s*` between `globals()` and `.set` tolerates rustfmt-broken
    // chains like `lua.globals()\n    .set("NAME", …)`.
    let re_globals_set = Regex::new(r#"(?:lua\.)?globals\(\)\s*\.set\(\s*"([^"]+)"\s*,"#)?;
    // Permissive `<any_ident>.set("NAME", …)` for table_enum files.
    let re_table_set = Regex::new(r#"\.set\(\s*"([^"]+)"\s*,"#)?;
    let re_add_method = Regex::new(r#"\.add_method(?:_mut)?\(\s*"([^"]+)""#)?;
    let re_add_function = Regex::new(r#"\.add_function(?:_mut)?\(\s*"([^"]+)""#)?;
    let re_add_field_get = Regex::new(r#"\.add_field_method_get\(\s*"([^"]+)""#)?;
    let re_add_field_set = Regex::new(r#"\.add_field_method_set\(\s*"([^"]+)""#)?;
    // mlua meta-methods: `.add_meta_method(MetaMethod::Eq, …)` etc.
    // The variant name is translated to its Lua `__name` form below.
    let re_add_meta_method =
        Regex::new(r#"\.add_meta_method(?:_mut)?\(\s*(?:mlua::)?MetaMethod::(\w+)"#)?;
    let re_impl_userdata =
        Regex::new(r#"impl(?:<[^>]*>)?\s+(?:mlua::)?UserData\s+for\s+(\w+)(?:<[^>]*>)?\s*\{"#)?;
    // Batch-stub pattern. Matches code like:
    //   for n in &[
    //       "foo", "bar",
    //   ] {
    //       methods.add_method(n, |_, _, ()| Ok(0));
    //   }
    // …and emits one ClassMethod per literal inside the array. The body
    // must call `add_method`/`add_method_mut`/`add_function`/etc. with
    // `n` as the first arg to qualify, so unrelated `for n in &[…]`
    // blocks are not picked up.
    let re_for_batch = Regex::new(
        r#"(?s)for\s+\w+\s+in\s+&\[\s*([^\]]*?)\s*\]\s*\{[^}]*?(?:methods\.)?(?:add_method|add_method_mut|add_function|add_function_mut|add_meta_method|add_meta_method_mut|add_field_method_get|add_field_method_set)\(\s*\*?\w+\s*,"#,
    )?;
    // Per-class helper-fn pattern. Matches call sites like
    //   for_each_stub_int(methods, &["foo", "bar"]);
    // …where the helper internally loops over the slice and calls
    // `methods.add_method(name, …)`. The audit treats the helper
    // closure as opaque and trusts the call site's literal array.
    let re_helper_call =
        Regex::new(r#"(?s)\b\w+\s*\(\s*methods\s*,\s*&\[\s*([^\]]*?)\s*\]\s*[,)]"#)?;
    let re_str_lit = Regex::new(r#""([^"]+)""#)?;
    // Manual audit-marker comment. Use these for bindings that don't fit
    // the standard `add_method`/`globals().set` patterns (e.g. registering
    // methods on the Lua standard library tables `table`, `os`, etc.).
    //
    //   // AUDIT: ClassMethod table:create table:pack
    //   // AUDIT: GlobalFunction print broadcast
    //
    // Each whitespace-separated token after the kind is one binding name.
    let re_audit_comment = Regex::new(r#"//\s*AUDIT:\s*(\w+)\s+([^\n]+)"#)?;
    // Pre-compiled UserData reference detector — used in the Table
    // heuristic for `lua.globals().set("X", classes::xxx::LuaXxx)`.
    let re_userdata_value = Regex::new(r"\bLua[A-Z]\w*\b")?;

    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let raw_content = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        // Strip `#[cfg(test)] mod tests { … }` blocks — they contain
        // `lua.globals().set("x", …)` calls and other patterns that
        // pollute the audit with binding-name noise from test helpers.
        let content = strip_test_modules(&raw_content);
        let file_label = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        // Pre-compute every `impl UserData for X { ... }` byte range
        // by matching the opener and brace-counting forward to the
        // closing `}`. Newtype `Lua` prefix is stripped at this step.
        let impl_ranges = find_impl_ranges(&content, &re_impl_userdata);

        // Helper: byte offset → 1-based line number.
        let line_at = |offset: usize| -> u32 {
            content[..offset.min(content.len())]
                .bytes()
                .filter(|&b| b == b'\n')
                .count() as u32
                + 1
        };
        // Helper: byte offset → enclosing impl class name (if any).
        let class_at = |offset: usize| -> Option<&str> {
            impl_ranges
                .iter()
                .find(|(s, e, _)| *s <= offset && offset < *e)
                .map(|(_, _, n)| n.as_str())
        };

        // Every `impl UserData for LuaXxx { … }` corresponds to the
        // C++ `registerClass(L, "Xxx", …)` declaration. Emit a `Class`
        // binding so the audit matches both sides. (`Lua` prefix is
        // already stripped by `find_impl_ranges`.) `__gc` is opt-in via
        // `// AUDIT: ClassMethod Xxx:__gc` — C++ only registers it for
        // classes that own non-Lua resources.
        for (start, _, name) in &impl_ranges {
            out.push(Binding {
                name: name.clone(),
                kind: BindingKind::Class,
                source_file: file_label.clone(),
                source_line: line_at(*start),
            });
        }

        // `globals().set("NAME", …)` — kind depends on file location:
        //   * Files under `lua_bindings/enums/` register integer
        //     constants → GlobalEnum.
        //   * Files under `lua_bindings/table_enums/` register
        //     table-scoped constants → TableEnum (the table prefix is
        //     pulled from the file name). These files use the
        //     `<table_ident>.set("NAME", …)` pattern (NOT
        //     `globals().set`), so we use a permissive regex below.
        //   * Everything else (constructors, free functions) →
        //     GlobalFunction.
        let path_str = file_label.replace('\\', "/");
        let is_enum_module = path_str.contains("lua_bindings/enums/");
        let table_module_name = path_str
            .split("lua_bindings/table_enums/")
            .nth(1)
            .and_then(|tail| tail.split('.').next())
            // `mod.rs` is the module hub — it just re-exports the
            // siblings; there's no actual `Table:mod` to register.
            .filter(|name| *name != "mod")
            .map(str::to_string);

        if table_module_name.is_some() {
            // Look for the wrapping `lua.globals().set("<TABLE>", table)`
            // call in this file to discover the actual Lua-visible table
            // name (which may differ from the snake_case file name —
            // e.g. file `config_keys.rs` exposes `configKeys`).
            let wrapping = re_globals_set.captures_iter(&content).next();
            let (table_name, wrapping_offset) = match wrapping {
                Some(c) => (c[1].to_string(), c.get(0).map(|m| m.start()).unwrap_or(0)),
                None => (table_module_name.clone().unwrap(), 0),
            };
            // The wrapping `globals().set("<TABLE>", …)` is itself the
            // C++ `registerTable(L, "<TABLE>")` — emit it as Table.
            out.push(Binding {
                name: table_name.clone(),
                kind: BindingKind::Table,
                source_file: file_label.clone(),
                source_line: line_at(wrapping_offset),
            });
            // Permissive: match `<ident>.set("NAME", …)`. Exclude the
            // wrapping `globals().set("configKeys", table)` call by
            // skipping captures whose name equals the table name.
            for cap in re_table_set.captures_iter(&content) {
                let m = cap.get(0).unwrap();
                if cap[1] == table_name {
                    continue;
                }
                out.push(Binding {
                    name: format!("{table_name}.{}", &cap[1]),
                    kind: BindingKind::TableEnum,
                    source_file: file_label.clone(),
                    source_line: line_at(m.start()),
                });
            }
        } else {
            for cap in re_globals_set.captures_iter(&content) {
                let m = cap.get(0).unwrap();
                let name = cap[1].to_string();
                let kind = if is_enum_module {
                    BindingKind::GlobalEnum
                } else {
                    BindingKind::GlobalFunction
                };
                out.push(Binding {
                    name: name.clone(),
                    kind,
                    source_file: file_label.clone(),
                    source_line: line_at(m.start()),
                });
                // Heuristic: if the value passed to `globals().set(…)`
                // is a UserData reference (i.e. it textually contains a
                // `LuaXxx` newtype path like `classes::game::LuaGame`),
                // also emit a `Table` binding. C++ exposes singletons
                // like `Game` via `registerTable`, not `registerClass`,
                // so this keeps the audit aligned.
                if !is_enum_module {
                    let value_window_start = m.end();
                    let value_window_end = (value_window_start + 200).min(content.len());
                    let value_slice = &content[value_window_start..value_window_end];
                    if re_userdata_value.is_match(value_slice) {
                        out.push(Binding {
                            name,
                            kind: BindingKind::Table,
                            source_file: file_label.clone(),
                            source_line: line_at(m.start()),
                        });
                    }
                }
            }
        }
        // Batch-stub helper pattern: `for n in &["foo","bar"] { methods.add_method(n,…) }`.
        // Emits one ClassMethod per literal in the array, using the
        // enclosing impl block's class name for context.
        for cap in re_for_batch.captures_iter(&content) {
            let array = cap.get(1).unwrap();
            let m = cap.get(0).unwrap();
            let cls = class_at(m.start());
            for lit in re_str_lit.captures_iter(array.as_str()) {
                let name = match cls {
                    Some(c) => format!("{c}:{}", &lit[1]),
                    None => lit[1].to_string(),
                };
                out.push(Binding {
                    name,
                    kind: BindingKind::ClassMethod,
                    source_file: file_label.clone(),
                    source_line: line_at(m.start()),
                });
            }
        }
        // Per-class helper-fn pattern. `for_each_stub_*(methods, &["a", "b"])`.
        for cap in re_helper_call.captures_iter(&content) {
            let array = cap.get(1).unwrap();
            let m = cap.get(0).unwrap();
            let cls = class_at(m.start());
            for lit in re_str_lit.captures_iter(array.as_str()) {
                let name = match cls {
                    Some(c) => format!("{c}:{}", &lit[1]),
                    None => lit[1].to_string(),
                };
                out.push(Binding {
                    name,
                    kind: BindingKind::ClassMethod,
                    source_file: file_label.clone(),
                    source_line: line_at(m.start()),
                });
            }
        }

        // add_method (multi-line).
        for cap in re_add_method.captures_iter(&content) {
            let m = cap.get(0).unwrap();
            let name = match class_at(m.start()) {
                Some(c) => format!("{c}:{}", &cap[1]),
                None => cap[1].to_string(),
            };
            out.push(Binding {
                name,
                kind: BindingKind::ClassMethod,
                source_file: file_label.clone(),
                source_line: line_at(m.start()),
            });
        }
        for cap in re_add_function.captures_iter(&content) {
            let m = cap.get(0).unwrap();
            let name = match class_at(m.start()) {
                Some(c) => format!("{c}.{}", &cap[1]),
                None => cap[1].to_string(),
            };
            out.push(Binding {
                name,
                kind: BindingKind::ClassMethod,
                source_file: file_label.clone(),
                source_line: line_at(m.start()),
            });
        }
        for cap in re_add_field_get.captures_iter(&content) {
            let m = cap.get(0).unwrap();
            let name = match class_at(m.start()) {
                Some(c) => format!("{c}.{}", &cap[1]),
                None => cap[1].to_string(),
            };
            out.push(Binding {
                name,
                kind: BindingKind::ClassField,
                source_file: file_label.clone(),
                source_line: line_at(m.start()),
            });
        }
        for cap in re_add_field_set.captures_iter(&content) {
            let m = cap.get(0).unwrap();
            let name = match class_at(m.start()) {
                Some(c) => format!("{c}.{}", &cap[1]),
                None => cap[1].to_string(),
            };
            out.push(Binding {
                name,
                kind: BindingKind::ClassField,
                source_file: file_label.clone(),
                source_line: line_at(m.start()),
            });
        }
        // Manual `// AUDIT: <Kind> <name1> <name2> …` markers.
        for cap in re_audit_comment.captures_iter(&content) {
            let m = cap.get(0).unwrap();
            let kind = match &cap[1] {
                "ClassMethod" => BindingKind::ClassMethod,
                "ClassField" => BindingKind::ClassField,
                "Class" => BindingKind::Class,
                "GlobalEnum" => BindingKind::GlobalEnum,
                "GlobalFunction" => BindingKind::GlobalFunction,
                "Table" => BindingKind::Table,
                "TableEnum" => BindingKind::TableEnum,
                _ => continue,
            };
            for name in cap[2].split_whitespace() {
                out.push(Binding {
                    name: name.to_string(),
                    kind,
                    source_file: file_label.clone(),
                    source_line: line_at(m.start()),
                });
            }
        }

        for cap in re_add_meta_method.captures_iter(&content) {
            let m = cap.get(0).unwrap();
            // Translate mlua MetaMethod variant → Lua `__name`.
            let lua_name = match &cap[1] {
                "Eq" => "__eq",
                "Gc" => "__gc",
                "Index" => "__index",
                "NewIndex" => "__newindex",
                "ToString" => "__tostring",
                "Len" => "__len",
                "Call" => "__call",
                "Add" => "__add",
                "Sub" => "__sub",
                "Mul" => "__mul",
                "Div" => "__div",
                "Mod" => "__mod",
                "Pow" => "__pow",
                "Unm" => "__unm",
                "Concat" => "__concat",
                "Lt" => "__lt",
                "Le" => "__le",
                other => other,
            };
            let name = match class_at(m.start()) {
                Some(c) => format!("{c}:{}", lua_name),
                None => lua_name.to_string(),
            };
            out.push(Binding {
                name,
                kind: BindingKind::ClassMethod,
                source_file: file_label.clone(),
                source_line: line_at(m.start()),
            });
        }
    }
    // De-duplicate: when the same name is registered as both
    // `GlobalFunction:X` AND `Class:X` (or `Table:X`), drop the
    // GlobalFunction record. C++ only uses one slot per identifier —
    // typically `Class` for instance types (Position, Item, …) and
    // `Table` for singletons (Game, configKeys). The double-emission
    // happens on the Rust side because we register class/table
    // constructors via `lua.globals().set("Name", …)`, which the
    // parser fires `GlobalFunction` on.
    let suppressed_names: BTreeSet<String> = out
        .iter()
        .filter_map(|b| match b.kind {
            BindingKind::Class | BindingKind::Table => Some(b.name.clone()),
            _ => None,
        })
        .collect();
    out.retain(|b| !(b.kind == BindingKind::GlobalFunction && suppressed_names.contains(&b.name)));
    Ok(out)
}

/// Remove every `#[cfg(test)] mod <name> { … }` block from a Rust source
/// file. The audit only cares about production bindings; test helpers
/// that call `lua.globals().set("x", …)` would otherwise show up as
/// spurious GlobalFunction bindings.
fn strip_test_modules(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut out = String::with_capacity(content.len());
    let mut cursor = 0;
    while let Some(start_rel) = content[cursor..].find("#[cfg(test)]") {
        let start = cursor + start_rel;
        out.push_str(&content[cursor..start]);
        // Skip the attribute, whitespace, and find the next `{` that opens
        // the module body. If we can't find one (malformed), bail.
        let mod_open = match content[start..].find('{') {
            Some(o) => start + o,
            None => break,
        };
        let mut depth: i32 = 1;
        let mut i = mod_open + 1;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        cursor = i;
    }
    out.push_str(&content[cursor..]);
    out
}

/// Rewrite Rust PascalCase acronym prefixes back to all-caps so the
/// audit names line up with the C++ side. Currently handles `Db` → `DB`
/// and `Xml` → `XML`; extend the list when other acronyms come up.
fn re_uppercase_acronyms(name: &str) -> String {
    const PAIRS: &[(&str, &str)] = &[("Db", "DB"), ("Xml", "XML")];
    let mut out = name.to_string();
    for (rs, cpp) in PAIRS {
        if let Some(rest) = out.strip_prefix(rs) {
            out = format!("{cpp}{rest}");
        }
    }
    out
}

/// Locate every `impl UserData for X { … }` block and return
/// `(start_byte, end_byte_exclusive, class_name)` ranges. Class name
/// has its `Lua` prefix stripped per workspace convention. The end
/// byte is computed by brace-counting from the opener's `{`.
fn find_impl_ranges(content: &str, re: &Regex) -> Vec<(usize, usize, String)> {
    let bytes = content.as_bytes();
    let mut ranges = Vec::new();
    for cap in re.captures_iter(content) {
        let m = cap.get(0).unwrap();
        let raw = cap[1].to_string();
        let stripped = raw.strip_prefix("Lua").unwrap_or(&raw);
        // Re-uppercase common acronyms that Rust's PascalCase convention
        // forces to be camel-cased (`Db`, `Xml`) but the C++ side keeps
        // as all-caps (`DB`, `XML`). This keeps the Rust newtype names
        // idiomatic while still matching the audit's C++ binding names.
        let class = re_uppercase_acronyms(stripped);
        // m.end() points just past the opener — find the position of
        // the actual `{` (which is the last char of the opener).
        let mut open = m.end();
        while open > 0 && bytes[open - 1] != b'{' {
            open -= 1;
        }
        let mut depth: i32 = 1;
        let mut i = open;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        ranges.push((m.start(), i, class));
    }
    ranges
}

// ── Diff ─────────────────────────────────────────────────────────────────────

/// Intentional Rust-side bindings that have no C++ counterpart but are
/// genuinely useful for Lua scripts using the Rust server. Each entry is
/// (Kind, name). The audit ignores these when computing the unexpected
/// set so they don't keep the report at FAIL forever. Add an
/// `intentional_differences.yml` note for every entry added here.
const INTENTIONAL_EXTRAS: &[(BindingKind, &str)] = &[
    // mlua's `add_field_method_get/set` is the Rust-idiomatic way to
    // expose userdata fields. C++ uses `registerMetaMethod` with
    // `__index`/`__newindex` instead, which doesn't surface a per-field
    // entry in the audit — so the Position fields are Rust-only.
    (BindingKind::ClassField, "Position.x"),
    (BindingKind::ClassField, "Position.y"),
    (BindingKind::ClassField, "Position.z"),
    // Convenience `tostring(Position(…))` for debug prints. Matches the
    // C++ behaviour but C++ doesn't register a Lua `__tostring`.
    (BindingKind::ClassMethod, "Position:__tostring"),
    // `Game` is registered both as a global UserData (yielding Class:Game
    // via the parser's `impl UserData` auto-emit) and as a Table (via the
    // value-detection heuristic). C++ only exposes it as Table. Keeping
    // the Class binding lets `Game:method` calls type-check on the Rust
    // side without breaking the audit.
    (BindingKind::Class, "Game"),
];

/// Build the audit report from C++ + Rust binding lists.
pub fn build_report(cpp: &[Binding], rust: &[Binding]) -> AuditReport {
    // Dedupe by (kind, name) — registerFunctions may be called from
    // multiple includes / macro expansions.
    let cpp_set: BTreeSet<(BindingKind, String)> = cpp.iter().map(|b| b.key()).collect();
    let rust_set: BTreeSet<(BindingKind, String)> = rust.iter().map(|b| b.key()).collect();
    let intentional: BTreeSet<(BindingKind, String)> = INTENTIONAL_EXTRAS
        .iter()
        .map(|(k, n)| (*k, n.to_string()))
        .collect();

    let missing_in_rust: Vec<String> = cpp_set
        .difference(&rust_set)
        .map(|(k, n)| format!("[{:?}] {n}", k))
        .collect();
    let unexpected_in_rust: Vec<String> = rust_set
        .difference(&cpp_set)
        .filter(|key| !intentional.contains(*key))
        .map(|(k, n)| format!("[{:?}] {n}", k))
        .collect();

    let mut by_kind_cpp: BTreeMap<String, usize> = BTreeMap::new();
    for (k, _) in &cpp_set {
        *by_kind_cpp.entry(format!("{k:?}")).or_default() += 1;
    }
    let mut by_kind_rust: BTreeMap<String, usize> = BTreeMap::new();
    for (k, _) in &rust_set {
        *by_kind_rust.entry(format!("{k:?}")).or_default() += 1;
    }

    let status = if missing_in_rust.is_empty() && unexpected_in_rust.is_empty() {
        "PASS"
    } else {
        "FAIL"
    }
    .to_string();

    let mut ledger_entries = Vec::new();
    if !missing_in_rust.is_empty() {
        ledger_entries.push(LedgerEntry {
            cpp: "src/luascript.cpp".to_string(),
            transition: "DONE → PARTIAL".to_string(),
            reason: format!(
                "{} of {} Lua bindings missing in Rust (static audit)",
                missing_in_rust.len(),
                cpp_set.len()
            ),
        });
    }

    AuditReport {
        lane: "lua_bindings".to_string(),
        sub_lane: "static".to_string(),
        status,
        cpp_total: cpp_set.len(),
        rust_total: rust_set.len(),
        missing_in_rust,
        unexpected_in_rust,
        by_kind_cpp,
        by_kind_rust,
        ledger_entries,
    }
}

/// Convenience: run the full audit and return the JSON-serializable report.
pub fn audit(cpp_path: &Path, rust_root: &Path) -> anyhow::Result<AuditReport> {
    let cpp = parse_cpp_bindings(cpp_path)?;
    let rust = parse_rust_bindings(rust_root)?;
    Ok(build_report(&cpp, &rust))
}

// Workaround: harness-tools needs PathBuf in its tests
#[allow(dead_code)]
fn _enforce_pathbuf_in_use() -> PathBuf {
    PathBuf::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmpfile(name: &str, content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "harness-tools-lua-audit-{:?}-{}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    fn tmpdir_with(files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "harness-tools-rust-audit-{:?}-{}",
            std::thread::current().id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for (name, content) in files {
            let path = dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, content).unwrap();
        }
        dir
    }

    // ── C++ parser ────────────────────────────────────────────────────────────

    #[test]
    fn cpp_parser_extracts_register_method() {
        let path = tmpfile(
            "luascript.cpp",
            r#"registerMethod(L, "Player", "getLevel", luaPlayerGetLevel);
registerMethod(L, "Player", "getHealth", luaPlayerGetHealth);"#,
        );
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].name, "Player:getLevel");
        assert_eq!(bindings[0].kind, BindingKind::ClassMethod);
        assert_eq!(bindings[1].name, "Player:getHealth");
    }

    #[test]
    fn cpp_parser_extracts_register_global_method() {
        let path = tmpfile(
            "luascript.cpp",
            r#"registerGlobalMethod(L, "getCreatureName", luaGetCreatureName);"#,
        );
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "getCreatureName");
        assert_eq!(bindings[0].kind, BindingKind::GlobalFunction);
    }

    #[test]
    fn cpp_parser_extracts_register_class_with_base() {
        let path = tmpfile(
            "luascript.cpp",
            r#"registerClass(L, "Player", "Creature", luaPlayerCreate);"#,
        );
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "Player");
        assert_eq!(bindings[0].kind, BindingKind::Class);
    }

    #[test]
    fn cpp_parser_extracts_register_class_with_empty_base() {
        let path = tmpfile(
            "luascript.cpp",
            r#"registerClass(L, "Game", "", luaGameCreate);"#,
        );
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "Game");
    }

    #[test]
    fn cpp_parser_extracts_register_table() {
        let path = tmpfile("luascript.cpp", r#"registerTable(L, "Game");"#);
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "Game");
        assert_eq!(bindings[0].kind, BindingKind::Table);
    }

    #[test]
    fn cpp_parser_extracts_register_variable() {
        let path = tmpfile(
            "luascript.cpp",
            r#"registerVariable(L, "MOVEEVENT", "STEP_IN", MOVE_EVENT_STEP_IN);"#,
        );
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "MOVEEVENT.STEP_IN");
        assert_eq!(bindings[0].kind, BindingKind::TableVariable);
    }

    #[test]
    fn cpp_parser_extracts_register_enum() {
        let path = tmpfile("luascript.cpp", r#"registerEnum(L, ACCOUNT_TYPE_NORMAL);"#);
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "ACCOUNT_TYPE_NORMAL");
        assert_eq!(bindings[0].kind, BindingKind::GlobalEnum);
    }

    #[test]
    fn cpp_parser_extracts_register_enum_in() {
        let path = tmpfile("luascript.cpp", r#"registerEnumIn(L, "configKeys", IP);"#);
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "configKeys.IP");
        assert_eq!(bindings[0].kind, BindingKind::TableEnum);
    }

    #[test]
    fn cpp_parser_skips_comment_lines() {
        let path = tmpfile(
            "luascript.cpp",
            r#"// registerMethod(L, "Player", "getLevel", luaPlayerGetLevel);
registerMethod(L, "Player", "getHealth", luaPlayerGetHealth);"#,
        );
        let bindings = parse_cpp_bindings(&path).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "Player:getHealth");
    }

    #[test]
    fn cpp_parser_skips_define_macros() {
        let path = tmpfile(
            "luascript.cpp",
            r#"#define registerEnum(L, value) \
    registerEnumIn(L, "global", value)
registerEnum(L, ACCOUNT_TYPE_NORMAL);"#,
        );
        let bindings = parse_cpp_bindings(&path).unwrap();
        // The #define line is skipped; only the real call is captured.
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "ACCOUNT_TYPE_NORMAL");
    }

    // ── Rust parser ───────────────────────────────────────────────────────────

    #[test]
    fn rust_parser_extracts_globals_set_function() {
        let dir = tmpdir_with(&[(
            "lib.rs",
            r#"fn register(lua: &mlua::Lua) {
    lua.globals().set("getCreatureName", lua.create_function(|_, ()| Ok(())).unwrap());
}"#,
        )]);
        let bindings = parse_rust_bindings(&dir).unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].name, "getCreatureName");
        assert_eq!(bindings[0].kind, BindingKind::GlobalFunction);
    }

    #[test]
    fn rust_parser_extracts_add_method_with_class_context() {
        let dir = tmpdir_with(&[(
            "player.rs",
            r#"impl mlua::UserData for Player {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("getLevel", |_, this, ()| Ok(this.level));
        methods.add_method("getHealth", |_, this, ()| Ok(this.health));
    }
}"#,
        )]);
        let bindings = parse_rust_bindings(&dir).unwrap();
        // Class binding + 2 explicit ClassMethods.
        assert_eq!(bindings.len(), 3);
        assert!(bindings
            .iter()
            .any(|b| b.kind == BindingKind::Class && b.name == "Player"));
        let methods: Vec<&str> = bindings
            .iter()
            .filter(|b| b.kind == BindingKind::ClassMethod)
            .map(|b| b.name.as_str())
            .collect();
        assert!(methods.contains(&"Player:getLevel"));
        assert!(methods.contains(&"Player:getHealth"));
    }

    #[test]
    fn rust_parser_extracts_field_methods() {
        let dir = tmpdir_with(&[(
            "item.rs",
            r#"impl UserData for Item {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("count", |_, this| Ok(this.count));
        fields.add_field_method_set("count", |_, this, v| { this.count = v; Ok(()) });
    }
}"#,
        )]);
        let bindings = parse_rust_bindings(&dir).unwrap();
        // Class binding + 2 ClassField (get + set, same name).
        assert_eq!(bindings.len(), 3);
        assert!(bindings
            .iter()
            .any(|b| b.kind == BindingKind::Class && b.name == "Item"));
        let fields: Vec<&str> = bindings
            .iter()
            .filter(|b| b.kind == BindingKind::ClassField)
            .map(|b| b.name.as_str())
            .collect();
        assert!(fields.iter().all(|n| *n == "Item.count"));
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn rust_parser_strips_lua_prefix_from_newtype_class_name() {
        let dir = tmpdir_with(&[(
            "position.rs",
            r#"impl mlua::UserData for LuaPosition {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("isSightClear", |_, _, ()| Ok(true));
    }
}"#,
        )]);
        let bindings = parse_rust_bindings(&dir).unwrap();
        // Class binding (stripped) + 1 ClassMethod (stripped).
        assert_eq!(bindings.len(), 2);
        assert!(bindings
            .iter()
            .any(|b| b.kind == BindingKind::Class && b.name == "Position"));
        assert!(bindings
            .iter()
            .any(|b| b.kind == BindingKind::ClassMethod && b.name == "Position:isSightClear"));
    }

    #[test]
    fn rust_parser_handles_zero_bindings_directory() {
        let dir = tmpdir_with(&[("stub.rs", r#"pub struct Foo { x: i32 }"#)]);
        let bindings = parse_rust_bindings(&dir).unwrap();
        assert!(bindings.is_empty());
    }

    // ── Diff / report ─────────────────────────────────────────────────────────

    #[test]
    fn build_report_pass_when_surfaces_match() {
        let cpp = vec![Binding {
            name: "Player:getLevel".into(),
            kind: BindingKind::ClassMethod,
            source_file: "luascript.cpp".into(),
            source_line: 1,
        }];
        let rust = vec![Binding {
            name: "Player:getLevel".into(),
            kind: BindingKind::ClassMethod,
            source_file: "player.rs".into(),
            source_line: 2,
        }];
        let r = build_report(&cpp, &rust);
        assert_eq!(r.status, "PASS");
        assert!(r.missing_in_rust.is_empty());
        assert!(r.ledger_entries.is_empty());
    }

    #[test]
    fn build_report_fail_when_rust_missing_binding() {
        let cpp = vec![Binding {
            name: "Player:getLevel".into(),
            kind: BindingKind::ClassMethod,
            source_file: "luascript.cpp".into(),
            source_line: 1,
        }];
        let rust = vec![];
        let r = build_report(&cpp, &rust);
        assert_eq!(r.status, "FAIL");
        assert_eq!(r.missing_in_rust.len(), 1);
        assert_eq!(r.ledger_entries.len(), 1);
        assert_eq!(r.ledger_entries[0].cpp, "src/luascript.cpp");
        assert_eq!(r.ledger_entries[0].transition, "DONE → PARTIAL");
    }

    #[test]
    fn build_report_reports_unexpected_rust_bindings() {
        let cpp = vec![];
        let rust = vec![Binding {
            name: "Player:newRustOnly".into(),
            kind: BindingKind::ClassMethod,
            source_file: "player.rs".into(),
            source_line: 1,
        }];
        let r = build_report(&cpp, &rust);
        assert_eq!(r.status, "FAIL");
        assert_eq!(r.unexpected_in_rust.len(), 1);
    }

    #[test]
    fn build_report_dedupes_repeated_cpp_bindings() {
        let cpp = vec![
            Binding {
                name: "Player:getLevel".into(),
                kind: BindingKind::ClassMethod,
                source_file: "luascript.cpp".into(),
                source_line: 1,
            },
            Binding {
                name: "Player:getLevel".into(),
                kind: BindingKind::ClassMethod,
                source_file: "luascript.cpp".into(),
                source_line: 5,
            },
        ];
        let r = build_report(&cpp, &[]);
        assert_eq!(r.cpp_total, 1);
        assert_eq!(r.missing_in_rust.len(), 1);
    }

    #[test]
    fn build_report_by_kind_counts_each_kind_separately() {
        let cpp = vec![
            Binding {
                name: "Player:getLevel".into(),
                kind: BindingKind::ClassMethod,
                source_file: "x".into(),
                source_line: 1,
            },
            Binding {
                name: "MAX_LEVEL".into(),
                kind: BindingKind::GlobalEnum,
                source_file: "x".into(),
                source_line: 2,
            },
        ];
        let r = build_report(&cpp, &[]);
        assert_eq!(r.by_kind_cpp.get("ClassMethod"), Some(&1));
        assert_eq!(r.by_kind_cpp.get("GlobalEnum"), Some(&1));
    }

    #[test]
    fn build_report_status_pass_only_when_both_sides_empty_diff() {
        let r = build_report(&[], &[]);
        assert_eq!(r.status, "PASS");
        assert_eq!(r.cpp_total, 0);
        assert_eq!(r.rust_total, 0);
    }
}
