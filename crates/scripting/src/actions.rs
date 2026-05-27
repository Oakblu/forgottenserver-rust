use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    UseItem,
    UseItemOn,
    UsePosition,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub script_name: String,
    pub item_id: Option<u16>,
    pub unique_id: Option<u16>,
    pub action_id: Option<u16>,
    pub action_type: ActionType,
}

impl Action {
    pub fn new(script_name: impl Into<String>, action_type: ActionType) -> Self {
        Self {
            script_name: script_name.into(),
            item_id: None,
            unique_id: None,
            action_id: None,
            action_type,
        }
    }
}

#[derive(Debug, Default)]
pub struct Actions {
    by_item_id: HashMap<u16, Action>,
    by_unique_id: HashMap<u16, Action>,
    by_action_id: HashMap<u16, Action>,
    default_action: Option<Action>,
}

impl Actions {
    pub fn new() -> Self {
        Self {
            by_item_id: HashMap::new(),
            by_unique_id: HashMap::new(),
            by_action_id: HashMap::new(),
            default_action: None,
        }
    }

    pub fn register_by_item_id(&mut self, item_id: u16, action: Action) {
        self.by_item_id.insert(item_id, action);
    }

    pub fn register_by_unique_id(&mut self, uid: u16, action: Action) {
        self.by_unique_id.insert(uid, action);
    }

    /// Register an action keyed by `actionId` — mirrors C++ `Actions::addActionId`
    /// / `actionItemMap` dispatch path.
    pub fn register_by_action_id(&mut self, aid: u16, action: Action) {
        self.by_action_id.insert(aid, action);
    }

    pub fn get_by_item_id(&self, item_id: u16) -> Option<&Action> {
        self.by_item_id.get(&item_id)
    }

    pub fn get_by_unique_id(&self, uid: u16) -> Option<&Action> {
        self.by_unique_id.get(&uid)
    }

    /// Look up an action by `actionId` — mirrors C++ `Actions::getAction` second
    /// dispatch branch (`actionItemMap.find(item->getActionId())`).
    pub fn get_by_action_id(&self, aid: u16) -> Option<&Action> {
        self.by_action_id.get(&aid)
    }

    pub fn get_default_action(&self) -> Option<&Action> {
        self.default_action.as_ref()
    }

    pub fn set_default_action(&mut self, action: Action) {
        self.default_action = Some(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_action(name: &str) -> Action {
        Action::new(name, ActionType::UseItem)
    }

    #[test]
    fn action_type_enum_variants_exist() {
        let _ = ActionType::UseItem;
        let _ = ActionType::UseItemOn;
        let _ = ActionType::UsePosition;
    }

    #[test]
    fn action_struct_fields() {
        let action = Action {
            script_name: "test".to_string(),
            item_id: Some(100),
            unique_id: Some(200),
            action_id: Some(300),
            action_type: ActionType::UseItemOn,
        };
        assert_eq!(action.script_name, "test");
        assert_eq!(action.item_id, Some(100));
        assert_eq!(action.unique_id, Some(200));
        assert_eq!(action.action_id, Some(300));
        assert_eq!(action.action_type, ActionType::UseItemOn);
    }

    #[test]
    fn actions_new_creates_empty_registry() {
        let actions = Actions::new();
        assert!(actions.get_by_item_id(1).is_none());
        assert!(actions.get_by_unique_id(1).is_none());
        assert!(actions.get_default_action().is_none());
    }

    #[test]
    fn register_by_item_id_adds_action() {
        let mut actions = Actions::new();
        actions.register_by_item_id(42, make_action("door"));
        assert!(actions.get_by_item_id(42).is_some());
    }

    #[test]
    fn register_by_unique_id_adds_action() {
        let mut actions = Actions::new();
        actions.register_by_unique_id(99, make_action("lever"));
        assert!(actions.get_by_unique_id(99).is_some());
    }

    #[test]
    fn get_by_item_id_returns_some_when_registered() {
        let mut actions = Actions::new();
        actions.register_by_item_id(10, make_action("open_door"));
        let a = actions.get_by_item_id(10).unwrap();
        assert_eq!(a.script_name, "open_door");
    }

    #[test]
    fn get_by_item_id_returns_none_when_not_registered() {
        let actions = Actions::new();
        assert!(actions.get_by_item_id(999).is_none());
    }

    #[test]
    fn get_by_unique_id_returns_some_when_registered() {
        let mut actions = Actions::new();
        actions.register_by_unique_id(5, make_action("chest"));
        let a = actions.get_by_unique_id(5).unwrap();
        assert_eq!(a.script_name, "chest");
    }

    #[test]
    fn get_by_unique_id_returns_none_when_not_registered() {
        let actions = Actions::new();
        assert!(actions.get_by_unique_id(999).is_none());
    }

    #[test]
    fn get_default_action_returns_none_when_not_set() {
        let actions = Actions::new();
        assert!(actions.get_default_action().is_none());
    }

    #[test]
    fn set_default_action_then_get_returns_some() {
        let mut actions = Actions::new();
        actions.set_default_action(make_action("fallback"));
        let a = actions.get_default_action().unwrap();
        assert_eq!(a.script_name, "fallback");
    }

    #[test]
    fn register_by_action_id_adds_action() {
        let mut actions = Actions::new();
        actions.register_by_action_id(2001, make_action("aid_handler"));
        assert!(actions.get_by_action_id(2001).is_some());
    }

    #[test]
    fn get_by_action_id_returns_some_when_registered() {
        let mut actions = Actions::new();
        actions.register_by_action_id(7, make_action("quest_chest"));
        let a = actions.get_by_action_id(7).unwrap();
        assert_eq!(a.script_name, "quest_chest");
    }

    #[test]
    fn get_by_action_id_returns_none_when_not_registered() {
        let actions = Actions::new();
        assert!(actions.get_by_action_id(999).is_none());
    }

    #[test]
    fn action_id_dispatch_is_independent_of_other_maps() {
        // Mirrors C++ Actions::getAction: uniqueId / actionId / itemId are
        // three independent dispatch paths; an aid match must not be served
        // from the item_id or unique_id map.
        let mut actions = Actions::new();
        actions.register_by_item_id(50, make_action("by_item"));
        actions.register_by_unique_id(50, make_action("by_uid"));
        actions.register_by_action_id(50, make_action("by_aid"));
        assert_eq!(actions.get_by_item_id(50).unwrap().script_name, "by_item");
        assert_eq!(actions.get_by_unique_id(50).unwrap().script_name, "by_uid");
        assert_eq!(actions.get_by_action_id(50).unwrap().script_name, "by_aid");
    }
}
