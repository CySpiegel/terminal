/// Send a Windows toast notification.
pub fn notify(title: &str, body: &str) {
    // TODO: Use windows-rs toast notification API
    let _ = std::process::Command::new("powershell")
        .arg("-Command")
        .arg(format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
             $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent(0); \
             $text = $template.GetElementsByTagName('text'); \
             $text.Item(0).AppendChild($template.CreateTextNode('{}')) > $null; \
             $text.Item(1).AppendChild($template.CreateTextNode('{}')) > $null; \
             $toast = [Windows.UI.Notifications.ToastNotification]::new($template); \
             [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('terminal').Show($toast)",
            title, body
        ))
        .spawn();
}
