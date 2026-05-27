use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use quote::ToTokens;
use serde::Serialize;
use sha2::{Digest, Sha256};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Attribute, Expr, Fields, ImplItem, Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro,
    ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, Meta, ReturnType, Signature, TraitItem,
    Type, Variant, Visibility,
};
use walkdir::WalkDir;

// ───────────────────────────────────────────────────────────────────────────────
// Output schema
// ───────────────────────────────────────────────────────────────────────────────

#[derive(Serialize, Default, Clone)]
struct Symbol {
    file: String,
    kind: String,
    qualified_name: String,
    signature: String,
    visibility: String,
    line_start: usize,
    line_end: usize,
    dependencies: Vec<String>,
    behavior_summary: String,
    risk_level: String,
    body_hash: String,
    notes: Vec<String>,
}

// ───────────────────────────────────────────────────────────────────────────────
// Entry point
// ───────────────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let crates_dir = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "forgottenserver-rust/crates".to_string());
    let out_path = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "rust_symbol_manifest.json".to_string());

    let root = PathBuf::from(&crates_dir).canonicalize().unwrap_or_else(|_| PathBuf::from(&crates_dir));
    let workspace_root = root
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.clone());

    let mut all: Vec<Symbol> = Vec::new();
    let mut files_processed = 0usize;
    let mut files_failed = 0usize;

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("rs"))
        .filter(|e| !e.path().components().any(|c| c.as_os_str() == "target"))
    {
        let path = entry.path();
        let rel_to_workspace = path
            .strip_prefix(&workspace_root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let src = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("read fail {}: {}", path.display(), e);
                files_failed += 1;
                continue;
            }
        };
        let file_ast = match syn::parse_file(&src) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("parse fail {}: {}", path.display(), e);
                files_failed += 1;
                continue;
            }
        };
        files_processed += 1;

        let lines: Vec<&str> = src.lines().collect();
        let mod_path = module_path_from_file(path, &root);

        // File-level module record.
        let file_doc = extract_doc(&file_ast.attrs);
        all.push(Symbol {
            file: rel_to_workspace.clone(),
            kind: "module".to_string(),
            qualified_name: mod_path.clone(),
            signature: format!("mod {}", mod_path),
            visibility: "pub".to_string(),
            line_start: 1,
            line_end: lines.len().max(1),
            dependencies: collect_use_paths(&file_ast.items),
            behavior_summary: first_line(&file_doc).unwrap_or_else(|| format!(
                "Top-level module `{}` rooted at this file.", mod_path
            )),
            risk_level: risk_for_module(&rel_to_workspace),
            body_hash: sha_hex(src.as_bytes()),
            notes: maybe_uncertain(&[]),
        });

        let mut ctx = Ctx {
            file: rel_to_workspace.clone(),
            src: &src,
            lines: &lines,
            mod_path: mod_path.clone(),
            out: Vec::new(),
        };
        walk_items(&file_ast.items, &mut ctx);
        all.extend(ctx.out);
    }

    let json = serde_json::to_vec_pretty(&all).expect("serialize manifest");
    fs::write(&out_path, &json).expect("write manifest");
    eprintln!(
        "manifest_extractor: processed {} files ({} failed) → {} symbols → {}",
        files_processed,
        files_failed,
        all.len(),
        out_path
    );
}

// ───────────────────────────────────────────────────────────────────────────────
// Walk context
// ───────────────────────────────────────────────────────────────────────────────

struct Ctx<'a> {
    file: String,
    src: &'a str,
    lines: &'a [&'a str],
    mod_path: String,
    out: Vec<Symbol>,
}

fn walk_items(items: &[Item], ctx: &mut Ctx) {
    for item in items {
        emit_item(item, ctx);
    }
}

fn emit_item(item: &Item, ctx: &mut Ctx) {
    match item {
        Item::Mod(m) => emit_mod(m, ctx),
        Item::Struct(s) => emit_struct(s, ctx),
        Item::Enum(e) => emit_enum(e, ctx),
        Item::Trait(t) => emit_trait(t, ctx),
        Item::Impl(i) => emit_impl(i, ctx),
        Item::Fn(f) => emit_fn(f, ctx),
        Item::Const(c) => emit_const(c, ctx),
        Item::Static(s) => emit_static(s, ctx),
        Item::Type(t) => emit_type_alias(t, ctx),
        Item::Macro(m) => emit_macro(m, ctx),
        Item::Union(u) => emit_union(u, ctx),
        _ => {}
    }
}

// ───────────────────────────────────────────────────────────────────────────────
// Item emitters
// ───────────────────────────────────────────────────────────────────────────────

