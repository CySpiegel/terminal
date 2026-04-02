pub mod theme;
pub mod keys;

use serde::Deserialize;

/// Application configuration loaded from TOML.
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub font: FontConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub gtd: GtdConfig,
    #[serde(default)]
    pub ai: terminal_ai::config::AiConfig,
}

#[derive(Debug, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: f32,
    pub line_height: f32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "monospace".to_string(),
            size: 14.0,
            line_height: 1.2,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub padding: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            padding: 4,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TerminalConfig {
    pub scrollback_lines: u32,
    pub cursor_style: String,
    pub cursor_blink: bool,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: 10000,
            cursor_style: "block".to_string(),
            cursor_blink: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GtdConfig {
    pub enabled: bool,
    pub hotkey: String,
    pub notification_check_interval_secs: u64,
}

impl Default for GtdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hotkey: "ctrl+shift+g".to_string(),
            notification_check_interval_secs: 60,
        }
    }
}
