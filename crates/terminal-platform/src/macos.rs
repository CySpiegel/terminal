/// Send a macOS desktop notification.
pub fn notify(title: &str, body: &str) {
    // TODO: Use NSUserNotification or UNUserNotificationCenter
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "display notification \"{}\" with title \"{}\"",
            body, title
        ))
        .spawn();
}