fn emit_mod(m: &ItemMod, ctx: &mut Ctx) {
    let name = m.ident.to_string();
    let (ls, le) = span_lines(m.span());
    let qn = join_qn(&ctx.mod_path, &name);
    let body_text = slice_lines(ctx.lines, ls, le);

    let signature = format!("{} mod {}", vis_str(&m.vis), name);
    let doc = extract_doc(&m.attrs);
    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: "module".to_string(),
        qualified_name: qn.clone(),
        signature,
        visibility: vis_str(&m.vis),
        line_start: ls,
        line_end: le,
        dependencies: vec![],
        behavior_summary: first_line(&doc).unwrap_or_else(|| format!("Inline module `{}`.", name)),
        risk_level: risk_for_module(&ctx.file),
        body_hash: sha_hex(body_text.as_bytes()),
        notes: if m.content.is_none() {
            vec![format!("declares external module `{}`; body lives in a separate file", name)]
        } else {
            vec![]
        },
    });

    if let Some((_, items)) = &m.content {
        let saved = ctx.mod_path.clone();
        ctx.mod_path = qn;
        walk_items(items, ctx);
        ctx.mod_path = saved;
    }
}

fn emit_struct(s: &ItemStruct, ctx: &mut Ctx) {
    let name = s.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(s.span());
    let body_text = slice_lines(ctx.lines, ls, le);
    let signature = compact_tokens(&s.to_token_stream(), 240);
    let deps = collect_deps_in_tokens(&s.to_token_stream());

    let derives = collect_derives(&s.attrs);
    let mut notes = vec![];
    if !derives.is_empty() {
        notes.push(format!("derives: {}", derives.join(", ")));
    }
    if is_serde_serializable(&s.attrs) {
        notes.push("serde-serializable".to_string());
    }
    if is_sea_orm_entity(&s.attrs) {
        notes.push("sea-orm entity/model".to_string());
    }

    let kind = if is_error_type(&name, &derives) {
        "error"
    } else if is_sea_orm_entity(&s.attrs) {
        "database_mapping"
    } else {
        "struct"
    }
    .to_string();

    let doc = extract_doc(&s.attrs);
    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: kind.clone(),
        qualified_name: qn.clone(),
        signature,
        visibility: vis_str(&s.vis),
        line_start: ls,
        line_end: le,
        dependencies: deps,
        behavior_summary: first_line(&doc).unwrap_or_else(|| describe_struct(&name, &s.fields)),
        risk_level: risk_for_type(&name, &ctx.file, &kind),
        body_hash: sha_hex(body_text.as_bytes()),
        notes,
    });

    // Fields
    for (idx, field) in s.fields.iter().enumerate() {
        let fname = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| format!("{idx}"));
        let fqn = format!("{qn}.{fname}");
        let (fls, fle) = span_lines(field.span());
        let ftext = slice_lines(ctx.lines, fls, fle);
        let fsig = compact_tokens(&field.to_token_stream(), 200);
        let fdoc = extract_doc(&field.attrs);
        ctx.out.push(Symbol {
            file: ctx.file.clone(),
            kind: "field".to_string(),
            qualified_name: fqn,
            signature: fsig,
            visibility: vis_str(&field.vis),
            line_start: fls,
            line_end: fle,
            dependencies: collect_deps_in_tokens(&field.ty.to_token_stream()),
            behavior_summary: first_line(&fdoc).unwrap_or_else(|| {
                format!("Field `{}` of `{}`.", fname, name)
            }),
            risk_level: "low".to_string(),
            body_hash: sha_hex(ftext.as_bytes()),
            notes: vec![],
        });
    }
}

fn emit_enum(e: &ItemEnum, ctx: &mut Ctx) {
    let name = e.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(e.span());
    let body_text = slice_lines(ctx.lines, ls, le);
    let signature = compact_tokens(&e.to_token_stream(), 240);
    let derives = collect_derives(&e.attrs);
    let mut notes = vec![];
    if !derives.is_empty() {
        notes.push(format!("derives: {}", derives.join(", ")));
    }
    let kind = if is_error_type(&name, &derives) {
        "error"
    } else if is_opcode_enum(&name, &ctx.file) {
        "protocol_opcode_enum"
    } else {
        "enum"
    }
    .to_string();
    let doc = extract_doc(&e.attrs);

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: kind.clone(),
        qualified_name: qn.clone(),
        signature,
        visibility: vis_str(&e.vis),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&e.to_token_stream()),
        behavior_summary: first_line(&doc)
            .unwrap_or_else(|| format!("Enumeration `{}` with {} variants.", name, e.variants.len())),
        risk_level: risk_for_type(&name, &ctx.file, &kind),
        body_hash: sha_hex(body_text.as_bytes()),
        notes,
    });

    for variant in &e.variants {
        emit_enum_variant(&qn, &name, variant, ctx);
    }
}

