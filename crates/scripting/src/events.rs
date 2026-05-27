use std::collections::HashMap;

// ── Player events ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerEvent {
    Login,
    Logout,
    Look,
    MoveItem,
    UseItem,
    CloseChannel,
    CombatBegin,
    CombatEnd,
    // Additional hooks from C++ events.h / player namespace
    BrowseField,
    LookInBattleList,
    LookInTrade,
    LookInShop,
    LookInMarket,
    ItemMoved,
    MoveCreature,
    ReportRuleViolation,
    ReportBug,
    RotateItem,
    Turn,
    TradeRequest,
    TradeAccept,
    TradeCompleted,
    PodiumRequest,
    PodiumEdit,
    GainExperience,
    LoseExperience,
    GainSkillTries,
    WrapItem,
    InventoryUpdate,
    NetworkMessage,
    SpellCheck,
}

// ── Monster events ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MonsterEvent {
    OnDropLoot,
    OnSpawn,
}

// ── Creature events ───────────────────────────────────────────────────────────

/// Maps to `tfs::events::creature` namespace in C++.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CreatureEvent {
    OnChangeOutfit,
    OnAreaCombat,
    OnTargetCombat,
    OnHear,
    OnChangeZone,
    OnUpdateStorage,
}

// ── Party events ──────────────────────────────────────────────────────────────

/// Maps to `tfs::events::party` namespace in C++.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PartyEvent {
    OnJoin,
    OnLeave,
    OnDisband,
    OnShareExperience,
    OnInvite,
    OnRevokeInvitation,
    OnPassLeadership,
}

// ── Callbacks registry ────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct EventsCallbacks {
    player_callbacks: HashMap<PlayerEvent, String>,
    monster_callbacks: HashMap<MonsterEvent, String>,
    creature_callbacks: HashMap<CreatureEvent, String>,
    party_callbacks: HashMap<PartyEvent, String>,
}

impl EventsCallbacks {
    pub fn new() -> Self {
        Self {
            player_callbacks: HashMap::new(),
            monster_callbacks: HashMap::new(),
            creature_callbacks: HashMap::new(),
            party_callbacks: HashMap::new(),
        }
    }

    // ── Player ────────────────────────────────────────────────────────────────

    pub fn register_player(&mut self, event: PlayerEvent, script_name: impl Into<String>) {
        self.player_callbacks.insert(event, script_name.into());
    }

    pub fn get_player_callback(&self, event: &PlayerEvent) -> Option<&str> {
        self.player_callbacks.get(event).map(String::as_str)
    }

    // ── Monster ───────────────────────────────────────────────────────────────

    pub fn register_monster(&mut self, event: MonsterEvent, script_name: impl Into<String>) {
        self.monster_callbacks.insert(event, script_name.into());
    }

    pub fn get_monster_callback(&self, event: &MonsterEvent) -> Option<&str> {
        self.monster_callbacks.get(event).map(String::as_str)
    }

    // ── Creature ──────────────────────────────────────────────────────────────

    pub fn register_creature(&mut self, event: CreatureEvent, script_name: impl Into<String>) {
        self.creature_callbacks.insert(event, script_name.into());
    }

    pub fn get_creature_callback(&self, event: &CreatureEvent) -> Option<&str> {
        self.creature_callbacks.get(event).map(String::as_str)
    }

    // ── Party ─────────────────────────────────────────────────────────────────

    pub fn register_party(&mut self, event: PartyEvent, script_name: impl Into<String>) {
        self.party_callbacks.insert(event, script_name.into());
    }

    pub fn get_party_callback(&self, event: &PartyEvent) -> Option<&str> {
        self.party_callbacks.get(event).map(String::as_str)
    }

    /// Returns the count of currently-registered callbacks across every
    /// kind. Useful as a `reload` invariant — after a full reload the
    /// total should equal the number of `enabled=true` rows in the new
    /// XML, with zero leakage from the previous load.
    pub fn total_registered(&self) -> usize {
        self.player_callbacks.len()
            + self.monster_callbacks.len()
            + self.creature_callbacks.len()
            + self.party_callbacks.len()
    }

    /// Drop every registered callback. Mirrors the C++ `tfs::events`
    /// implementation, which resets every handler struct to `-1` before
    /// re-running `load_from_xml`.
    pub fn clear(&mut self) {
        self.player_callbacks.clear();
        self.monster_callbacks.clear();
        self.creature_callbacks.clear();
        self.party_callbacks.clear();
    }

