use wasm_bindgen::JsValue;

/// Formats as `h:mm AM/PM`.
fn clock_text(date: &js_sys::Date) -> String {
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    let ampm = if hours >= 12 { "PM" } else { "AM" };
    let h12 = if hours == 0 {
        12
    } else if hours > 12 {
        hours - 12
    } else {
        hours
    };
    format!("{}:{:02} {}", h12, minutes, ampm)
}

/// Local time as `h:mm AM/PM`, used to stamp chat messages.
pub fn current_time_str() -> String {
    clock_text(&js_sys::Date::new_0())
}

/// Same as `current_time_str` for a saved timestamp in milliseconds.
pub fn clock_from_ms(ms: i64) -> String {
    clock_text(&js_sys::Date::new(&JsValue::from_f64(ms as f64)))
}

/// Milliseconds since 1970.
pub fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}

/// "just now", "5m ago", "3h ago", "2d ago", "3w ago".
pub fn time_ago(now_ms: i64, then_ms: i64) -> String {
    let seconds = (now_ms - then_ms).max(0) / 1000;
    match seconds {
        0..=59 => "just now".to_string(),
        60..=3_599 => format!("{}m ago", seconds / 60),
        3_600..=86_399 => format!("{}h ago", seconds / 3_600),
        86_400..=604_799 => format!("{}d ago", seconds / 86_400),
        _ => format!("{}w ago", seconds / 604_800),
    }
}

/// How long an answer took: "0.8s", "12.4s", "1m 5s".
pub fn format_duration(ms: u64) -> String {
    let seconds = ms as f64 / 1000.0;
    if seconds < 60.0 {
        return format!("{seconds:.1}s");
    }
    let total = (seconds.round()) as u64;
    format!("{}m {}s", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn under_a_minute_shows_seconds() {
        assert_eq!(format_duration(800), "0.8s");
        assert_eq!(format_duration(12_400), "12.4s");
        assert_eq!(format_duration(59_900), "59.9s");
    }

    #[test]
    fn a_minute_or_more_shows_minutes_and_seconds() {
        assert_eq!(format_duration(60_000), "1m 0s");
        assert_eq!(format_duration(65_000), "1m 5s");
        assert_eq!(format_duration(125_400), "2m 5s");
    }

    #[test]
    fn time_ago_picks_the_right_unit() {
        let now = 1_000_000_000_000;
        assert_eq!(time_ago(now, now - 5_000), "just now");
        assert_eq!(time_ago(now, now - 5 * 60_000), "5m ago");
        assert_eq!(time_ago(now, now - 3 * 3_600_000), "3h ago");
        assert_eq!(time_ago(now, now - 2 * 86_400_000), "2d ago");
        assert_eq!(time_ago(now, now - 21 * 86_400_000), "3w ago");
    }

    #[test]
    fn time_ago_never_goes_negative() {
        assert_eq!(time_ago(1_000, 5_000), "just now");
    }
}
