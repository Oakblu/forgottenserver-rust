//! XML parser for `data/actions/actions.xml`.
//!
//! Mirrors C++ `Actions::loadFromXml` / `Action::configureEvent`.

/// A single row parsed from an `<action>` element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedActionRow {
    /// Item IDs covered by this action (empty if keyed only by uniqueid/actionid).
    /// For `itemid="X"` → `[X]`. For `fromid="A" toid="B"` → `A..=B` inclusive.
    pub item_ids: Vec<u16>,
    /// `uniqueid` attribute, if present.
    pub unique_id: Option<u16>,
    /// `actionid` attribute, if present.
    pub action_id: Option<u16>,
    /// Relative path to the Lua script, if present.
    pub script_name: Option<String>,
    /// Built-in function name (`function="market"` etc.), if `script` is absent.
    pub function_name: Option<String>,
    /// Whether far use is permitted (`allowfaruse="1"`).
    pub allow_far_use: bool,
}

/// Outcome of `parse_actions_xml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedActionsXml {
    pub rows: Vec<ParsedActionRow>,
    pub warnings: Vec<String>,
}

/// Parse an `<actions>` XML document and return all valid rows plus warnings.
///
/// Hard errors (malformed XML, missing root element) return `Err`.
/// Per-row errors (no recognisable key attribute) are emitted as warnings —
/// matching C++'s print-error-then-continue behaviour.
pub fn parse_actions_xml(xml: &str) -> Result<ParsedActionsXml, String> {
    let doc = roxmltree::Document::parse(xml).map_err(|e| format!("XML parse error: {e}"))?;
    let root = doc
        .descendants()
        .find(|n| n.has_tag_name("actions"))
        .ok_or_else(|| "Missing <actions> root element".to_string())?;

    let mut rows: Vec<ParsedActionRow> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    for node in root.children().filter(|n| n.is_element()) {
        let item_id: Option<u16> = node.attribute("itemid").and_then(|s| s.parse().ok());
        let from_id: Option<u16> = node.attribute("fromid").and_then(|s| s.parse().ok());
        let to_id: Option<u16> = node.attribute("toid").and_then(|s| s.parse().ok());
        let unique_id: Option<u16> = node.attribute("uniqueid").and_then(|s| s.parse().ok());
        let action_id: Option<u16> = node.attribute("actionid").and_then(|s| s.parse().ok());

        let item_ids: Vec<u16> = if let Some(id) = item_id {
            vec![id]
        } else if let (Some(from), Some(to)) = (from_id, to_id) {
            if from <= to {
                (from..=to).collect()
            } else {
                warnings.push(format!(
                    "[Action::configureEvent] Invalid range fromid={from} toid={to}: {}",
                    node.attribute("script").unwrap_or("<no-script>")
                ));
                continue;
            }
        } else if unique_id.is_some() || action_id.is_some() {
            vec![]
        } else {
            warnings.push(format!(
                "[Action::configureEvent] Missing itemid/fromid/toid/uniqueid/actionid: {}",
                node.attribute("script").unwrap_or("<no-script>")
            ));
            continue;
        };

        let allow_far_use = node
            .attribute("allowfaruse")
            .map(|s| matches!(s, "1" | "true"))
            .unwrap_or(false);

        rows.push(ParsedActionRow {
            item_ids,
            unique_id,
            action_id,
            script_name: node.attribute("script").map(str::to_string),
            function_name: node.attribute("function").map(str::to_string),
            allow_far_use,
        });
    }

    Ok(ParsedActionsXml { rows, warnings })
}

