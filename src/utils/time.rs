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