fn emit_enum_variant(parent_qn: &str, parent_name: &str, v: &Variant, ctx: &mut Ctx) {
    let vname = v.ident.to_string();
    let qn = format!("{parent_qn}::{vname}");
    let (ls, le) = span_lines(v.span());
    let text = slice_lines(ctx.lines, ls, le);
    let sig = compact_tokens(&v.to_token_stream(), 200);
    let vdoc = extract_doc(&v.attrs);

    let mut notes = vec![];
    let mut kind = "enum_variant".to_string();
    if let Some((_, disc)) = &v.discriminant {
        let disc_text = compact_tokens(&disc.to_token_stream(), 80);
        notes.push(format!("discriminant: {}", disc_text));
        if is_opcode_context(parent_name, &ctx.file) {
            kind = "protocol_opcode".to_string();
        }
    }

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind,
        qualified_name: qn,
        signature: sig,
        visibility: "pub".to_string(),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&v.to_token_stream()),
        behavior_summary: first_line(&vdoc).unwrap_or_else(|| {
            format!("Variant `{}` of enum `{}`.", vname, parent_name)
        }),
        risk_level: if notes.iter().any(|n| n.contains("discriminant")) {
            "medium".to_string()
        } else {
            "low".to_string()
        },
        body_hash: sha_hex(text.as_bytes()),
        notes,
    });
}

fn emit_trait(t: &ItemTrait, ctx: &mut Ctx) {
    let name = t.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(t.span());
    let body_text = slice_lines(ctx.lines, ls, le);
    let sig = compact_tokens(&t.to_token_stream(), 240);
    let doc = extract_doc(&t.attrs);

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: "trait".to_string(),
        qualified_name: qn.clone(),
        signature: sig,
        visibility: vis_str(&t.vis),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&t.to_token_stream()),
        behavior_summary: first_line(&doc).unwrap_or_else(|| {
            format!("Trait `{}` with {} associated items.", name, t.items.len())
        }),
        risk_level: risk_for_type(&name, &ctx.file, "trait"),
        body_hash: sha_hex(body_text.as_bytes()),
        notes: vec![],
    });

    for item in &t.items {
        match item {
            TraitItem::Fn(f) => {
                let mname = f.sig.ident.to_string();
                let mqn = format!("{qn}::{mname}");
                let (mls, mle) = span_lines(f.span());
                let text = slice_lines(ctx.lines, mls, mle);
                let signature = sig_to_string(&f.sig);
                let mdoc = extract_doc(&f.attrs);
                let provided = f.default.is_some();
                let mut notes = vec!["trait method".to_string()];
                if provided {
                    notes.push("default-provided".to_string());
                }
                ctx.out.push(Symbol {
                    file: ctx.file.clone(),
                    kind: "trait_method".to_string(),
                    qualified_name: mqn,
                    signature,
                    visibility: "pub".to_string(),
                    line_start: mls,
                    line_end: mle,
                    dependencies: collect_deps_in_tokens(&f.to_token_stream()),
                    behavior_summary: first_line(&mdoc).unwrap_or_else(|| {
                        format!("Trait method `{}::{}`.", name, mname)
                    }),
                    risk_level: risk_for_fn(&mname, &ctx.file, &text),
                    body_hash: sha_hex(text.as_bytes()),
                    notes,
                });
            }
            TraitItem::Const(c) => {
                let cname = c.ident.to_string();
                let cqn = format!("{qn}::{cname}");
                let (cls, cle) = span_lines(c.span());
                let text = slice_lines(ctx.lines, cls, cle);
                let signature = compact_tokens(&c.to_token_stream(), 200);
                ctx.out.push(Symbol {
                    file: ctx.file.clone(),
                    kind: "trait_const".to_string(),
                    qualified_name: cqn,
                    signature,
                    visibility: "pub".to_string(),
                    line_start: cls,
                    line_end: cle,
                    dependencies: collect_deps_in_tokens(&c.ty.to_token_stream()),
                    behavior_summary: first_line(&extract_doc(&c.attrs))
                        .unwrap_or_else(|| format!("Associated constant `{}` on trait `{}`.", cname, name)),
                    risk_level: "low".to_string(),
                    body_hash: sha_hex(text.as_bytes()),
                    notes: vec!["trait const".to_string()],
                });
            }
            TraitItem::Type(ty) => {
                let tname = ty.ident.to_string();
                let tqn = format!("{qn}::{tname}");
                let (tls, tle) = span_lines(ty.span());
                let text = slice_lines(ctx.lines, tls, tle);
                ctx.out.push(Symbol {
                    file: ctx.file.clone(),
                    kind: "trait_type".to_string(),
                    qualified_name: tqn,
                    signature: compact_tokens(&ty.to_token_stream(), 200),
                    visibility: "pub".to_string(),
                    line_start: tls,
                    line_end: tle,
                    dependencies: vec![],
                    behavior_summary: first_line(&extract_doc(&ty.attrs))
                        .unwrap_or_else(|| format!("Associated type `{}` on trait `{}`.", tname, name)),
                    risk_level: "low".to_string(),
                    body_hash: sha_hex(text.as_bytes()),
                    notes: vec!["associated type".to_string()],
                });
            }
            _ => {}
        }
    }
}

