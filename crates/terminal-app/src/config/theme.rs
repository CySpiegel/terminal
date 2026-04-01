use serde::Deserialize;

/// Color theme configuration.
#[derive(Debug, Deserialize)]
pub struct Theme {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub selection: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Deserialize)]
pub struct ThemeColors {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            foreground: "#ffffff".to_string(),
            background: "#000000".to_string(),
            cursor: "#ffffff".to_string(),
            selection: "#44475a".to_string(),
            colors: ThemeColors {
                black: "#000000".to_string(),
                red: "#cc0000".to_string(),
                green: "#00cc00".to_string(),
                yellow: "#cccc00".to_string(),
                blue: "#0000cc".to_string(),
                magenta: "#cc00cc".to_string(),
                cyan: "#00cccc".to_string(),
                white: "#cccccc".to_string(),
            },
        }
    }
}
