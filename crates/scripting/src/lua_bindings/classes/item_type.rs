//! `ItemType:*` Lua binding for `items::registry::ItemType`.
//!
//! The Rust `ItemType` struct exposes a small subset (server_id,
//! client_id, group, flags, speed, weight). Most C++ `ItemType`
//! methods are bound here as stubs returning C++ defaults (0/""/false)
//! until the full XML loader extends the Rust struct. Methods backed
//! by real fields return live values.

#![cfg(feature = "lua-scripting")]
#![allow(dead_code)]

use forgottenserver_items::registry::ItemType;
use mlua::{UserData, UserDataMethods};

#[derive(Debug, Clone)]
pub struct LuaItemType(pub ItemType);

impl LuaItemType {
    pub fn new(it: ItemType) -> Self {
        Self(it)
    }
}

impl<'lua> mlua::FromLua<'lua> for LuaItemType {
    fn from_lua(value: mlua::Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(ud.borrow::<LuaItemType>()?.clone()),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "LuaItemType",
                message: Some("expected ItemType userdata".into()),
            }),
        }
    }
}

impl UserData for LuaItemType {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(mlua::MetaMethod::Eq, |_, this, other: LuaItemType| {
            Ok(this.0.server_id == other.0.server_id)
        });

        // ── Real bindings ─────────────────────────────────────────
        methods.add_method("getId", |_, this, ()| Ok(this.0.server_id as i64));
        methods.add_method("getClientId", |_, this, ()| Ok(this.0.client_id as i64));
        methods.add_method("getGroup", |_, this, ()| Ok(this.0.group as i64));
        methods.add_method("getWeight", |_, this, ()| Ok(this.0.weight as i64));
        methods.add_method("getType", |_, this, ()| Ok(this.0.group as i64));

        // ── Stubs (return C++ default) ────────────────────────────
        // Most return 0 / "" / false until the full ItemType struct is migrated.
        for_each_stub_int(
            methods,
            &[
                "getAbilities",
                "getAmmoType",
                "getArmor",
                "getAttack",
                "getAttackSpeed",
                "getCapacity",
                "getCharges",
                "getClassification",
                "getCorpseType",
                "getDecayId",
                "getDefense",
                "getDestroyId",
                "getDurationMax",
                "getDurationMin",
                "getElementDamage",
                "getElementType",
                "getExtraDefense",
                "getFluidSource",
                "getHitChance",
                "getLevelDoor",
                "getMinReqLevel",
                "getMinReqMagicLevel",
                "getRequiredLevel",
                "getRotateTo",
                "getShootRange",
                "getSlotPosition",
                "getTransformDeEquipId",
                "getTransformEquipId",
                "getWeaponType",
                "getWieldInfo",
                "getWorth",
            ],
        );
        for_each_stub_string(
            methods,
            &[
                "getArticle",
                "getDescription",
                "getName",
                "getPluralName",
                "getRuneSpellName",
                "getVocationString",
            ],
        );
        for_each_stub_bool(
            methods,
            &[
                "hasAllowDistRead",
                "hasShowAttributes",
                "hasShowCharges",
                "hasShowCount",
                "hasShowDuration",
                "hasSubType",
                "isBlocking",
                "isContainer",
                "isCorpse",
                "isDoor",
                "isFluidContainer",
                "isGroundTile",
                "isMagicField",
                "isMovable",
                "isPickupable",
                "isReadable",
                "isRotatable",
                "isRune",
                "isStackable",
                "isStoreItem",
                "isUseable",
                "isWritable",
            ],
        );
        // Market statistics return tables of {transactions=N, total=N}; stub empty.
        methods.add_method("getMarketBuyStatistics", |lua, _this, ()| {
            lua.create_table()
        });
        methods.add_method("getMarketSellStatistics", |lua, _this, ()| {
            lua.create_table()
        });
    }
}

fn for_each_stub_int<'lua, M: UserDataMethods<'lua, LuaItemType>>(
    methods: &mut M,
    names: &[&'static str],
) {
    for name in names {
        methods.add_method(name, |_, _this, ()| Ok(0i64));
    }
}
fn for_each_stub_string<'lua, M: UserDataMethods<'lua, LuaItemType>>(
    methods: &mut M,
    names: &[&'static str],
) {
    for name in names {
        methods.add_method(name, |_, _this, ()| Ok(String::new()));
    }
}
fn for_each_stub_bool<'lua, M: UserDataMethods<'lua, LuaItemType>>(
    methods: &mut M,
    names: &[&'static str],
) {
    for name in names {
        methods.add_method(name, |_, _this, ()| Ok(false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forgottenserver_common::itemloader::ItemGroup;

    fn fresh_lua() -> mlua::Lua {
        let lua = mlua::Lua::new();
        crate::lua_bindings::install_bindings(
            &lua,
            crate::lua_bindings::GameStateHandle::default(),
        )
        .unwrap();
        lua
    }

    fn sample_item_type() -> ItemType {
        ItemType {
            server_id: 1987,
            client_id: 1987,
            group: ItemGroup::Container,
            flags: 0,
            speed: 0,
            weight: 1000,
        }
    }

    #[test]
    fn get_id_returns_server_id() {
        let lua = fresh_lua();
        lua.globals()
            .set("it", LuaItemType::new(sample_item_type()))
            .unwrap();
        let v: i64 = lua.load("return it:getId()").eval().unwrap();
        assert_eq!(v, 1987);
    }

    #[test]
    fn get_weight_returns_field() {
        let lua = fresh_lua();
        lua.globals()
            .set("it", LuaItemType::new(sample_item_type()))
            .unwrap();
        let v: i64 = lua.load("return it:getWeight()").eval().unwrap();
        assert_eq!(v, 1000);
    }

    #[test]
    fn stub_methods_return_defaults() {
        let lua = fresh_lua();
        lua.globals()
            .set("it", LuaItemType::new(sample_item_type()))
            .unwrap();
        let v: i64 = lua.load("return it:getAttack()").eval().unwrap();
        assert_eq!(v, 0);
        let v: bool = lua.load("return it:isStackable()").eval().unwrap();
        assert!(!v);
    }
}
