//! `Container:*` Lua binding for `items::container::Container`.
//!
//! Methods returning Item userdata are stubbed until the Item
//! binding ships (return nil).

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_items::container::Container;
use mlua::{UserData, UserDataMethods, Value};

#[derive(Debug, Clone)]
pub struct LuaContainer(pub Container);

impl LuaContainer {
    pub fn new(c: Container) -> Self {
        Self(c)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaContainer {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaContainer>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaContainer",
                message: Some("expected Container userdata".into()),
            }),
        }
    }
}

impl UserData for LuaContainer {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, _this, _other: Value| Ok(false));

        methods.add_method("getCapacity", |_, this, ()| Ok(this.0.capacity() as i64));
        methods.add_method("getSize", |_, this, ()| Ok(this.0.size() as i64));
        methods.add_method("getEmptySlots", |_, this, ()| {
            Ok((this.0.capacity() as i64 - this.0.size() as i64).max(0))
        });
        methods.add_method("getItemHoldingCount", |_, this, ()| {
            Ok(this.0.get_item_holding_count() as i64)
        });
        methods.add_method(
            "getItemCountById",
            |_, this, (id, _sub): (i64, Option<i64>)| {
                // Always pass None for sub_type — the variant we have wraps the C++ default.
                Ok(this.0.get_item_type_count(id as u16, None) as i64)
            },
        );
        methods.add_method("hasItem", |_, this, item_id: i64| {
            Ok(this.0.is_holding_item(item_id as u16))
        });
        methods.add_method("getCorpseOwner", |_, _this, ()| {
            // Stub: corpse owner attribute access requires reading item attrs;
            // returns 0 until Item attribute binding ships.
            Ok(0i64)
        });
        // Stubs returning nil until Item binding ships:
        methods.add_method("getItem", |_, _this, _idx: i64| Ok(mlua::Value::Nil));
        methods.add_method("getItems", |lua, _this, ()| lua.create_table());
        // addItem / addItemEx: stub — would need Item type.
        methods.add_method_mut(
            "addItem",
            |_, _this, _args: (Value, Option<i64>, Option<i64>)| Ok(mlua::Value::Nil),
        );
        methods.add_method_mut(
            "addItemEx",
            |_, _this, _args: (Value, Option<i64>, Option<i64>, Option<i64>)| Ok(0i64),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_lua() -> mlua::Lua {
        let lua = mlua::Lua::new();
        crate::lua_bindings::install_bindings(
            &lua,
            crate::lua_bindings::GameStateHandle::default(),
        )
        .unwrap();
        lua
    }

    fn sample_container() -> Container {
        // Container::new(item_type_id, capacity)
        Container::new(1987, 20)
    }

    #[test]
    fn get_capacity_returns_struct_field() {
        let lua = fresh_lua();
        lua.globals()
            .set("c", LuaContainer::new(sample_container()))
            .unwrap();
        let v: i64 = lua.load("return c:getCapacity()").eval().unwrap();
        assert_eq!(v, 20);
    }

    #[test]
    fn get_size_returns_zero_for_empty_container() {
        let lua = fresh_lua();
        lua.globals()
            .set("c", LuaContainer::new(sample_container()))
            .unwrap();
        let v: i64 = lua.load("return c:getSize()").eval().unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn get_empty_slots_returns_capacity_for_empty_container() {
        let lua = fresh_lua();
        lua.globals()
            .set("c", LuaContainer::new(sample_container()))
            .unwrap();
        let v: i64 = lua.load("return c:getEmptySlots()").eval().unwrap();
        assert_eq!(v, 20);
    }
}
