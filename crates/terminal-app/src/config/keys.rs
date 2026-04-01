use serde::Deserialize;

/// Keybinding configuration.
#[derive(Debug, Deserialize, Default)]
pub struct KeybindingsConfig {
    #[serde(default)]
    pub terminal: Vec<KeyBinding>,
    #[serde(default)]
    pub gtd: Vec<KeyBinding>,
}

#[derive(Debug, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub action: String,
}