    /// Parse an `events.xml` document and register every enabled
    /// `(class, method)` pair into the matching enum. Mirrors
    /// `tfs::events::load_from_xml` in `forgottenserver/src/events.cpp`.
    ///
    /// XML shape:
    /// ```xml
    /// <events>
    ///   <event class="Creature" method="onHear" enabled="1" />
    ///   <event class="Player"   method="onLook" enabled="true" />
    ///   <event class="Party"    method="onJoin" enabled="false" />  <!-- skipped -->
    /// </events>
    /// ```
    ///
    /// `script_name` for each registered event is `"<lowercase-class>.lua"`,
    /// matching the C++ behaviour where one Lua file per class holds every
    /// method of that class. The script-file loader itself is wired by the
    /// `load_from_file` bootstrap below; this parser only populates the
    /// registry so unit tests can run without touching disk.
    ///
    /// Unknown classes / methods are silently skipped (the C++ side
    /// prints a warning to stdout, which we don't replicate here — the
    /// returned warnings vec is the audit surface).
    pub fn parse_events_xml(&mut self, xml: &str) -> Result<Vec<String>, String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;
        let root = doc
            .descendants()
            .find(|n| n.has_tag_name("events"))
            .ok_or_else(|| "Missing <events> root element".to_string())?;

        let mut warnings: Vec<String> = Vec::new();
        for node in root.children().filter(|n| n.is_element()) {
            let enabled = node
                .attribute("enabled")
                .map(parse_xml_bool)
                .unwrap_or(false);
            if !enabled {
                continue;
            }
            let class_name = node.attribute("class").unwrap_or("");
            let method_name = node.attribute("method").unwrap_or("");
            let script = format!("{}.lua", class_name.to_ascii_lowercase());

            let matched = match class_name {
                "Creature" => self.register_creature_method(method_name, &script),
                "Party" => self.register_party_method(method_name, &script),
                "Player" => self.register_player_method(method_name, &script),
                "Monster" => self.register_monster_method(method_name, &script),
                _ => {
                    warnings.push(format!(
                        "[Events::parse_events_xml] Unknown class: {class_name}"
                    ));
                    continue;
                }
            };
            if !matched {
                warnings.push(format!(
                    "[Events::parse_events_xml] Unknown {class_name} method: {method_name}"
                ));
            }
        }
        Ok(warnings)
    }

    fn register_creature_method(&mut self, method: &str, script: &str) -> bool {
        let event = match method {
            "onChangeOutfit" => CreatureEvent::OnChangeOutfit,
            "onAreaCombat" => CreatureEvent::OnAreaCombat,
            "onTargetCombat" => CreatureEvent::OnTargetCombat,
            "onHear" => CreatureEvent::OnHear,
            "onChangeZone" => CreatureEvent::OnChangeZone,
            "onUpdateStorage" => CreatureEvent::OnUpdateStorage,
            _ => return false,
        };
        self.register_creature(event, script);
        true
    }

    fn register_party_method(&mut self, method: &str, script: &str) -> bool {
        let event = match method {
            "onJoin" => PartyEvent::OnJoin,
            "onLeave" => PartyEvent::OnLeave,
            "onDisband" => PartyEvent::OnDisband,
            "onShareExperience" => PartyEvent::OnShareExperience,
            "onInvite" => PartyEvent::OnInvite,
            "onRevokeInvitation" => PartyEvent::OnRevokeInvitation,
            "onPassLeadership" => PartyEvent::OnPassLeadership,
            _ => return false,
        };
        self.register_party(event, script);
        true
    }

    fn register_player_method(&mut self, method: &str, script: &str) -> bool {
        let event = match method {
            "onBrowseField" => PlayerEvent::BrowseField,
            "onLook" => PlayerEvent::Look,
            "onLookInBattleList" => PlayerEvent::LookInBattleList,
            "onLookInTrade" => PlayerEvent::LookInTrade,
            "onLookInShop" => PlayerEvent::LookInShop,
            "onLookInMarket" => PlayerEvent::LookInMarket,
            "onTradeRequest" => PlayerEvent::TradeRequest,
            "onTradeAccept" => PlayerEvent::TradeAccept,
            "onTradeCompleted" => PlayerEvent::TradeCompleted,
            "onPodiumRequest" => PlayerEvent::PodiumRequest,
            "onPodiumEdit" => PlayerEvent::PodiumEdit,
            "onMoveItem" => PlayerEvent::MoveItem,
            "onItemMoved" => PlayerEvent::ItemMoved,
            "onMoveCreature" => PlayerEvent::MoveCreature,
            "onReportRuleViolation" => PlayerEvent::ReportRuleViolation,
            "onReportBug" => PlayerEvent::ReportBug,
            "onRotateItem" => PlayerEvent::RotateItem,
            "onTurn" => PlayerEvent::Turn,
            "onGainExperience" => PlayerEvent::GainExperience,
            "onLoseExperience" => PlayerEvent::LoseExperience,
            "onGainSkillTries" => PlayerEvent::GainSkillTries,
            "onWrapItem" => PlayerEvent::WrapItem,
            "onInventoryUpdate" => PlayerEvent::InventoryUpdate,
            "onNetworkMessage" => PlayerEvent::NetworkMessage,
            "onSpellCheck" => PlayerEvent::SpellCheck,
            _ => return false,
        };
        self.register_player(event, script);
        true
    }

    fn register_monster_method(&mut self, method: &str, script: &str) -> bool {
        let event = match method {
            "onDropLoot" => MonsterEvent::OnDropLoot,
            "onSpawn" => MonsterEvent::OnSpawn,
            _ => return false,
        };
        self.register_monster(event, script);
        true
    }

    /// Bootstrap wrapper. Mirrors `tfs::events::load()`: read the
    /// `events.xml` file at `path`, populate the registry, return parse
    /// warnings (unknown class/method names) for the caller to log.
    /// Errors only on outright filesystem / XML failures.
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<Vec<String>, String> {
        let xml = std::fs::read_to_string(path)
            .map_err(|e| format!("Events::load_from_file({:?}): {e}", path))?;
        self.parse_events_xml(&xml)
    }

    /// Bootstrap wrapper. Mirrors `tfs::events::reload()`: drop the
    /// previous registry contents, then re-parse the file. Used by the
    /// `/reload events` admin command so script edits take effect
    /// without a server restart.
    pub fn reload_from_file(&mut self, path: &std::path::Path) -> Result<Vec<String>, String> {
        self.clear();
        self.load_from_file(path)
    }
}

