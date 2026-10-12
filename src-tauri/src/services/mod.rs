pub mod attachments;
pub mod clipboard_attachments;
pub mod decide;
pub mod failover;
pub mod history;
pub mod llm;
pub mod ocr;
pub mod panel;
pub mod prompt;
pub mod screen;
pub mod selection;
#[cfg(desktop)]
pub mod shortcut;
pub mod tray;
pub mod web;
pub mod web_search;
#[cfg(target_os = "macos")]
pub mod window;
