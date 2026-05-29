//! Migrated from forgottenserver/src/luavariant.h
//!
//! Tagged union type for Lua values used across scripting callbacks without
//! importing the Lua library into common. Maps C++ `std::variant<uint32_t,
//! Position, Position, std::string>` (with named indices) to a Rust enum.

#![allow(dead_code)]

use crate::position::Position;

// ---------------------------------------------------------------------------
// LuaVariantType – mirrors the C++ enum indices exactly
// ---------------------------------------------------------------------------

/// Discriminant for [`LuaVariant`].  Mirrors `LuaVariantType_t` from
/// `luavariant.h` with the same numeric values.
///
/// | C++ constant           | value |
/// |------------------------|-------|
/// | `VARIANT_NUMBER`       |   0   |
/// | `VARIANT_POSITION`     |   1   |
/// | `VARIANT_TARGETPOSITION` |  2  |
/// | `VARIANT_STRING`       |   3   |
/// | `VARIANT_NONE`         | usize::MAX (std::variant_npos) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LuaVariantType {
    Number = 0,
    Position = 1,
    TargetPosition = 2,
    String = 3,
    None = u8::MAX,
}

// ---------------------------------------------------------------------------
// LuaVariant
// ---------------------------------------------------------------------------

/// Tagged union for Lua callback values.
///
/// Mirrors the C++ `LuaVariant` class whose backing storage is
/// `std::variant<uint32_t, Position, Position, std::string>`.
///
/// Note that `Position` and `TargetPosition` carry the same data type
/// (`Position`) but are distinguished by the active discriminant — exactly as
/// the C++ code uses `std::get<VARIANT_POSITION>` vs
/// `std::get<VARIANT_TARGETPOSITION>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LuaVariant {
    /// `VARIANT_NONE` — no value set (default state).
    #[default]
    None,
    /// `VARIANT_NUMBER` — holds a `uint32_t` in C++.
    Number(u32),
    /// `VARIANT_POSITION` — a regular world position.
    Position(Position),
    /// `VARIANT_TARGETPOSITION` — a target position (same struct, different slot).
    TargetPosition(Position),
    /// `VARIANT_STRING` — a UTF-8 string.
    String(std::string::String),
}

impl LuaVariant {
    // -----------------------------------------------------------------------
    // Constructors (mirror `set*` methods)
    // -----------------------------------------------------------------------

    /// Create a `Number` variant.  Mirrors `setNumber`.
    pub fn number(value: u32) -> Self {
        LuaVariant::Number(value)
    }

    /// Create a `Position` variant.  Mirrors `setPosition`.
    pub fn position(value: Position) -> Self {
        LuaVariant::Position(value)
    }

    /// Create a `TargetPosition` variant.  Mirrors `setTargetPosition`.
    pub fn target_position(value: Position) -> Self {
        LuaVariant::TargetPosition(value)
    }

    /// Create a `String` variant.  Mirrors `setString`.
    pub fn string(value: impl Into<std::string::String>) -> Self {
        LuaVariant::String(value.into())
    }

    // -----------------------------------------------------------------------
    // Type query (mirror `is*` methods)
    // -----------------------------------------------------------------------

    pub fn is_none(&self) -> bool {
        matches!(self, LuaVariant::None)
    }

    pub fn is_number(&self) -> bool {
        matches!(self, LuaVariant::Number(_))
    }

    pub fn is_position(&self) -> bool {
        matches!(self, LuaVariant::Position(_))
    }