fn emit_impl(i: &ItemImpl, ctx: &mut Ctx) {
    let self_ty = compact_tokens(&i.self_ty.to_token_stream(), 120);
    let self_name = type_root_name(&i.self_ty);
    let trait_path = i
        .trait_
        .as_ref()
        .map(|(_, p, _)| compact_tokens(&p.to_token_stream(), 120));
    let parent_qn = join_qn(&ctx.mod_path, &self_name);
    let (impl_ls, impl_le) = span_lines(i.span());
    let impl_text = slice_lines(ctx.lines, impl_ls, impl_le);

    let impl_signature = match &trait_path {
        Some(tp) => format!("impl {} for {}", tp, self_ty),
        None => format!("impl {}", self_ty),
    };
    let mut impl_notes = vec![];
    if let Some(tp) = &trait_path {
        impl_notes.push(format!("trait impl: {}", tp));
    }
    if !i.generics.params.is_empty() {
        impl_notes.push(format!(
            "generics: {}",
            compact_tokens(&i.generics.to_token_stream(), 120)
        ));
    }
    let impl_qn = match &trait_path {
        Some(tp) => format!("{parent_qn}::<impl {} for {}>", tp, self_ty),
        None => format!("{parent_qn}::<impl>"),
    };

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: "impl_block".to_string(),
        qualified_name: impl_qn.clone(),
        signature: impl_signature,
        visibility: "pub".to_string(),
        line_start: impl_ls,
        line_end: impl_le,
        dependencies: collect_deps_in_tokens(&i.to_token_stream()),
        behavior_summary: trait_path
            .as_ref()
            .map(|tp| format!("Implements trait `{}` for `{}`.", tp, self_ty))
            .unwrap_or_else(|| format!("Inherent impl block for `{}`.", self_ty)),
        risk_level: risk_for_type(&self_name, &ctx.file, "impl"),
        body_hash: sha_hex(impl_text.as_bytes()),
        notes: impl_notes,
    });

    for ii in &i.items {
        match ii {
            ImplItem::Fn(f) => {
                let mname = f.sig.ident.to_string();
                let (mls, mle) = span_lines(f.span());
                let text = slice_lines(ctx.lines, mls, mle);
                let signature = sig_to_string(&f.sig);
                let mdoc = extract_doc(&f.attrs);
                let is_assoc = !has_self_receiver(&f.sig);
                let kind = if trait_path.is_some() {
                    "trait_impl_method"
                } else if is_assoc {
                    "associated_function"
                } else {
                    "method"
                }
                .to_string();
                let mut notes = vec![];
                if let Some(tp) = &trait_path {
                    notes.push(format!("implements {}::{}", tp, mname));
                }
                if is_lua_binding(&ctx.file, &text) {
                    notes.push("lua binding (heuristic)".to_string());
                }
                if is_scheduler_event(&ctx.file, &mname, &text) {
                    notes.push("scheduler event (heuristic)".to_string());
                }

                let mqn = format!("{impl_qn}::{mname}");
                ctx.out.push(Symbol {
                    file: ctx.file.clone(),
                    kind,
                    qualified_name: mqn,
                    signature,
                    visibility: vis_str(&f.vis),
                    line_start: mls,
                    line_end: mle,
                    dependencies: collect_deps_in_tokens(&f.to_token_stream()),
                    behavior_summary: first_line(&mdoc).unwrap_or_else(|| {
                        describe_method(&mname, &self_ty, trait_path.as_deref())
                    }),
                    risk_level: risk_for_fn(&mname, &ctx.file, &text),
                    body_hash: sha_hex(text.as_bytes()),
                    notes,
                });
            }
            ImplItem::Const(c) => {
                let cname = c.ident.to_string();
                let (cls, cle) = span_lines(c.span());
                let text = slice_lines(ctx.lines, cls, cle);
                let cqn = format!("{impl_qn}::{cname}");
                ctx.out.push(Symbol {
                    file: ctx.file.clone(),
                    kind: "associated_const".to_string(),
                    qualified_name: cqn,
                    signature: compact_tokens(&c.to_token_stream(), 200),
                    visibility: vis_str(&c.vis),
                    line_start: cls,
                    line_end: cle,
                    dependencies: collect_deps_in_tokens(&c.ty.to_token_stream()),
                    behavior_summary: first_line(&extract_doc(&c.attrs)).unwrap_or_else(|| {
                        format!("Associated constant `{}` on `{}`.", cname, self_ty)
                    }),
                    risk_level: "low".to_string(),
                    body_hash: sha_hex(text.as_bytes()),
                    notes: vec![],
                });
            }
            ImplItem::Type(ty) => {
                let tname = ty.ident.to_string();
                let (tls, tle) = span_lines(ty.span());
                let text = slice_lines(ctx.lines, tls, tle);
                let tqn = format!("{impl_qn}::{tname}");
                ctx.out.push(Symbol {
                    file: ctx.file.clone(),
                    kind: "associated_type".to_string(),
                    qualified_name: tqn,
                    signature: compact_tokens(&ty.to_token_stream(), 200),
                    visibility: vis_str(&ty.vis),
                    line_start: tls,
                    line_end: tle,
                    dependencies: vec![],
                    behavior_summary: first_line(&extract_doc(&ty.attrs)).unwrap_or_else(|| {
                        format!("Associated type `{}` on `{}`.", tname, self_ty)
                    }),
                    risk_level: "low".to_string(),
                    body_hash: sha_hex(text.as_bytes()),
                    notes: vec![],
                });
            }
            _ => {}
        }
    }
}

