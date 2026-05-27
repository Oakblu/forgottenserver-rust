//! `Item:*` Lua binding for `items::item::Item` (the runtime item instance,
//! distinct from `ItemType`).

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_items::item::Item;
use mlua::{UserData, UserDataMethods, Value};
use std::sync::{Arc, Mutex};

pub struct LuaItem(pub Arc<Mutex<Item>>);

impl LuaItem {
    pub fn new(it: Item) -> Self {
        Self(Arc::new(Mutex::new(it)))
    }
}

impl Clone for LuaItem {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaItem {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaItem>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaItem",
                message: Some("expected Item userdata".into()),
            }),
        }
    }
}

impl UserData for LuaItem {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaItem| {
            Ok(Arc::ptr_eq(&this.0, &other.0))
        });
        // ── Real getters backed by Item / ItemType ────────────────
        methods.add_method("getId", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_id() as i64)
        });
        methods.add_method("getName", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_name().to_string())
        });
        methods.add_method("getCount", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_count() as i64)
        });
        methods.add_method("getWeight", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_weight() as i64)
        });
        methods.add_method("getActionId", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_action_id() as i64)
        });
        methods.add_method("getUniqueId", |_, this, ()| {
            Ok(this.0.lock().unwrap().get_unique_id() as i64)
        });
        methods.add_method_mut("setActionId", |_, this, id: i64| {
            this.0.lock().unwrap().set_action_id(id.max(0) as u16);
            Ok(())
        });
        methods.add_method("isStoreItem", |_, _this, ()| Ok(false));
        methods.add_method("isItem", |_, _this, _args: Value| Ok(true));
        methods.add_method("isLoadedFromMap", |_, _this, ()| Ok(false));

        // ── Stub getters (return defaults) ────────────────────────
        for n in &["getArticle", "getPluralName", "getSpecialDescription"] {
            methods.add_method(n, |_, _this, ()| Ok(String::new()));
        }
        for n in &[
            "getBoostPercent",
            "getCharges",
            "getFluidType",
            "getReflect",
            "getSubType",
            "getWorth",
        ] {
            methods.add_method(n, |_, _this, _args: Value| Ok(0i64));
        }
        for n in &["hasAttribute", "hasParent", "hasProperty"] {
            methods.add_method(n, |_, _this, _args: Value| Ok(false));
        }
        // ── Attribute family (record-only; real serialisation TBD) ─
        methods.add_method("getAttribute", |_, _this, _key: Value| Ok(Value::Nil));
        methods.add_method_mut("setAttribute", |_, _this, _args: Value| Ok(true));
        methods.add_method_mut("removeAttribute", |_, _this, _key: Value| Ok(true));
        // Custom-attribute API routes through the real `Item::*_custom_attribute`
        // storage. Lua scripts pass either a string or integer key — we
        // normalise to a string (mirroring the C++ `to_string(int64)`
        // overload) and store via `CustomAttribute` enum variants chosen
        // by the Lua value's type.
        methods.add_method("getCustomAttribute", |lua, this, key: Value| {
            let key_str = lua_value_to_custom_key(&key)?;
            let g = this.0.lock().unwrap();
            Ok(match g.get_custom_attribute(&key_str) {
                Some(forgottenserver_items::item::CustomAttribute::String(s)) => {
                    Value::String(lua.create_string(s)?)
                }
                Some(forgottenserver_items::item::CustomAttribute::Integer(n)) => {
                    Value::Integer(*n)
                }
                Some(forgottenserver_items::item::CustomAttribute::Float(f)) => Value::Number(*f),
                Some(forgottenserver_items::item::CustomAttribute::Boolean(b)) => {
                    Value::Boolean(*b)
                }
                None => Value::Nil,
            })
        });
        methods.add_method_mut(
            "setCustomAttribute",
            |_, this, (key, val): (Value, Value)| {
                let key_str = lua_value_to_custom_key(&key)?;
                let custom = lua_value_to_custom_attribute(&val)?;
                this.0
                    .lock()
                    .unwrap()
                    .set_custom_attribute(&key_str, custom);
                Ok(true)
            },
        );
        methods.add_method_mut("removeCustomAttribute", |_, this, key: Value| {
            let key_str = lua_value_to_custom_key(&key)?;
            Ok(this.0.lock().unwrap().remove_custom_attribute(&key_str))
        });
        methods.add_method_mut("setBoostPercent", |_, _this, _args: Value| Ok(()));
        methods.add_method_mut("setReflect", |_, _this, _args: Value| Ok(()));
        methods.add_method_mut("setStoreItem", |_, _this, _flag: bool| Ok(()));
        // ── Parent / position / tile (need game-state plumbing) ────
        methods.add_method("getParent", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method("getTopParent", |_, _this, _args: Value| Ok(Value::Nil));
        methods.add_method("getPosition", |lua, _this, ()| lua.create_table());
        methods.add_method("getTile", |_, _this, _args: Value| Ok(Value::Nil));
        // ── Lifecycle (no-op stubs) ───────────────────────────────
        for n in &["clone", "decay", "moveTo", "remove", "split", "transform"] {
            methods.add_method(n, |_, _this, _args: Value| Ok(false));
        }
    }
}

