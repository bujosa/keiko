//! Human-readable sizes and dates.

/// 9_437_184 → `9 MB`; under 1 MB → `0.4 MB`.
pub fn size(bytes: u64) -> String {
    let mb = bytes as f64 / 1_048_576.0;
    if mb >= 10.0 {
        format!("{mb:.0} MB")
    } else if mb >= 1.0 {
        format!("{mb:.1} MB")
    } else {
        format!("{:.1} MB", mb.max(0.1))
    }
}

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/// `2026-10-28T20:13:36Z` → `Oct 28, 2026`.
pub fn date(iso: &str) -> Option<String> {
    let d = iso.get(..10)?;
    let mut it = d.split('-');
    let y: u32 = it.next()?.parse().ok()?;
    let m: usize = it.next()?.parse().ok()?;
    let day: u32 = it.next()?.parse().ok()?;
    let month = MONTHS.get(m.checked_sub(1)?)?;
    Some(format!("{month} {day}, {y}"))
}

/// Seconds ago → `just now`, `5 min ago`, `3 h ago`, `2 days ago`.
pub fn ago(secs: u64) -> String {
    match secs {
        0..60 => "just now".into(),
        60..3600 => format!("{} min ago", secs / 60),
        3600..86_400 => format!("{} h ago", secs / 3600),
        86_400..172_800 => "yesterday".into(),
        _ => format!("{} days ago", secs / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(size(106 * 1_048_576), "106 MB");
        assert_eq!(size(9_437_184), "9.0 MB");
        assert_eq!(size(10), "0.1 MB");
        assert_eq!(date("2026-10-28T20:13:36Z").as_deref(), Some("Oct 28, 2026"));
        assert_eq!(date("2026-13-01"), None);
        assert_eq!(date("x"), None);
        assert_eq!(ago(5), "just now");
        assert_eq!(ago(180), "3 min ago");
        assert_eq!(ago(7200), "2 h ago");
        assert_eq!(ago(90_000), "yesterday");
        assert_eq!(ago(3 * 86_400), "3 days ago");
    }
}