fn emit_fn(f: &ItemFn, ctx: &mut Ctx) {
    let name = f.sig.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(f.span());
    let text = slice_lines(ctx.lines, ls, le);
    let signature = sig_to_string(&f.sig);
    let doc = extract_doc(&f.attrs);

    let mut notes = vec![];
    if f.sig.unsafety.is_some() {
        notes.push("unsafe fn".to_string());
    }
    if f.sig.asyncness.is_some() {
        notes.push("async fn".to_string());
    }
    if is_lua_binding(&ctx.file, &text) {
        notes.push("lua binding (heuristic)".to_string());
    }
    if is_scheduler_event(&ctx.file, &name, &text) {
        notes.push("scheduler event (heuristic)".to_string());
    }
    if name == "main" {
        notes.push("binary entry point".to_string());
    }

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: "function".to_string(),
        qualified_name: qn,
        signature,
        visibility: vis_str(&f.vis),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&f.to_token_stream()),
        behavior_summary: first_line(&doc).unwrap_or_else(|| {
            describe_fn(&name, &f.sig)
        }),
        risk_level: risk_for_fn(&name, &ctx.file, &text),
        body_hash: sha_hex(text.as_bytes()),
        notes,
    });
}

fn emit_const(c: &ItemConst, ctx: &mut Ctx) {
    let name = c.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(c.span());
    let text = slice_lines(ctx.lines, ls, le);
    let signature = compact_tokens(&c.to_token_stream(), 240);
    let doc = extract_doc(&c.attrs);

    let kind = if is_opcode_context_const(&ctx.file, &name) {
        "protocol_opcode"
    } else {
        "constant"
    }
    .to_string();

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind,
        qualified_name: qn,
        signature,
        visibility: vis_str(&c.vis),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&c.ty.to_token_stream()),
        behavior_summary: first_line(&doc).unwrap_or_else(|| {
            format!("Constant `{}`.", name)
        }),
        risk_level: risk_for_const(&name, &ctx.file),
        body_hash: sha_hex(text.as_bytes()),
        notes: vec![],
    });
}

fn emit_static(s: &ItemStatic, ctx: &mut Ctx) {
    let name = s.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(s.span());
    let text = slice_lines(ctx.lines, ls, le);
    let signature = compact_tokens(&s.to_token_stream(), 240);
    let doc = extract_doc(&s.attrs);

    let mut notes = vec![];
    if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
        notes.push("mutable static (unsafe access required)".to_string());
    }

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: "static".to_string(),
        qualified_name: qn,
        signature,
        visibility: vis_str(&s.vis),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&s.ty.to_token_stream()),
        behavior_summary: first_line(&doc).unwrap_or_else(|| {
            format!("Static binding `{}`.", name)
        }),
        risk_level: if notes.iter().any(|n| n.contains("mutable")) {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        body_hash: sha_hex(text.as_bytes()),
        notes,
    });
}

fn emit_type_alias(t: &ItemType, ctx: &mut Ctx) {
    let name = t.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(t.span());
    let text = slice_lines(ctx.lines, ls, le);
    let signature = compact_tokens(&t.to_token_stream(), 240);
    let doc = extract_doc(&t.attrs);
    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: "type_alias".to_string(),
        qualified_name: qn,
        signature,
        visibility: vis_str(&t.vis),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&t.ty.to_token_stream()),
        behavior_summary: first_line(&doc).unwrap_or_else(|| {
            format!("Type alias `{}`.", name)
        }),
        risk_level: "low".to_string(),
        body_hash: sha_hex(text.as_bytes()),
        notes: vec![],
    });
}

fn emit_macro(m: &ItemMacro, ctx: &mut Ctx) {
    let name = m
        .ident
        .as_ref()
        .map(|i| i.to_string())
        .unwrap_or_else(|| m.mac.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default());
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(m.span());
    let text = slice_lines(ctx.lines, ls, le);
    let signature = compact_tokens(&m.to_token_stream(), 240);
    let mac_path = compact_tokens(&m.mac.path.to_token_stream(), 80);

    let kind = if mac_path == "macro_rules" || m.ident.is_some() {
        "macro"
    } else {
        "macro_invocation"
    }
    .to_string();
    let doc = extract_doc(&m.attrs);

    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind,
        qualified_name: qn,
        signature,
        visibility: if m.ident.is_some() { "pub".to_string() } else { "private".to_string() },
        line_start: ls,
        line_end: le,
        dependencies: vec![mac_path.clone()],
        behavior_summary: first_line(&doc).unwrap_or_else(|| {
            if m.ident.is_some() {
                format!("Declarative macro `{}!` (macro_rules).", name)
            } else {
                format!("Top-level macro invocation `{}!`.", mac_path)
            }
        }),
        risk_level: if m.ident.is_some() { "medium".to_string() } else { "low".to_string() },
        body_hash: sha_hex(text.as_bytes()),
        notes: maybe_uncertain(&["macro body not deeply parsed"]),
    });
}