/// Build `Actions` entries from a parsed row and register them.
///
/// Rows with only `function_name` (built-in) and no `script_name` are skipped.
/// Mirrors C++ `Actions::registerEvent`.
pub fn apply_parsed_action(
    actions: &mut crate::actions::Actions,
    row: &ParsedActionRow,
) {
    let script_name = match &row.script_name {
        Some(s) => s.clone(),
        None => return, // built-in function, no Lua — skip
    };

    if let Some(uid) = row.unique_id {
        let mut action = crate::actions::Action::new(script_name.clone(), crate::actions::ActionType::UseItem);
        action.allow_far_use = row.allow_far_use;
        action.unique_id = Some(uid);
        actions.register_by_unique_id(uid, action);
    }

    if let Some(aid) = row.action_id {
        let mut action = crate::actions::Action::new(script_name.clone(), crate::actions::ActionType::UseItem);
        action.allow_far_use = row.allow_far_use;
        action.action_id = Some(aid);
        actions.register_by_action_id(aid, action);
    }

    for &id in &row.item_ids {
        let mut action = crate::actions::Action::new(script_name.clone(), crate::actions::ActionType::UseItem);
        action.allow_far_use = row.allow_far_use;
        action.item_id = Some(id);
        actions.register_by_item_id(id, action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_itemid() {
        let xml = r#"<actions><action itemid="2120" script="tools/rope.lua" /></actions>"#;
        let result = parse_actions_xml(xml).unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].item_ids, vec![2120]);
        assert_eq!(result.rows[0].script_name.as_deref(), Some("tools/rope.lua"));
        assert!(!result.rows[0].allow_far_use);
    }

    #[test]
    fn parse_fromid_toid_range() {
        let xml = r#"<actions><action fromid="10" toid="12" script="range.lua" /></actions>"#;
        let result = parse_actions_xml(xml).unwrap();
        assert_eq!(result.rows[0].item_ids, vec![10, 11, 12]);
    }

    #[test]
    fn parse_allowfaruse_flag() {
        let xml =
            r#"<actions><action itemid="2580" script="fishing.lua" allowfaruse="1" /></actions>"#;
        let result = parse_actions_xml(xml).unwrap();
        assert!(result.rows[0].allow_far_use);
    }

    #[test]
    fn parse_uniqueid() {
        let xml =
            r#"<actions><action uniqueid="30015" script="quests/annihilator.lua" /></actions>"#;
        let result = parse_actions_xml(xml).unwrap();
        assert_eq!(result.rows[0].unique_id, Some(30015));
        assert!(result.rows[0].item_ids.is_empty());
    }

    #[test]
    fn parse_function_attribute() {
        let xml = r#"<actions><action itemid="14405" function="market" /></actions>"#;
        let result = parse_actions_xml(xml).unwrap();
        assert_eq!(result.rows[0].function_name.as_deref(), Some("market"));
        assert!(result.rows[0].script_name.is_none());
    }

    #[test]
    fn missing_key_emits_warning() {
        let xml = r#"<actions><action script="orphan.lua" /></actions>"#;
        let result = parse_actions_xml(xml).unwrap();
        assert!(result.rows.is_empty());
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn malformed_xml_returns_err() {
        let result = parse_actions_xml("not xml at all");
        assert!(result.is_err());
    }

    #[test]
    fn missing_root_returns_err() {
        let result = parse_actions_xml("<root />");
        assert!(result.is_err());
    }

    #[test]
    fn multiple_rows_parsed() {
        let xml = r#"<actions>
            <action itemid="2120" script="tools/rope.lua" />
            <action fromid="2666" toid="2670" script="others/food.lua" />
        </actions>"#;
        let result = parse_actions_xml(xml).unwrap();
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[1].item_ids.len(), 5);
    }

    #[test]
    fn apply_parsed_action_registers_by_item_id() {
        use crate::actions::Actions;
        let xml = r#"<actions><action itemid="2120" script="tools/rope.lua" /></actions>"#;
        let parsed = parse_actions_xml(xml).unwrap();
        let mut actions = Actions::new();
        apply_parsed_action(&mut actions, &parsed.rows[0]);
        assert!(actions.get_by_item_id(2120).is_some());
        assert_eq!(
            actions.get_by_item_id(2120).unwrap().script_name,
            "tools/rope.lua"
        );
    }

    #[test]
    fn apply_parsed_action_skips_function_only_rows() {
        use crate::actions::Actions;
        let xml = r#"<actions><action itemid="14405" function="market" /></actions>"#;
        let parsed = parse_actions_xml(xml).unwrap();
        let mut actions = Actions::new();
        apply_parsed_action(&mut actions, &parsed.rows[0]);
        assert!(actions.get_by_item_id(14405).is_none());
    }

    #[test]
    fn apply_parsed_action_registers_range() {
        use crate::actions::Actions;
        let xml =
            r#"<actions><action fromid="10" toid="12" script="range.lua" /></actions>"#;
        let parsed = parse_actions_xml(xml).unwrap();
        let mut actions = Actions::new();
        apply_parsed_action(&mut actions, &parsed.rows[0]);
        assert!(actions.get_by_item_id(10).is_some());
        assert!(actions.get_by_item_id(11).is_some());
        assert!(actions.get_by_item_id(12).is_some());
        assert!(actions.get_by_item_id(13).is_none());
    }

    #[test]
    fn apply_parsed_action_registers_by_unique_id() {
        use crate::actions::Actions;
        let xml =
            r#"<actions><action uniqueid="30015" script="quests/annihilator.lua" /></actions>"#;
        let parsed = parse_actions_xml(xml).unwrap();
        let mut actions = Actions::new();
        apply_parsed_action(&mut actions, &parsed.rows[0]);
        assert!(actions.get_by_unique_id(30015).is_some());
    }
}