    pub fn is_target_position(&self) -> bool {
        matches!(self, LuaVariant::TargetPosition(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, LuaVariant::String(_))
    }

    // -----------------------------------------------------------------------
    // Discriminant (mirrors `type()`)
    // -----------------------------------------------------------------------

    /// Returns the [`LuaVariantType`] discriminant.  Mirrors the C++ `type()`
    /// method which casts `variant.index()` to `LuaVariantType_t`.
    pub fn variant_type(&self) -> LuaVariantType {
        match self {
            LuaVariant::None => LuaVariantType::None,
            LuaVariant::Number(_) => LuaVariantType::Number,
            LuaVariant::Position(_) => LuaVariantType::Position,
            LuaVariant::TargetPosition(_) => LuaVariantType::TargetPosition,
            LuaVariant::String(_) => LuaVariantType::String,
        }
    }

    // -----------------------------------------------------------------------
    // Value accessors (mirror `get*` methods – panic on wrong variant)
    // -----------------------------------------------------------------------

    /// Returns the inner number.  Panics if not `Number`.  Mirrors `getNumber`.
    pub fn get_number(&self) -> u32 {
        match self {
            LuaVariant::Number(n) => *n,
            _ => panic!("LuaVariant::get_number called on {:?}", self.variant_type()),
        }
    }

    /// Returns a reference to the inner position.  Panics if not `Position`.
    /// Mirrors `getPosition`.
    pub fn get_position(&self) -> &Position {
        match self {
            LuaVariant::Position(p) => p,
            _ => panic!(
                "LuaVariant::get_position called on {:?}",
                self.variant_type()
            ),
        }
    }

    /// Returns a reference to the inner target position.  Panics if not
    /// `TargetPosition`.  Mirrors `getTargetPosition`.
    pub fn get_target_position(&self) -> &Position {
        match self {
            LuaVariant::TargetPosition(p) => p,
            _ => panic!(
                "LuaVariant::get_target_position called on {:?}",
                self.variant_type()
            ),
        }
    }

    /// Returns a reference to the inner string.  Panics if not `String`.
    /// Mirrors `getString`.
    pub fn get_string(&self) -> &str {
        match self {
            LuaVariant::String(s) => s.as_str(),
            _ => panic!("LuaVariant::get_string called on {:?}", self.variant_type()),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    // -----------------------------------------------------------------------
    // LuaVariantType discriminants
    // -----------------------------------------------------------------------

    #[test]
    fn test_lua_variant_type_number_discriminant() {
        assert_eq!(LuaVariantType::Number as u8, 0);
    }

    #[test]
    fn test_lua_variant_type_position_discriminant() {
        assert_eq!(LuaVariantType::Position as u8, 1);
    }

    #[test]
    fn test_lua_variant_type_target_position_discriminant() {
        assert_eq!(LuaVariantType::TargetPosition as u8, 2);
    }

    #[test]
    fn test_lua_variant_type_string_discriminant() {
        assert_eq!(LuaVariantType::String as u8, 3);
    }

    #[test]
    fn test_lua_variant_type_none_discriminant() {
        assert_eq!(LuaVariantType::None as u8, u8::MAX);
    }

    // -----------------------------------------------------------------------
    // Default / None variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_is_none() {
        let v = LuaVariant::default();
        assert!(v.is_none());
        assert_eq!(v.variant_type(), LuaVariantType::None);
    }

    #[test]
    fn test_none_is_none() {
        let v = LuaVariant::None;
        assert!(v.is_none());
        assert!(!v.is_number());
        assert!(!v.is_position());
        assert!(!v.is_target_position());
        assert!(!v.is_string());
    }

    // -----------------------------------------------------------------------
    // Number variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_number_constructor_and_accessor() {
        let v = LuaVariant::number(42);
        assert_eq!(v.get_number(), 42);
    }

    #[test]
    fn test_number_is_number() {
        let v = LuaVariant::number(0);
        assert!(v.is_number());
        assert!(!v.is_none());
        assert!(!v.is_position());
        assert!(!v.is_target_position());
        assert!(!v.is_string());
    }

    #[test]
    fn test_number_variant_type() {
        let v = LuaVariant::number(100);
        assert_eq!(v.variant_type(), LuaVariantType::Number);
    }

    #[test]
    fn test_number_max_u32() {
        let v = LuaVariant::number(u32::MAX);
        assert_eq!(v.get_number(), u32::MAX);
    }

    #[test]
    fn test_number_zero() {
        let v = LuaVariant::number(0);
        assert_eq!(v.get_number(), 0);
    }

    // -----------------------------------------------------------------------
    // Position variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_position_constructor_and_accessor() {
        let pos = Position::new(100, 200, 7);
        let v = LuaVariant::position(pos);
        assert_eq!(v.get_position(), &pos);
    }

    #[test]
    fn test_position_is_position() {
        let v = LuaVariant::position(Position::new(1, 2, 3));
        assert!(v.is_position());
        assert!(!v.is_none());
        assert!(!v.is_number());
        assert!(!v.is_target_position());
        assert!(!v.is_string());
    }

    #[test]
    fn test_position_variant_type() {
        let v = LuaVariant::position(Position::new(10, 20, 5));
        assert_eq!(v.variant_type(), LuaVariantType::Position);
    }