fn emit_union(u: &syn::ItemUnion, ctx: &mut Ctx) {
    let name = u.ident.to_string();
    let qn = join_qn(&ctx.mod_path, &name);
    let (ls, le) = span_lines(u.span());
    let text = slice_lines(ctx.lines, ls, le);
    let signature = compact_tokens(&u.to_token_stream(), 240);
    ctx.out.push(Symbol {
        file: ctx.file.clone(),
        kind: "union".to_string(),
        qualified_name: qn,
        signature,
        visibility: vis_str(&u.vis),
        line_start: ls,
        line_end: le,
        dependencies: collect_deps_in_tokens(&u.to_token_stream()),
        behavior_summary: format!("Union `{}` with {} fields.", name, u.fields.named.len()),
        risk_level: "high".to_string(),
        body_hash: sha_hex(text.as_bytes()),
        notes: vec!["union (unsafe to read)".to_string()],
    });
}

// ───────────────────────────────────────────────────────────────────────────────
// Helpers
// ───────────────────────────────────────────────────────────────────────────────

fn join_qn(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}::{name}")
    }
}

fn vis_str(v: &Visibility) -> String {
    match v {
        Visibility::Public(_) => "pub".to_string(),
        Visibility::Restricted(r) => format!("pub({})", compact_tokens(&r.in_token.to_token_stream(), 0).trim_start_matches("in ").trim().to_string() + &compact_tokens(&r.path.to_token_stream(), 80)),
        Visibility::Inherited => "private".to_string(),
    }
}

fn span_lines(span: Span) -> (usize, usize) {
    let s = span.start();
    let e = span.end();
    let start_line = if s.line == 0 { 1 } else { s.line };
    let end_line = if e.line == 0 { start_line } else { e.line };
    (start_line, end_line.max(start_line))
}

fn slice_lines(lines: &[&str], start: usize, end: usize) -> String {
    let s = start.saturating_sub(1).min(lines.len());
    let e = end.min(lines.len());
    if s >= e {
        return String::new();
    }
    lines[s..e].join("\n")
}

fn sha_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn compact_tokens(t: &proc_macro2::TokenStream, max: usize) -> String {
    let raw = t.to_string();
    let single: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if max > 0 && single.len() > max {
        format!("{}…", &single[..max])
    } else {
        single
    }
}

fn sig_to_string(s: &Signature) -> String {
    let mut buf = String::new();
    if s.constness.is_some() {
        buf.push_str("const ");
    }
    if s.asyncness.is_some() {
        buf.push_str("async ");
    }
    if s.unsafety.is_some() {
        buf.push_str("unsafe ");
    }
    if let Some(abi) = &s.abi {
        buf.push_str(&compact_tokens(&abi.to_token_stream(), 60));
        buf.push(' ');
    }
    buf.push_str("fn ");
    buf.push_str(&s.ident.to_string());
    let generics = compact_tokens(&s.generics.to_token_stream(), 120);
    if !generics.is_empty() {
        buf.push_str(&generics);
    }
    buf.push('(');
    let inputs: Vec<String> = s
        .inputs
        .iter()
        .map(|a| compact_tokens(&a.to_token_stream(), 120))
        .collect();
    buf.push_str(&inputs.join(", "));
    buf.push(')');
    match &s.output {
        ReturnType::Default => {}
        ReturnType::Type(_, t) => {
            buf.push_str(" -> ");
            buf.push_str(&compact_tokens(&t.to_token_stream(), 120));
        }
    }
    if let Some(wc) = &s.generics.where_clause {
        buf.push(' ');
        buf.push_str(&compact_tokens(&wc.to_token_stream(), 200));
    }
    buf
}

fn has_self_receiver(s: &Signature) -> bool {
    matches!(s.inputs.first(), Some(syn::FnArg::Receiver(_)))
}

fn extract_doc(attrs: &[Attribute]) -> String {
    let mut out = String::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(mnv) = &attr.meta {
                if let Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &mnv.value {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    out.push_str(s.value().trim());
                }
            }
        }
    }
    out
}

fn first_line(s: &str) -> Option<String> {
    let line = s.lines().next()?.trim();
    if line.is_empty() {
        None
    } else {
        Some(line.to_string())
    }
}

fn maybe_uncertain(extra: &[&str]) -> Vec<String> {
    extra.iter().map(|s| s.to_string()).collect()
}

fn collect_derives(attrs: &[Attribute]) -> Vec<String> {
    let mut out = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|nested| {
                let s = compact_tokens(&nested.path.to_token_stream(), 80);
                out.push(s);
                Ok(())
            });
        }
    }
    out
}

fn is_serde_serializable(attrs: &[Attribute]) -> bool {
    collect_derives(attrs)
        .iter()
        .any(|d| d == "Serialize" || d == "Deserialize" || d.ends_with("::Serialize") || d.ends_with("::Deserialize"))
}

