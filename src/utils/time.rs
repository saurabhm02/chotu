/// Local time as `h:mm AM/PM`, used to stamp chat messages.
pub fn current_time_str() -> String {
    let date = js_sys::Date::new_0();
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
}
