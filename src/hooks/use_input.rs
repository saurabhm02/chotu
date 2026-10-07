use std::rc::Rc;

use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::models::{all_commands, Command, Commands};

const MAX_INPUT_HEIGHT: i32 = 180;

pub struct InputHandle {
    pub text: String,
    /// Slash-commands matching what's typed. Empty means "popup closed".
    pub commands: Vec<Command>,
    pub selected_index: usize,
    pub on_input: Callback<InputEvent>,
    pub on_keydown: Callback<KeyboardEvent>,
    pub on_send_click: Callback<MouseEvent>,
    pub on_cmd_select: Callback<Commands>,
    pub on_cmd_hover: Callback<usize>,
}

/// The text box: typed text, the slash-command popup (filtering + keyboard
/// navigation), and submitting a message. `on_submit` fires with the trimmed
/// text whenever the user sends a message.
#[hook]
pub fn use_input(
    input_ref: NodeRef,
    on_submit: Callback<String>,
    can_send_empty: bool,
) -> InputHandle {
    let text = use_state(String::new);
    let selected_index = use_state(|| 0usize);
    let palette_dismissed = use_state(|| false);

    let commands = matching_commands(&text, *palette_dismissed);

    {
        let selected_index = selected_index.clone();
        let count = commands.len();
        use_effect_with(count, move |&count| {
            if count > 0 && *selected_index >= count {
                selected_index.set(count - 1);
            }
            || ()
        });
    }

    let pick_command: Rc<dyn Fn(&Command)> = {
        let input_ref = input_ref.clone();
        let text = text.clone();
        let palette_dismissed = palette_dismissed.clone();
        Rc::new(move |command: &Command| {
            let value = format!("/{} ", command.name);
            fill_textarea(&input_ref, &value);
            text.set(value);
            palette_dismissed.set(true);
        })
    };

    let submit: Rc<dyn Fn()> = {
        let input_ref = input_ref.clone();
        let text = text.clone();
        let palette_dismissed = palette_dismissed.clone();
        Rc::new(move || {
            let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() else {
                return;
            };
            let message = textarea.value().trim().to_string();
            if message.is_empty() && !can_send_empty {
                return;
            }
            fill_textarea(&input_ref, "");
            text.set(String::new());
            palette_dismissed.set(true);
            on_submit.emit(message);
        })
    };

    let on_input = {
        let input_ref = input_ref.clone();
        let text = text.clone();
        let palette_dismissed = palette_dismissed.clone();
        Callback::from(move |_: InputEvent| {
            let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() else {
                return;
            };
            auto_grow(&textarea);
            text.set(textarea.value());
            palette_dismissed.set(false);
        })
    };

    let on_send_click = {
        let submit = submit.clone();
        Callback::from(move |_: MouseEvent| submit())
    };

    let on_cmd_select = {
        let pick_command = pick_command.clone();
        Callback::from(move |cmd: Commands| {
            if let Some(command) = all_commands().into_iter().find(|c| c.cmd == cmd) {
                pick_command(&command);
            }
        })
    };

    let on_cmd_hover = {
        let selected_index = selected_index.clone();
        Callback::from(move |idx: usize| selected_index.set(idx))
    };

    let on_keydown = {
        let input_ref = input_ref.clone();
        let text = text.clone();
        let selected_index = selected_index.clone();
        let palette_dismissed = palette_dismissed.clone();
        let pick_command = pick_command.clone();
        let submit = submit.clone();
        let visible_commands = commands.clone();

        Callback::from(move |event: KeyboardEvent| {
            let palette_is_open = !visible_commands.is_empty();
            let handled_by_palette = palette_is_open
                && handle_palette_key(
                    &event,
                    &visible_commands,
                    &selected_index,
                    &palette_dismissed,
                    &pick_command,
                );
            if handled_by_palette {
                return;
            }

            handle_enter_key(&event, &input_ref, &text, &submit);
        })
    };

    InputHandle {
        text: (*text).clone(),
        commands,
        selected_index: *selected_index,
        on_input,
        on_keydown,
        on_send_click,
        on_cmd_select,
        on_cmd_hover,
    }
}

/// Resizes the textarea to fit its text, up to `MAX_INPUT_HEIGHT` tall.
fn auto_grow(textarea: &HtmlTextAreaElement) {
    let style = textarea.style();
    let _ = style.set_property("height", "auto");
    let height = textarea.scroll_height().min(MAX_INPUT_HEIGHT);
    let _ = style.set_property("height", &format!("{height}px"));
}

/// Sets the textarea's text, shrinks it back down, and refocuses it.
fn fill_textarea(input_ref: &NodeRef, value: &str) {
    let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() else {
        return;
    };
    textarea.set_value(value);
    let _ = textarea.style().set_property("height", "auto");
    let _ = textarea.focus();
}

/// Slash-commands whose name matches what's typed after the `/`.
/// Empty result means: nothing to show, popup stays closed.
fn matching_commands(text: &str, dismissed: bool) -> Vec<Command> {
    if dismissed || !text.starts_with('/') {
        return Vec::new();
    }
    let search = text.trim_start_matches('/').trim().to_lowercase();
    all_commands()
        .into_iter()
        .filter(|c| search.is_empty() || c.name.to_lowercase().contains(&search))
        .collect()
}

/// Arrow keys move the highlight, Tab/Enter pick the highlighted command,
/// Escape closes the popup. Returns `true` if one of those keys was pressed.
fn handle_palette_key(
    event: &KeyboardEvent,
    visible: &[Command],
    selected_index: &UseStateHandle<usize>,
    dismissed: &UseStateHandle<bool>,
    pick: &Rc<dyn Fn(&Command)>,
) -> bool {
    let len = visible.len();
    match event.key().as_str() {
        "ArrowDown" => {
            event.prevent_default();
            selected_index.set((**selected_index + 1) % len);
            true
        }
        "ArrowUp" => {
            event.prevent_default();
            let prev = if **selected_index == 0 {
                len - 1
            } else {
                **selected_index - 1
            };
            selected_index.set(prev);
            true
        }
        "Tab" => {
            event.prevent_default();
            if let Some(command) = visible.get(**selected_index) {
                pick(command);
            }
            true
        }
        "Enter" if !event.shift_key() => {
            event.prevent_default();
            if let Some(command) = visible.get(**selected_index) {
                pick(command);
            }
            true
        }
        "Escape" => {
            event.prevent_default();
            dismissed.set(true);
            true
        }
        _ => false,
    }
}

/// Plain Enter sends the message. Shift+Enter inserts a newline instead and
/// re-measures the textarea's height.
fn handle_enter_key(
    event: &KeyboardEvent,
    input_ref: &NodeRef,
    text: &UseStateHandle<String>,
    submit: &Rc<dyn Fn()>,
) {
    if event.key() != "Enter" {
        return;
    }

    if event.shift_key() {
        if let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() {
            auto_grow(&textarea);
            text.set(textarea.value());
        }
        return;
    }

    event.prevent_default();
    submit();
}