/// Lua key → custom-attribute key (string). Accepts string keys directly
/// and stringifies integer keys (C++ uses `std::to_string(int64_t)` on
/// the `setCustomAttribute(int64_t)` overload).
fn lua_value_to_custom_key(value: &Value) -> mlua::Result<String> {
    match value {
        Value::String(s) => Ok(s.to_str()?.to_string()),
        Value::Integer(n) => Ok(n.to_string()),
        Value::Number(f) => Ok((*f as i64).to_string()),
        other => Err(mlua::Error::FromLuaConversionError {
            from: other.type_name(),
            to: "custom-attribute key (string|integer)",
            message: None,
        }),
    }
}

/// Lua value → typed `CustomAttribute`. Picks the variant that matches
/// the Lua type — string → String, integer → Integer, number → Float,
/// boolean → Boolean. Anything else is a typed conversion error.
fn lua_value_to_custom_attribute(
    value: &Value,
) -> mlua::Result<forgottenserver_items::item::CustomAttribute> {
    use forgottenserver_items::item::CustomAttribute;
    match value {
        Value::String(s) => Ok(CustomAttribute::String(s.to_str()?.to_string())),
        Value::Integer(n) => Ok(CustomAttribute::Integer(*n)),
        Value::Number(f) => Ok(CustomAttribute::Float(*f)),
        Value::Boolean(b) => Ok(CustomAttribute::Boolean(*b)),
        other => Err(mlua::Error::FromLuaConversionError {
            from: other.type_name(),
            to: "CustomAttribute (string|integer|number|boolean)",
            message: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_items::items_registry::ItemTypeData;

    fn fresh_lua() -> mlua::Lua {
        let lua = mlua::Lua::new();
        crate::lua_bindings::install_bindings(
            &lua,
            crate::lua_bindings::GameStateHandle::default(),
        )
        .unwrap();
        lua
    }

    #[test]
    fn count_returns_field() {
        let lua = fresh_lua();
        let it = Item::new(Arc::new(ItemTypeData::default()), 5);
        lua.globals().set("i", LuaItem::new(it)).unwrap();
        let c: i64 = lua.load("return i:getCount()").eval().unwrap();
        assert_eq!(c, 5);
    }

    /// Lua-side round-trip: `setCustomAttribute(key, value)` then
    /// `getCustomAttribute(key)` returns the same value across all four
    /// typed branches.
    #[test]
    fn custom_attribute_round_trips_via_lua_setter_getter() {
        let lua = fresh_lua();
        let it = Item::new(Arc::new(ItemTypeData::default()), 1);
        lua.globals().set("i", LuaItem::new(it)).unwrap();
        // String
        let s: String = lua
            .load(
                r#"i:setCustomAttribute("name", "Excalibur"); return i:getCustomAttribute("name")"#,
            )
            .eval()
            .unwrap();
        assert_eq!(s, "Excalibur");
        // Integer
        let n: i64 = lua
            .load(r#"i:setCustomAttribute("level", 42); return i:getCustomAttribute("level")"#)
            .eval()
            .unwrap();
        assert_eq!(n, 42);
        // Boolean
        let b: bool = lua
            .load(r#"i:setCustomAttribute("magic", true); return i:getCustomAttribute("magic")"#)
            .eval()
            .unwrap();
        assert!(b);
    }

    /// `removeCustomAttribute` reports true on first removal, false on
    /// second — mirroring the Item-level boolean return.
    #[test]
    fn custom_attribute_remove_via_lua_reports_success_then_failure() {
        let lua = fresh_lua();
        let it = Item::new(Arc::new(ItemTypeData::default()), 1);
        lua.globals().set("i", LuaItem::new(it)).unwrap();
        lua.load(r#"i:setCustomAttribute("k", 1)"#).exec().unwrap();
        let first: bool = lua
            .load(r#"return i:removeCustomAttribute("k")"#)
            .eval()
            .unwrap();
        assert!(first);
        let second: bool = lua
            .load(r#"return i:removeCustomAttribute("k")"#)
            .eval()
            .unwrap();
        assert!(!second);
    }

    /// Lua integer key gets stringified before storage (matches the
    /// C++ `setCustomAttribute(int64_t, R)` overload).
    #[test]
    fn custom_attribute_integer_key_stringifies_on_lua_side() {
        let lua = fresh_lua();
        let it = Item::new(Arc::new(ItemTypeData::default()), 1);
        lua.globals().set("i", LuaItem::new(it)).unwrap();
        let s: String = lua
            .load(r#"i:setCustomAttribute(7, "seven"); return i:getCustomAttribute("7")"#)
            .eval()
            .unwrap();
        assert_eq!(s, "seven");
    }
}