    // -----------------------------------------------------------------------
    // TargetPosition variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_target_position_constructor_and_accessor() {
        let pos = Position::new(50, 60, 3);
        let v = LuaVariant::target_position(pos);
        assert_eq!(v.get_target_position(), &pos);
    }

    #[test]
    fn test_target_position_is_target_position() {
        let v = LuaVariant::target_position(Position::new(1, 2, 3));
        assert!(v.is_target_position());
        assert!(!v.is_none());
        assert!(!v.is_number());
        assert!(!v.is_position());
        assert!(!v.is_string());
    }

    #[test]
    fn test_target_position_variant_type() {
        let v = LuaVariant::target_position(Position::new(10, 20, 5));
        assert_eq!(v.variant_type(), LuaVariantType::TargetPosition);
    }

    #[test]
    fn test_position_and_target_position_different_types() {
        let pos = Position::new(1, 2, 3);
        let p = LuaVariant::position(pos);
        let tp = LuaVariant::target_position(pos);
        // Same data, different discriminants
        assert_ne!(p, tp);
        assert_eq!(p.variant_type(), LuaVariantType::Position);
        assert_eq!(tp.variant_type(), LuaVariantType::TargetPosition);
    }

    // -----------------------------------------------------------------------
    // String variant
    // -----------------------------------------------------------------------

    #[test]
    fn test_string_constructor_and_accessor() {
        let v = LuaVariant::string("hello");
        assert_eq!(v.get_string(), "hello");
    }

    #[test]
    fn test_string_is_string() {
        let v = LuaVariant::string("test");
        assert!(v.is_string());
        assert!(!v.is_none());
        assert!(!v.is_number());
        assert!(!v.is_position());
        assert!(!v.is_target_position());
    }

    #[test]
    fn test_string_variant_type() {
        let v = LuaVariant::string("abc");
        assert_eq!(v.variant_type(), LuaVariantType::String);
    }

    #[test]
    fn test_string_empty() {
        let v = LuaVariant::string("");
        assert_eq!(v.get_string(), "");
        assert!(v.is_string());
    }

    #[test]
    fn test_string_from_string_type() {
        let s = std::string::String::from("owned string");
        let v = LuaVariant::string(s.clone());
        assert_eq!(v.get_string(), s.as_str());
    }

    // -----------------------------------------------------------------------
    // Equality and Clone
    // -----------------------------------------------------------------------

    #[test]
    fn test_number_equality() {
        let a = LuaVariant::number(7);
        let b = LuaVariant::number(7);
        let c = LuaVariant::number(8);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_string_equality() {
        let a = LuaVariant::string("hello");
        let b = LuaVariant::string("hello");
        let c = LuaVariant::string("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_clone_number() {
        let a = LuaVariant::number(99);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_clone_string() {
        let a = LuaVariant::string("cloned");
        let b = a.clone();
        assert_eq!(a, b);
    }

    // -----------------------------------------------------------------------
    // Panic safety — accessors on wrong variant
    // -----------------------------------------------------------------------

    #[test]
    #[should_panic]
    fn test_get_number_on_none_panics() {
        LuaVariant::None.get_number();
    }

    #[test]
    #[should_panic]
    fn test_get_position_on_number_panics() {
        LuaVariant::number(1).get_position();
    }

    #[test]
    #[should_panic]
    fn test_get_target_position_on_string_panics() {
        LuaVariant::string("x").get_target_position();
    }

    #[test]
    #[should_panic]
    fn test_get_string_on_position_panics() {
        LuaVariant::position(Position::new(1, 2, 3)).get_string();
    }

    // -----------------------------------------------------------------------
    // Confirming tests for panic-correct stubs (panic-correct classification)
    //
    // C++ evidence: luavariant.h — getNumber/getPosition/getTargetPosition/
    // getString accessors accessed the backing std::variant via std::get<N>,
    // which is undefined behaviour on the wrong active member. Rust panics
    // instead — safer and equivalent observable contract.
    // -----------------------------------------------------------------------

    #[test]
    fn test_lua_variant_get_number_returns_value_for_number_variant() {
        // C++: LuaVariant::getNumber() — std::get<VARIANT_NUMBER>(data)
        let v = LuaVariant::number(12345);
        assert_eq!(v.get_number(), 12345);
    }

    #[test]
    #[should_panic]
    fn test_lua_variant_get_number_panics_for_non_number_variant() {
        // C++: calling getNumber() on a non-Number variant is undefined behaviour;
        // Rust panics to enforce the contract safely.
        LuaVariant::string("not a number").get_number();
    }

    #[test]
    fn test_lua_variant_get_position_returns_value_for_position_variant() {
        // C++: LuaVariant::getPosition() — std::get<VARIANT_POSITION>(data)
        let pos = Position::new(300, 400, 5);
        let v = LuaVariant::position(pos);
        assert_eq!(v.get_position(), &pos);
    }

    #[test]
    #[should_panic]
    fn test_lua_variant_get_position_panics_for_non_position_variant() {
        // C++: calling getPosition() on a non-Position variant is undefined behaviour;
        // Rust panics instead.
        LuaVariant::None.get_position();
    }

    #[test]
    fn test_lua_variant_get_target_position_returns_value_for_target_position_variant() {
        // C++: LuaVariant::getTargetPosition() — std::get<VARIANT_TARGETPOSITION>(data)
        let pos = Position::new(10, 20, 3);
        let v = LuaVariant::target_position(pos);
        assert_eq!(v.get_target_position(), &pos);
    }

    #[test]
    #[should_panic]
    fn test_lua_variant_get_target_position_panics_for_non_target_position_variant() {
        // C++: calling getTargetPosition() on a non-TargetPosition variant is
        // undefined behaviour; Rust panics instead.
        LuaVariant::number(99).get_target_position();
    }

    #[test]
    fn test_lua_variant_get_string_returns_value_for_string_variant() {
        // C++: LuaVariant::getString() — std::get<VARIANT_STRING>(data)
        let v = LuaVariant::string("hello world");
        assert_eq!(v.get_string(), "hello world");
    }

    #[test]
    #[should_panic]
    fn test_lua_variant_get_string_panics_for_non_string_variant() {
        // C++: calling getString() on a non-String variant is undefined behaviour;
        // Rust panics instead.
        LuaVariant::number(0).get_string();
    }
}
