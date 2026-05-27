use dioxus::prelude::*;

/// Notification severity levels. The dropdown matches on every variant
/// to pick a colour even though nothing pushes notifications today —
/// the producer hooks were removed when no production callsite was
/// wiring them. The variants stay so the renderer keeps compiling once
/// soland surfaces a real notification feed.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub message: String,
    pub severity: NotificationSeverity,
    pub timestamp: String,
    pub read: bool,
}

pub static NOTIFICATIONS: GlobalSignal<Vec<Notification>> = GlobalSignal::new(Vec::new);

pub fn mark_all_read() {
    let mut notifications = NOTIFICATIONS.write();
    for n in notifications.iter_mut() {
        n.read = true;
    }
}

pub fn clear_notifications() {
    let mut notifications = NOTIFICATIONS.write();
    notifications.clear();
}

pub fn unread_count() -> usize {
    let notifications = NOTIFICATIONS.read();
    notifications.iter().filter(|n| !n.read).count()
}
