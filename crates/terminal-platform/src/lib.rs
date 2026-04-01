//! Platform-specific glue for font hints, desktop notifications,
//! and OS integration.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

/// Send a desktop notification.
pub fn notify(title: &str, body: &str) {
    tracing::info!("notification: {} — {}", title, body);

    #[cfg(target_os = "macos")]
    macos::notify(title, body);

    #[cfg(target_os = "linux")]
    linux::notify(title, body);

    #[cfg(target_os = "windows")]
    windows::notify(title, body);
}

/// Get the platform-specific config directory.
pub fn config_dir() -> std::path::PathBuf {
    dirs_next::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("terminal")
}

/// Get the platform-specific data directory.
pub fn data_dir() -> std::path::PathBuf {
    dirs_next::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("terminal")
}