/// Parse the `enabled="..."` attribute value to a bool. Mirrors C++
/// `pugi::xml_attribute::as_bool()`: case-insensitive `"true"` / `"1"` →
/// true, everything else (including missing values handled by the caller)
/// → false.
fn parse_xml_bool(s: &str) -> bool {
    matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── PlayerEvent enum variants ─────────────────────────────────────────────

    #[test]
    fn player_event_enum_variants_exist() {
        let _ = PlayerEvent::Login;
        let _ = PlayerEvent::Logout;
        let _ = PlayerEvent::Look;
        let _ = PlayerEvent::MoveItem;
        let _ = PlayerEvent::UseItem;
        let _ = PlayerEvent::CloseChannel;
        let _ = PlayerEvent::CombatBegin;
        let _ = PlayerEvent::CombatEnd;
    }

    #[test]
    fn player_event_extended_variants_exist() {
        let _ = PlayerEvent::BrowseField;
        let _ = PlayerEvent::LookInBattleList;
        let _ = PlayerEvent::LookInTrade;
        let _ = PlayerEvent::LookInShop;
        let _ = PlayerEvent::LookInMarket;
        let _ = PlayerEvent::ItemMoved;
        let _ = PlayerEvent::MoveCreature;
        let _ = PlayerEvent::ReportRuleViolation;
        let _ = PlayerEvent::ReportBug;
        let _ = PlayerEvent::RotateItem;
        let _ = PlayerEvent::Turn;
        let _ = PlayerEvent::TradeRequest;
        let _ = PlayerEvent::TradeAccept;
        let _ = PlayerEvent::TradeCompleted;
        let _ = PlayerEvent::PodiumRequest;
        let _ = PlayerEvent::PodiumEdit;
        let _ = PlayerEvent::GainExperience;
        let _ = PlayerEvent::LoseExperience;
        let _ = PlayerEvent::GainSkillTries;
        let _ = PlayerEvent::WrapItem;
        let _ = PlayerEvent::InventoryUpdate;
        let _ = PlayerEvent::NetworkMessage;
        let _ = PlayerEvent::SpellCheck;
    }

    // ── MonsterEvent enum variants ────────────────────────────────────────────

    #[test]
    fn monster_event_enum_variants_exist() {
        let _ = MonsterEvent::OnDropLoot;
        let _ = MonsterEvent::OnSpawn;
    }

    // ── CreatureEvent enum variants ───────────────────────────────────────────

    #[test]
    fn creature_event_enum_variants_exist() {
        let _ = CreatureEvent::OnChangeOutfit;
        let _ = CreatureEvent::OnAreaCombat;
        let _ = CreatureEvent::OnTargetCombat;
        let _ = CreatureEvent::OnHear;
        let _ = CreatureEvent::OnChangeZone;
        let _ = CreatureEvent::OnUpdateStorage;
    }

    // ── PartyEvent enum variants ──────────────────────────────────────────────

    #[test]
    fn party_event_enum_variants_exist() {
        let _ = PartyEvent::OnJoin;
        let _ = PartyEvent::OnLeave;
        let _ = PartyEvent::OnDisband;
        let _ = PartyEvent::OnShareExperience;
        let _ = PartyEvent::OnInvite;
        let _ = PartyEvent::OnRevokeInvitation;
        let _ = PartyEvent::OnPassLeadership;
    }

    // ── EventsCallbacks construction ──────────────────────────────────────────

    #[test]
    fn events_callbacks_new_creates_empty_registry() {
        let cb = EventsCallbacks::new();
        assert!(cb.get_player_callback(&PlayerEvent::Login).is_none());
        assert!(cb.get_monster_callback(&MonsterEvent::OnDropLoot).is_none());
        assert!(cb.get_creature_callback(&CreatureEvent::OnHear).is_none());
        assert!(cb.get_party_callback(&PartyEvent::OnJoin).is_none());
    }

    // ── Player callback registration ──────────────────────────────────────────

    #[test]
    fn register_player_callback_for_login() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::Login, "player_login.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::Login),
            Some("player_login.lua")
        );
    }

    #[test]
    fn register_player_callback_for_logout() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::Logout, "player_logout.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::Logout),
            Some("player_logout.lua")
        );
    }

    #[test]
    fn register_player_callback_for_look() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::Look, "look.lua");
        assert_eq!(cb.get_player_callback(&PlayerEvent::Look), Some("look.lua"));
    }

    #[test]
    fn register_player_callback_for_move_item() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::MoveItem, "move.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::MoveItem),
            Some("move.lua")
        );
    }

    #[test]
    fn register_player_callback_for_use_item() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::UseItem, "use.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::UseItem),
            Some("use.lua")
        );
    }

    #[test]
    fn register_player_callback_for_close_channel() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::CloseChannel, "close.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::CloseChannel),
            Some("close.lua")
        );
    }

    #[test]
    fn register_player_callback_for_combat_begin() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::CombatBegin, "combat_begin.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::CombatBegin),
            Some("combat_begin.lua")
        );
    }

    #[test]
    fn register_player_callback_for_combat_end() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::CombatEnd, "combat_end.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::CombatEnd),
            Some("combat_end.lua")
        );
    }

    #[test]
    fn register_player_callback_for_browse_field() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::BrowseField, "browse_field.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::BrowseField),
            Some("browse_field.lua")
        );
    }

    #[test]
    fn register_player_callback_for_look_in_battle_list() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::LookInBattleList, "battle_list.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInBattleList),
            Some("battle_list.lua")
        );
    }

    #[test]
    fn register_player_callback_for_look_in_trade() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::LookInTrade, "look_in_trade.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInTrade),
            Some("look_in_trade.lua")
        );
    }

    #[test]
    fn register_player_callback_for_look_in_shop() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::LookInShop, "look_in_shop.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInShop),
            Some("look_in_shop.lua")
        );
    }

    #[test]
    fn register_player_callback_for_look_in_market() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::LookInMarket, "look_in_market.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInMarket),
            Some("look_in_market.lua")
        );
    }

    #[test]
    fn register_player_callback_for_item_moved() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::ItemMoved, "item_moved.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::ItemMoved),
            Some("item_moved.lua")
        );
    }

    #[test]
    fn register_player_callback_for_move_creature() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::MoveCreature, "move_creature.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::MoveCreature),
            Some("move_creature.lua")
        );
    }

    #[test]
    fn register_player_callback_for_report_rule_violation() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::ReportRuleViolation, "report_rule.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::ReportRuleViolation),
            Some("report_rule.lua")
        );
    }

    #[test]
    fn register_player_callback_for_report_bug() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::ReportBug, "report_bug.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::ReportBug),
            Some("report_bug.lua")
        );
    }

    #[test]
    fn register_player_callback_for_rotate_item() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::RotateItem, "rotate_item.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::RotateItem),
            Some("rotate_item.lua")
        );
    }

    #[test]
    fn register_player_callback_for_turn() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::Turn, "turn.lua");
        assert_eq!(cb.get_player_callback(&PlayerEvent::Turn), Some("turn.lua"));
    }

    #[test]
    fn register_player_callback_for_trade_request() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::TradeRequest, "trade_request.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::TradeRequest),
            Some("trade_request.lua")
        );
    }

    #[test]
    fn register_player_callback_for_trade_accept() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::TradeAccept, "trade_accept.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::TradeAccept),
            Some("trade_accept.lua")
        );
    }

    #[test]
    fn register_player_callback_for_trade_completed() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::TradeCompleted, "trade_completed.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::TradeCompleted),
            Some("trade_completed.lua")
        );
    }

    #[test]
    fn register_player_callback_for_podium_request() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::PodiumRequest, "podium_request.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::PodiumRequest),
            Some("podium_request.lua")
        );
    }

    #[test]
    fn register_player_callback_for_podium_edit() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::PodiumEdit, "podium_edit.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::PodiumEdit),
            Some("podium_edit.lua")
        );
    }

    #[test]
    fn register_player_callback_for_gain_experience() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::GainExperience, "gain_exp.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::GainExperience),
            Some("gain_exp.lua")
        );
    }

    #[test]
    fn register_player_callback_for_lose_experience() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::LoseExperience, "lose_exp.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LoseExperience),
            Some("lose_exp.lua")
        );
    }

    #[test]
    fn register_player_callback_for_gain_skill_tries() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::GainSkillTries, "gain_skill.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::GainSkillTries),
            Some("gain_skill.lua")
        );
    }

    #[test]
    fn register_player_callback_for_wrap_item() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::WrapItem, "wrap_item.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::WrapItem),
            Some("wrap_item.lua")
        );
    }

    #[test]
    fn register_player_callback_for_inventory_update() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::InventoryUpdate, "inv_update.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::InventoryUpdate),
            Some("inv_update.lua")
        );
    }

    #[test]
    fn register_player_callback_for_network_message() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::NetworkMessage, "net_msg.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::NetworkMessage),
            Some("net_msg.lua")
        );
    }

    #[test]
    fn register_player_callback_for_spell_check() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::SpellCheck, "spell_check.lua");
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::SpellCheck),
            Some("spell_check.lua")
        );
    }

    #[test]
    fn get_player_callback_returns_none_for_unregistered() {
        let cb = EventsCallbacks::new();
        assert!(cb.get_player_callback(&PlayerEvent::Login).is_none());
    }

    // ── Monster callback registration ─────────────────────────────────────────

    #[test]
    fn register_monster_on_drop_loot() {
        let mut cb = EventsCallbacks::new();
        cb.register_monster(MonsterEvent::OnDropLoot, "drop_loot.lua");
        assert_eq!(
            cb.get_monster_callback(&MonsterEvent::OnDropLoot),
            Some("drop_loot.lua")
        );
    }

    #[test]
    fn register_monster_on_spawn() {
        let mut cb = EventsCallbacks::new();
        cb.register_monster(MonsterEvent::OnSpawn, "on_spawn.lua");
        assert_eq!(
            cb.get_monster_callback(&MonsterEvent::OnSpawn),
            Some("on_spawn.lua")
        );
    }

    #[test]
    fn get_monster_callback_returns_none_for_unregistered() {
        let cb = EventsCallbacks::new();
        assert!(cb.get_monster_callback(&MonsterEvent::OnDropLoot).is_none());
    }

    // ── Creature callback registration ────────────────────────────────────────

    #[test]
    fn register_creature_on_change_outfit() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnChangeOutfit, "outfit.lua");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnChangeOutfit),
            Some("outfit.lua")
        );
    }

    #[test]
    fn register_creature_on_area_combat() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnAreaCombat, "area_combat.lua");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnAreaCombat),
            Some("area_combat.lua")
        );
    }

    #[test]
    fn register_creature_on_target_combat() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnTargetCombat, "target_combat.lua");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnTargetCombat),
            Some("target_combat.lua")
        );
    }

    #[test]
    fn register_creature_on_hear() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnHear, "on_hear.lua");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnHear),
            Some("on_hear.lua")
        );
    }

    #[test]
    fn register_creature_on_change_zone() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnChangeZone, "change_zone.lua");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnChangeZone),
            Some("change_zone.lua")
        );
    }

    #[test]
    fn register_creature_on_update_storage() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnUpdateStorage, "update_storage.lua");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnUpdateStorage),
            Some("update_storage.lua")
        );
    }

    #[test]
    fn get_creature_callback_returns_none_for_unregistered() {
        let cb = EventsCallbacks::new();
        assert!(cb.get_creature_callback(&CreatureEvent::OnHear).is_none());
    }

    #[test]
    fn all_creature_events_can_be_registered_independently() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnChangeOutfit, "outfit.lua");
        cb.register_creature(CreatureEvent::OnAreaCombat, "area_combat.lua");
        cb.register_creature(CreatureEvent::OnTargetCombat, "target_combat.lua");
        cb.register_creature(CreatureEvent::OnHear, "hear.lua");
        cb.register_creature(CreatureEvent::OnChangeZone, "zone.lua");
        cb.register_creature(CreatureEvent::OnUpdateStorage, "storage.lua");

        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnChangeOutfit),
            Some("outfit.lua")
        );
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnAreaCombat),
            Some("area_combat.lua")
        );
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnTargetCombat),
            Some("target_combat.lua")
        );
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnHear),
            Some("hear.lua")
        );
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnChangeZone),
            Some("zone.lua")
        );
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnUpdateStorage),
            Some("storage.lua")
        );
    }

    // ── Party callback registration ───────────────────────────────────────────

    #[test]
    fn register_party_on_join() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnJoin, "party_join.lua");
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnJoin),
            Some("party_join.lua")
        );
    }

    #[test]
    fn register_party_on_leave() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnLeave, "party_leave.lua");
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnLeave),
            Some("party_leave.lua")
        );
    }

    #[test]
    fn register_party_on_disband() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnDisband, "party_disband.lua");
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnDisband),
            Some("party_disband.lua")
        );
    }

    #[test]
    fn register_party_on_share_experience() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnShareExperience, "share_exp.lua");
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnShareExperience),
            Some("share_exp.lua")
        );
    }

    #[test]
    fn register_party_on_invite() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnInvite, "party_invite.lua");
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnInvite),
            Some("party_invite.lua")
        );
    }

    #[test]
    fn register_party_on_revoke_invitation() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnRevokeInvitation, "revoke.lua");
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnRevokeInvitation),
            Some("revoke.lua")
        );
    }

    #[test]
    fn register_party_on_pass_leadership() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnPassLeadership, "pass_lead.lua");
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnPassLeadership),
            Some("pass_lead.lua")
        );
    }

    #[test]
    fn get_party_callback_returns_none_for_unregistered() {
        let cb = EventsCallbacks::new();
        assert!(cb.get_party_callback(&PartyEvent::OnJoin).is_none());
    }

    #[test]
    fn all_party_events_can_be_registered_independently() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnJoin, "join.lua");
        cb.register_party(PartyEvent::OnLeave, "leave.lua");
        cb.register_party(PartyEvent::OnDisband, "disband.lua");
        cb.register_party(PartyEvent::OnShareExperience, "share.lua");
        cb.register_party(PartyEvent::OnInvite, "invite.lua");
        cb.register_party(PartyEvent::OnRevokeInvitation, "revoke.lua");
        cb.register_party(PartyEvent::OnPassLeadership, "lead.lua");

        assert_eq!(cb.get_party_callback(&PartyEvent::OnJoin), Some("join.lua"));
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnLeave),
            Some("leave.lua")
        );
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnDisband),
            Some("disband.lua")
        );
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnShareExperience),
            Some("share.lua")
        );
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnInvite),
            Some("invite.lua")
        );
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnRevokeInvitation),
            Some("revoke.lua")
        );
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnPassLeadership),
            Some("lead.lua")
        );
    }

    // ── Cross-namespace isolation ─────────────────────────────────────────────

    #[test]
    fn registering_player_event_does_not_affect_creature_callbacks() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::Login, "login.lua");
        assert!(cb.get_creature_callback(&CreatureEvent::OnHear).is_none());
        assert!(cb.get_party_callback(&PartyEvent::OnJoin).is_none());
        assert!(cb.get_monster_callback(&MonsterEvent::OnSpawn).is_none());
    }

    #[test]
    fn registering_creature_event_does_not_affect_player_callbacks() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnHear, "hear.lua");
        assert!(cb.get_player_callback(&PlayerEvent::Login).is_none());
        assert!(cb.get_party_callback(&PartyEvent::OnJoin).is_none());
        assert!(cb.get_monster_callback(&MonsterEvent::OnSpawn).is_none());
    }

    #[test]
    fn registering_party_event_does_not_affect_other_callbacks() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnJoin, "join.lua");
        assert!(cb.get_player_callback(&PlayerEvent::Login).is_none());
        assert!(cb.get_creature_callback(&CreatureEvent::OnHear).is_none());
        assert!(cb.get_monster_callback(&MonsterEvent::OnSpawn).is_none());
    }

    // ── Re-registration overwrites ────────────────────────────────────────────

    #[test]
    fn re_registering_player_event_overwrites_previous() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::Login, "old.lua");
        cb.register_player(PlayerEvent::Login, "new.lua");
        assert_eq!(cb.get_player_callback(&PlayerEvent::Login), Some("new.lua"));
    }

    #[test]
    fn re_registering_creature_event_overwrites_previous() {
        let mut cb = EventsCallbacks::new();
        cb.register_creature(CreatureEvent::OnHear, "old.lua");
        cb.register_creature(CreatureEvent::OnHear, "new.lua");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnHear),
            Some("new.lua")
        );
    }

    #[test]
    fn re_registering_party_event_overwrites_previous() {
        let mut cb = EventsCallbacks::new();
        cb.register_party(PartyEvent::OnJoin, "old.lua");
        cb.register_party(PartyEvent::OnJoin, "new.lua");
        assert_eq!(cb.get_party_callback(&PartyEvent::OnJoin), Some("new.lua"));
    }

    #[test]
    fn re_registering_monster_event_overwrites_previous() {
        let mut cb = EventsCallbacks::new();
        cb.register_monster(MonsterEvent::OnDropLoot, "old.lua");
        cb.register_monster(MonsterEvent::OnDropLoot, "new.lua");
        assert_eq!(
            cb.get_monster_callback(&MonsterEvent::OnDropLoot),
            Some("new.lua")
        );
    }

    // ── Bulk registration tests (preserved from before) ───────────────────────

    #[test]
    fn all_player_events_can_be_registered_independently() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::Login, "login.lua");
        cb.register_player(PlayerEvent::Logout, "logout.lua");
        cb.register_player(PlayerEvent::Look, "look.lua");
        cb.register_player(PlayerEvent::MoveItem, "move.lua");
        cb.register_player(PlayerEvent::UseItem, "use.lua");
        cb.register_player(PlayerEvent::CloseChannel, "close.lua");
        cb.register_player(PlayerEvent::CombatBegin, "begin.lua");
        cb.register_player(PlayerEvent::CombatEnd, "end.lua");

        assert_eq!(
            cb.get_player_callback(&PlayerEvent::Login),
            Some("login.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::Logout),
            Some("logout.lua")
        );
        assert_eq!(cb.get_player_callback(&PlayerEvent::Look), Some("look.lua"));
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::MoveItem),
            Some("move.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::UseItem),
            Some("use.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::CloseChannel),
            Some("close.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::CombatBegin),
            Some("begin.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::CombatEnd),
            Some("end.lua")
        );
    }

    #[test]
    fn all_monster_events_can_be_registered_independently() {
        let mut cb = EventsCallbacks::new();
        cb.register_monster(MonsterEvent::OnDropLoot, "loot.lua");
        cb.register_monster(MonsterEvent::OnSpawn, "spawn.lua");

        assert_eq!(
            cb.get_monster_callback(&MonsterEvent::OnDropLoot),
            Some("loot.lua")
        );
        assert_eq!(
            cb.get_monster_callback(&MonsterEvent::OnSpawn),
            Some("spawn.lua")
        );
    }

    #[test]
    fn all_extended_player_events_can_be_registered_independently() {
        let mut cb = EventsCallbacks::new();
        cb.register_player(PlayerEvent::BrowseField, "browse.lua");
        cb.register_player(PlayerEvent::LookInBattleList, "battle.lua");
        cb.register_player(PlayerEvent::LookInTrade, "trade.lua");
        cb.register_player(PlayerEvent::LookInShop, "shop.lua");
        cb.register_player(PlayerEvent::LookInMarket, "market.lua");
        cb.register_player(PlayerEvent::ItemMoved, "item_moved.lua");
        cb.register_player(PlayerEvent::MoveCreature, "move_creature.lua");
        cb.register_player(PlayerEvent::ReportRuleViolation, "rule.lua");
        cb.register_player(PlayerEvent::ReportBug, "bug.lua");
        cb.register_player(PlayerEvent::RotateItem, "rotate.lua");
        cb.register_player(PlayerEvent::Turn, "turn.lua");
        cb.register_player(PlayerEvent::TradeRequest, "trade_req.lua");
        cb.register_player(PlayerEvent::TradeAccept, "trade_acc.lua");
        cb.register_player(PlayerEvent::TradeCompleted, "trade_done.lua");
        cb.register_player(PlayerEvent::PodiumRequest, "podium_req.lua");
        cb.register_player(PlayerEvent::PodiumEdit, "podium_edit.lua");
        cb.register_player(PlayerEvent::GainExperience, "gain_exp.lua");
        cb.register_player(PlayerEvent::LoseExperience, "lose_exp.lua");
        cb.register_player(PlayerEvent::GainSkillTries, "gain_skill.lua");
        cb.register_player(PlayerEvent::WrapItem, "wrap.lua");
        cb.register_player(PlayerEvent::InventoryUpdate, "inv.lua");
        cb.register_player(PlayerEvent::NetworkMessage, "net.lua");
        cb.register_player(PlayerEvent::SpellCheck, "spell.lua");

        assert_eq!(
            cb.get_player_callback(&PlayerEvent::BrowseField),
            Some("browse.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInBattleList),
            Some("battle.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInTrade),
            Some("trade.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInShop),
            Some("shop.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LookInMarket),
            Some("market.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::ItemMoved),
            Some("item_moved.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::MoveCreature),
            Some("move_creature.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::ReportRuleViolation),
            Some("rule.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::ReportBug),
            Some("bug.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::RotateItem),
            Some("rotate.lua")
        );
        assert_eq!(cb.get_player_callback(&PlayerEvent::Turn), Some("turn.lua"));
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::TradeRequest),
            Some("trade_req.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::TradeAccept),
            Some("trade_acc.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::TradeCompleted),
            Some("trade_done.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::PodiumRequest),
            Some("podium_req.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::PodiumEdit),
            Some("podium_edit.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::GainExperience),
            Some("gain_exp.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::LoseExperience),
            Some("lose_exp.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::GainSkillTries),
            Some("gain_skill.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::WrapItem),
            Some("wrap.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::InventoryUpdate),
            Some("inv.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::NetworkMessage),
            Some("net.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::SpellCheck),
            Some("spell.lua")
        );
    }

    // ── parse_events_xml + load/reload (Session 21) ─────────────────────

    /// Valid XML across all four classes registers each `(class, method)`
    /// pair under the matching enum, with the script name derived from
    /// the lowercased class name.
    #[test]
    fn parse_events_xml_registers_each_class_method_pair() {
        let xml = r#"<events>
            <event class="Creature" method="onHear" enabled="1" />
            <event class="Party"    method="onJoin" enabled="1" />
            <event class="Player"   method="onLook" enabled="1" />
            <event class="Monster"  method="onSpawn" enabled="1" />
        </events>"#;
        let mut cb = EventsCallbacks::new();
        let warnings = cb.parse_events_xml(xml).unwrap();
        assert!(warnings.is_empty(), "no warnings: {warnings:?}");
        assert_eq!(
            cb.get_creature_callback(&CreatureEvent::OnHear),
            Some("creature.lua")
        );
        assert_eq!(
            cb.get_party_callback(&PartyEvent::OnJoin),
            Some("party.lua")
        );
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::Look),
            Some("player.lua")
        );
        assert_eq!(
            cb.get_monster_callback(&MonsterEvent::OnSpawn),
            Some("monster.lua")
        );
        assert_eq!(cb.total_registered(), 4);
    }

    /// `enabled="false"` (or missing) skips the row entirely.
    #[test]
    fn parse_events_xml_skips_disabled_rows() {
        let xml = r#"<events>
            <event class="Creature" method="onHear" enabled="false" />
            <event class="Party"    method="onJoin" />
            <event class="Player"   method="onLook" enabled="true" />
        </events>"#;
        let mut cb = EventsCallbacks::new();
        let warnings = cb.parse_events_xml(xml).unwrap();
        assert!(warnings.is_empty());
        assert!(cb.get_creature_callback(&CreatureEvent::OnHear).is_none());
        assert!(cb.get_party_callback(&PartyEvent::OnJoin).is_none());
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::Look),
            Some("player.lua")
        );
        assert_eq!(cb.total_registered(), 1);
    }

    /// Unknown class returns a warning, never panics.
    #[test]
    fn parse_events_xml_unknown_class_emits_warning() {
        let xml = r#"<events>
            <event class="Quest" method="onAccept" enabled="1" />
        </events>"#;
        let mut cb = EventsCallbacks::new();
        let warnings = cb.parse_events_xml(xml).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].contains("Unknown class: Quest"),
            "{}",
            warnings[0]
        );
        assert_eq!(cb.total_registered(), 0);
    }

    /// Unknown method on a known class returns a warning.
    #[test]
    fn parse_events_xml_unknown_method_emits_warning() {
        let xml = r#"<events>
            <event class="Creature" method="onTeleport" enabled="1" />
        </events>"#;
        let mut cb = EventsCallbacks::new();
        let warnings = cb.parse_events_xml(xml).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].contains("Unknown Creature method: onTeleport"),
            "{}",
            warnings[0]
        );
    }

    /// Malformed XML surfaces a parse error.
    #[test]
    fn parse_events_xml_malformed_returns_err() {
        let mut cb = EventsCallbacks::new();
        let result = cb.parse_events_xml("<events><event"); // truncated
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("XML parse error"));
    }

    /// Missing `<events>` root surfaces a typed error.
    #[test]
    fn parse_events_xml_missing_root_returns_err() {
        let mut cb = EventsCallbacks::new();
        let result = cb.parse_events_xml("<other-root/>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing <events>"));
    }

    /// `load_from_file` round-trip via a tempfile.
    #[test]
    fn load_from_file_reads_xml_and_registers() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.xml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"<events>
                <event class="Player" method="onLook" enabled="1" />
            </events>"#
        )
        .unwrap();
        let mut cb = EventsCallbacks::new();
        let warnings = cb.load_from_file(&path).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(
            cb.get_player_callback(&PlayerEvent::Look),
            Some("player.lua")
        );
    }

    /// `reload_from_file` drops any previous registry contents — even
    /// callbacks whose `(class, method)` pair doesn't appear in the
    /// reloaded XML must be gone.
    #[test]
    fn reload_from_file_clears_previous_registry() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.xml");
        let mut cb = EventsCallbacks::new();
        // Manually register a callback that will NOT be in the reloaded XML.
        cb.register_party(PartyEvent::OnDisband, "stale.lua");
        assert!(cb.get_party_callback(&PartyEvent::OnDisband).is_some());
        // Reload with XML that registers something else.
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"<events>
                <event class="Monster" method="onDropLoot" enabled="1" />
            </events>"#
        )
        .unwrap();
        cb.reload_from_file(&path).unwrap();
        // Stale entry must be gone.
        assert!(cb.get_party_callback(&PartyEvent::OnDisband).is_none());
        // New entry is present.
        assert_eq!(
            cb.get_monster_callback(&MonsterEvent::OnDropLoot),
            Some("monster.lua")
        );
    }
}
