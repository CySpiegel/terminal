/// Send a Linux desktop notification via notify-send.
pub fn notify(title: &str, body: &str) {
    // TODO: Use D-Bus directly via zbus crate
    let _ = std::process::Command::new("notify-send")
        .arg(title)
        .arg(body)
        .spawn();
}
