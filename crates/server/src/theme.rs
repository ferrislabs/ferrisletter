//! Theme registry + display preferences scaffold.
//!
//! Ferrisletter ships with **zero built-in themes** — it provides only the
//! pattern. Downstreams (e.g. Lattice) register concrete themes at startup
//! via [`ThemeRegistry::register`].
//!
//! A theme provides CSS that Claude uses as a base for Generative UI digests.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A named theme provides CSS that Claude uses as a base for Generative UI digests.
pub trait Theme: Send + Sync {
    /// A stable short identifier used by users (e.g. "default", "daltonian").
    fn name(&self) -> &str;

    /// Human-readable one-line description.
    fn description(&self) -> &str;

    /// CSS snippet that Claude should use as the foundation of the digest artifact.
    fn css(&self) -> &str;
}

/// Lightweight descriptor returned to MCP clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub description: String,
}

/// A registry of named themes.
///
/// Downstreams register concrete [`Theme`] implementations at startup.
/// Ferrisletter exposes the registered themes via `ferrisletter_list_themes`
/// so users can discover what's available.
#[derive(Default)]
pub struct ThemeRegistry {
    themes: HashMap<String, Box<dyn Theme>>,
    /// Insertion order, so `list()` returns themes in a stable order.
    order: Vec<String>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a theme. If a theme with the same name already exists, it
    /// is replaced.
    pub fn register(&mut self, theme: Box<dyn Theme>) {
        let name = theme.name().to_string();
        if !self.themes.contains_key(&name) {
            self.order.push(name.clone());
        }
        self.themes.insert(name, theme);
    }

    /// Look up a theme by name.
    pub fn get(&self, name: &str) -> Option<&dyn Theme> {
        self.themes.get(name).map(|b| b.as_ref())
    }

    /// List theme names in registration order.
    pub fn names(&self) -> Vec<&str> {
        self.order.iter().map(|s| s.as_str()).collect()
    }

    /// List theme info (name + description) in registration order.
    pub fn list(&self) -> Vec<ThemeInfo> {
        self.order
            .iter()
            .filter_map(|n| self.themes.get(n))
            .map(|t| ThemeInfo {
                name: t.name().to_string(),
                description: t.description().to_string(),
            })
            .collect()
    }

    /// Number of registered themes.
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// True if no themes are registered.
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }
}

/// Generic display preferences for a user's digest.
///
/// Newsletters can extend this with product-specific fields via the
/// [`extra`](DisplayPreferences::extra) JSON blob (e.g. Lattice stores
/// `summary_words`, `separator_style`, `layout_density` there).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayPreferences {
    /// Name of the theme the user selected (must exist in the registry).
    pub theme: Option<String>,
    /// Freeform instructions the user gave Claude (e.g. "use emoji headers").
    pub custom_instructions: Option<String>,
    /// Product-specific preferences as JSON (stored as JSONB in databases).
    #[serde(default)]
    pub extra: serde_json::Value,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTheme {
        name: String,
        description: String,
        css: String,
    }

    impl Theme for FakeTheme {
        fn name(&self) -> &str {
            &self.name
        }
        fn description(&self) -> &str {
            &self.description
        }
        fn css(&self) -> &str {
            &self.css
        }
    }

    fn make(name: &str, description: &str) -> Box<dyn Theme> {
        Box::new(FakeTheme {
            name: name.to_string(),
            description: description.to_string(),
            css: format!("body {{ --name: {name}; }}"),
        })
    }

    #[test]
    fn new_registry_is_empty() {
        let reg = ThemeRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        assert!(reg.get("default").is_none());
    }

    #[test]
    fn register_and_get() {
        let mut reg = ThemeRegistry::new();
        reg.register(make("default", "Clean dark theme"));
        reg.register(make("minimal", "Headlines only"));

        assert_eq!(reg.len(), 2);
        assert_eq!(
            reg.get("default").unwrap().description(),
            "Clean dark theme"
        );
        assert_eq!(reg.get("minimal").unwrap().description(), "Headlines only");
    }

    #[test]
    fn list_preserves_registration_order() {
        let mut reg = ThemeRegistry::new();
        reg.register(make("default", "a"));
        reg.register(make("light", "b"));
        reg.register(make("minimal", "c"));

        let names: Vec<&str> = reg.names();
        assert_eq!(names, vec!["default", "light", "minimal"]);

        let info = reg.list();
        assert_eq!(info[0].name, "default");
        assert_eq!(info[2].name, "minimal");
    }

    #[test]
    fn register_replaces_existing() {
        let mut reg = ThemeRegistry::new();
        reg.register(make("default", "original"));
        reg.register(make("default", "replacement"));

        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get("default").unwrap().description(), "replacement");
    }

    #[test]
    fn display_preferences_default_is_empty() {
        let prefs = DisplayPreferences::default();
        assert!(prefs.theme.is_none());
        assert!(prefs.custom_instructions.is_none());
        // serde_json::Value::default() is Null
        assert!(prefs.extra.is_null());
    }

    #[test]
    fn display_preferences_roundtrip() {
        let prefs = DisplayPreferences {
            theme: Some("daltonian".into()),
            custom_instructions: Some("use emoji headers".into()),
            extra: serde_json::json!({ "summary_words": 40 }),
        };
        let json = serde_json::to_string(&prefs).unwrap();
        let back: DisplayPreferences = serde_json::from_str(&json).unwrap();
        assert_eq!(back.theme.as_deref(), Some("daltonian"));
        assert_eq!(back.extra["summary_words"], 40);
    }
}
