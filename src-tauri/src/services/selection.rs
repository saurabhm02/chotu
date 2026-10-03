//! Reading the text the user has selected in whatever app is in front.
//! Only macOS can do this for now; other systems always get `None`.

/// The selected text of the app in front, or `None` if nothing is selected.
#[cfg(target_os = "macos")]
pub fn capture() -> Option<String> {
    if let Some(text) = via_accessibility() {
        log::info!("selected text read through the Accessibility API");
        return Some(text);
    }
    let text = via_copy_shortcut();
    if text.is_some() {
        log::info!("selected text read by simulating Cmd+C");
    }
    text
}

#[cfg(not(target_os = "macos"))]
pub fn capture() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
use std::{thread::sleep, time::Duration};

#[cfg(target_os = "macos")]
use accessibility::{AXAttribute, AXUIElement};
#[cfg(target_os = "macos")]
use arboard::Clipboard;
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;
#[cfg(target_os = "macos")]
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// Ask macOS: "which element has focus, and what text is selected in it?"
/// Fast and leaves the clipboard alone, but not every app supports it.
#[cfg(target_os = "macos")]
fn via_accessibility() -> Option<String> {
    let system = AXUIElement::system_wide();

    let focused_attr = AXAttribute::new(&CFString::new("AXFocusedUIElement"));
    let focused = system
        .attribute(&focused_attr)
        .ok()?
        .downcast_into::<AXUIElement>()?;

    let selected_attr = AXAttribute::new(&CFString::new("AXSelectedText"));
    let selected = focused
        .attribute(&selected_attr)
        .ok()?
        .downcast_into::<CFString>()?;

    let text = selected.to_string();
    (!text.trim().is_empty()).then_some(text)
}

/// Fallback: press Cmd+C ourselves, read the clipboard, then put the user's
/// old clipboard text back.
#[cfg(target_os = "macos")]
fn via_copy_shortcut() -> Option<String> {
    let mut clipboard = Clipboard::new().ok()?;
    let previous = clipboard.get_text().ok();
    clipboard.set_text("").ok()?; // so "nothing copied" can't look like the old text

    let mut enigo = Enigo::new(&Settings::default()).ok()?;
    enigo.key(Key::Meta, Direction::Press).ok()?;
    enigo.key(Key::Unicode('c'), Direction::Click).ok()?;
    enigo.key(Key::Meta, Direction::Release).ok()?;

    // Slow apps need a moment to fill the clipboard; wait up to ~300ms.
    let mut copied = String::new();
    for _ in 0..10 {
        sleep(Duration::from_millis(30));
        copied = clipboard.get_text().unwrap_or_default();
        if !copied.is_empty() {
            break;
        }
    }

    if let Some(previous) = previous {
        let _ = clipboard.set_text(previous);
    }
    (!copied.trim().is_empty()).then_some(copied)
}
