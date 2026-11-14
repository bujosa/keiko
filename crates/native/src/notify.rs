//! macOS notifications, sent as Keiko (its icon and name), not as a script runner.

/// Show a notification. Failures (notifications turned off for Keiko) are returned, never fatal.
pub fn notify(title: &str, subtitle: &str, message: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            let _ = mac_notification_sys::set_application("dev.bujosa.keiko");
        });
        mac_notification_sys::Notification::new()
            .title(title)
            .subtitle(subtitle)
            .message(message)
            .asynchronous(true)
            .send()
            .map(|_| ())
            .map_err(|e| format!("notification: {e}"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (title, subtitle, message);
        Ok(())
    }
}