fn is_sea_orm_entity(attrs: &[Attribute]) -> bool {
    collect_derives(attrs)
        .iter()
        .any(|d| d.contains("DeriveEntityModel") || d.contains("DeriveEntity") || d.contains("FromQueryResult"))
        || attrs
            .iter()
            .any(|a| a.path().is_ident("sea_orm") || a.path().is_ident("table_name"))
}

fn is_error_type(name: &str, derives: &[String]) -> bool {
    name.ends_with("Error")
        || derives.iter().any(|d| d == "Error" || d.ends_with("::Error") || d == "thiserror::Error")
}

fn is_opcode_enum(name: &str, file: &str) -> bool {
    let f = file.to_lowercase();
    (f.contains("network") || f.contains("protocol")) && {
        let n = name.to_lowercase();
        n.contains("opcode") || n.contains("packet") || n.ends_with("kind") || n.ends_with("type")
    }
}

fn is_opcode_context(parent_name: &str, file: &str) -> bool {
    is_opcode_enum(parent_name, file)
}

fn is_opcode_context_const(file: &str, name: &str) -> bool {
    let f = file.to_lowercase();
    let n = name.to_lowercase();
    (f.contains("network") || f.contains("protocol"))
        && (n.contains("opcode")
            || n.contains("packet")
            || n.contains("header")
            || n.starts_with("op_")
            || n.starts_with("cmd_"))
}

fn is_lua_binding(file: &str, body: &str) -> bool {
    let f = file.to_lowercase();
    if f.contains("lua_api") || f.contains("scripting") {
        return true;
    }
    body.contains("LuaState")
        || body.contains("LuaResult")
        || body.contains("mlua::")
        || body.contains("rlua::")
        || body.contains("lua_register")
        || body.contains("RegisterFunction")
}

fn is_scheduler_event(file: &str, name: &str, body: &str) -> bool {
    let f = file.to_lowercase();
    let n = name.to_lowercase();
    f.contains("scheduler")
        || n.contains("schedule")
        || n.contains("dispatcher")
        || body.contains("Scheduler::")
        || body.contains("dispatcher::")
        || body.contains("addEvent")
        || body.contains("add_event")
}

fn risk_for_module(file: &str) -> String {
    let f = file.to_lowercase();
    if f.contains("network") || f.contains("protocol") || f.contains("xtea") || f.contains("rsa") {
        "high".to_string()
    } else if f.contains("database") || f.contains("iologindata") || f.contains("iomarket") {
        "high".to_string()
    } else if f.contains("scripting") || f.contains("lua") {
        "high".to_string()
    } else if f.contains("game") || f.contains("combat") || f.contains("movement") {
        "high".to_string()
    } else if f.contains("entity") || f.contains("items") || f.contains("map") || f.contains("world") {
        "medium".to_string()
    } else {
        "medium".to_string()
    }
}

fn risk_for_type(name: &str, file: &str, kind: &str) -> String {
    let module_risk = risk_for_module(file);
    let n = name.to_lowercase();
    if kind == "error" {
        return "medium".to_string();
    }
    if n.ends_with("error") {
        return "medium".to_string();
    }
    if module_risk == "high" {
        return "high".to_string();
    }
    if matches!(kind, "trait" | "database_mapping") {
        return "high".to_string();
    }
    "medium".to_string()
}

fn risk_for_fn(name: &str, file: &str, body: &str) -> String {
    let n = name.to_lowercase();
    let module_risk = risk_for_module(file);
    if body.contains("unsafe ") {
        return "critical".to_string();
    }
    if n.contains("drop") || n.contains("kill") || n.contains("shutdown") || n.contains("delete") || n.contains("panic") {
        return "high".to_string();
    }
    if n.contains("login") || n.contains("logout") || n.contains("auth") || n.contains("password") || n.contains("token") {
        return "high".to_string();
    }
    if n.contains("encrypt") || n.contains("decrypt") || n.contains("hash") || n.contains("rsa") || n.contains("xtea") {
        return "critical".to_string();
    }
    if n.contains("save") || n.contains("load") || n.contains("read") || n.contains("write") || n.contains("flush") {
        return "high".to_string();
    }
    if module_risk == "high" {
        return "high".to_string();
    }
    if matches!(n.as_str(), "new" | "default" | "clone" | "len" | "is_empty" | "name" | "id" | "fmt") {
        return "low".to_string();
    }
    "medium".to_string()
}

fn risk_for_const(name: &str, file: &str) -> String {
    let n = name.to_lowercase();
    let f = file.to_lowercase();
    if f.contains("network") || f.contains("protocol") || n.contains("opcode") || n.contains("packet") {
        "high".to_string()
    } else if n.contains("key") || n.contains("secret") {
        "critical".to_string()
    } else {
        "low".to_string()
    }
}

fn describe_struct(name: &str, fields: &Fields) -> String {
    match fields {
        Fields::Named(n) => format!("Struct `{}` with {} named fields.", name, n.named.len()),
        Fields::Unnamed(u) => format!("Tuple struct `{}` with {} positional fields.", name, u.unnamed.len()),
        Fields::Unit => format!("Unit struct `{}`.", name),
    }
}

fn describe_fn(name: &str, sig: &Signature) -> String {
    let arity = sig.inputs.len();
    let ret = match &sig.output {
        ReturnType::Default => "()".to_string(),
        ReturnType::Type(_, t) => compact_tokens(&t.to_token_stream(), 80),
    };
    format!("Function `{}` ({} args) returning `{}`.", name, arity, ret)
}

fn describe_method(name: &str, self_ty: &str, trait_path: Option<&str>) -> String {
    match trait_path {
        Some(tp) => format!("Method `{}` implementing `{}` for `{}`.", name, tp, self_ty),
        None => format!("Inherent method `{}` on `{}`.", name, self_ty),
    }
}

fn type_root_name(t: &Type) -> String {
    if let Type::Path(tp) = t {
        if let Some(seg) = tp.path.segments.last() {
            return seg.ident.to_string();
        }
    }
    compact_tokens(&t.to_token_stream(), 120)
}

fn collect_deps_in_tokens(t: &proc_macro2::TokenStream) -> Vec<String> {
    if let Ok(parsed) = syn::parse2::<syn::Type>(t.clone()) {
        let mut v = PathVisitor { set: BTreeSet::new() };
        v.visit_type(&parsed);
        return finalize_paths(v.set);
    }
    if let Ok(parsed) = syn::parse2::<syn::ItemFn>(t.clone()) {
        let mut v = PathVisitor { set: BTreeSet::new() };
        v.visit_item_fn(&parsed);
        return finalize_paths(v.set);
    }
    if let Ok(parsed) = syn::parse2::<syn::ImplItemFn>(t.clone()) {
        let mut v = PathVisitor { set: BTreeSet::new() };
        v.visit_impl_item_fn(&parsed);
        return finalize_paths(v.set);
    }
    if let Ok(parsed) = syn::parse2::<syn::Item>(t.clone()) {
        let mut v = PathVisitor { set: BTreeSet::new() };
        v.visit_item(&parsed);
        return finalize_paths(v.set);
    }
    // Fallback: best-effort scan of identifiers in the raw token stream
    let mut out = BTreeSet::new();
    for tok in t.clone().into_iter() {
        if let proc_macro2::TokenTree::Ident(id) = tok {
            let s = id.to_string();
            if s.chars().next().map_or(false, |c| c.is_uppercase()) {
                out.insert(s);
            }
        }
    }
    finalize_paths(out)
}

struct PathVisitor {
    set: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for PathVisitor {
    fn visit_path(&mut self, p: &'ast syn::Path) {
        let segs: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
        if !segs.is_empty() {
            self.set.insert(segs.join("::"));
        }
        syn::visit::visit_path(self, p);
    }
}

fn finalize_paths(set: BTreeSet<String>) -> Vec<String> {
    set.into_iter()
        .filter(|p| !is_builtin_path(p))
        .take(40)
        .collect()
}

fn is_builtin_path(p: &str) -> bool {
    matches!(
        p,
        "Self"
            | "self"
            | "super"
            | "crate"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
            | "bool"
            | "char"
            | "str"
            | "String"
            | "Option"
            | "Some"
            | "None"
            | "Result"
            | "Ok"
            | "Err"
            | "Box"
            | "Vec"
    )
}

fn collect_use_paths(items: &[Item]) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for item in items {
        if let Item::Use(u) = item {
            let s = compact_tokens(&u.tree.to_token_stream(), 200);
            set.insert(s);
        }
    }
    set.into_iter().take(60).collect()
}

// ───────────────────────────────────────────────────────────────────────────────
// Module path resolution
// ───────────────────────────────────────────────────────────────────────────────

fn module_path_from_file(path: &Path, root: &Path) -> String {
    // Expected layout: <root>/<crate>/src/(lib.rs | main.rs | <sub>/...)
    let rel = match path.strip_prefix(root) {
        Ok(p) => p.to_path_buf(),
        Err(_) => path.to_path_buf(),
    };
    let comps: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if comps.is_empty() {
        return "".to_string();
    }
    let crate_name = comps.first().cloned().unwrap_or_default();
    // Drop the "src" component
    let mut tail: Vec<String> = comps
        .iter()
        .skip(1)
        .skip_while(|c| c.as_str() != "src")
        .skip(1)
        .cloned()
        .collect();
    if tail.is_empty() {
        return crate_name;
    }
    // Strip trailing .rs and special filenames
    let last = tail.last().cloned().unwrap_or_default();
    tail.pop();
    let mut segs: Vec<String> = vec![crate_name.replace('-', "_")];
    segs.extend(tail.into_iter().map(|s| s.replace('-', "_")));
    let last_no_ext = last.trim_end_matches(".rs").to_string();
    match last_no_ext.as_str() {
        "lib" | "main" | "mod" => {}
        other => segs.push(other.to_string().replace('-', "_")),
    }
    segs.join("::")
}
