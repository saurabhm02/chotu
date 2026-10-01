use std::rc::Rc;

use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::components::ask_bar::view::AskBarView;
use crate::services::dispatcher::run_cmd;
use crate::services::tauri_bridge::{resize_window, start_window_drag};
use crate::types::chat::current_time_str;
use crate::types::command::detect_cmd;
use crate::types::{all_commands, ChatTurn, Command, Commands};

#[derive(Clone, PartialEq, Default)]
struct History(Vec<ChatTurn>);

enum HistoryAction {
    Push(ChatTurn),
    AppendDelta(String),
    Finish {
        response: String,
        sources: Vec<(String, String)>,
    },
}

impl Reducible for History {
    type Action = HistoryAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut turns = self.0.clone();
        match action {
            HistoryAction::Push(turn) => turns.push(turn),
            HistoryAction::AppendDelta(delta) => {
                if let Some(last) = turns.last_mut() {
                    last.response.push_str(&delta);
                }
            }
            HistoryAction::Finish { response, sources } => {
                if let Some(last) = turns.last_mut() {
                    last.response = response;
                    last.sources = sources;
                }
            }
        }
        Rc::new(History(turns))
    }
}

#[function_component(AskBar)]
pub fn ask_bar() -> Html {
    let input_ref = use_node_ref();
    let history = use_reducer(History::default);
    let is_loading = use_state(|| false);
    let input_text = use_state(String::new);
    let selected_index = use_state(|| 0usize);
    let is_palette_dismissed = use_state(|| false);

    let on_mousedown = Callback::from(|e: MouseEvent| {
        if e.button() == 0 {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                let tag = target.tag_name().to_lowercase();
                if tag != "textarea"
                    && tag != "input"
                    && tag != "button"
                    && tag != "svg"
                    && tag != "path"
                    && tag != "a"
                {
                    start_window_drag();
                }
            }
        }
    });

    let visible_commands: Vec<Command> = {
        let text = (*input_text).clone();
        if !*is_palette_dismissed && text.starts_with('/') {
            let filter = text.strip_prefix('/').unwrap_or("").trim().to_lowercase();
            all_commands()
                .into_iter()
                .filter(|c| {
                    if filter.is_empty() {
                        true
                    } else {
                        c.name.to_lowercase().starts_with(&filter)
                            || c.name.to_lowercase().contains(&filter)
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    };

    let is_palette_open = !visible_commands.is_empty();

    // Clamp selected_index if commands list shrinks
    {
        let selected_index = selected_index.clone();
        let cmd_len = visible_commands.len();
        use_effect_with(cmd_len, move |&len| {
            if len > 0 && *selected_index >= len {
                selected_index.set(len - 1);
            }
            || ()
        });
    }

    {
        let deps = (
            history.0.clone(),
            *is_loading,
            (*input_text).clone(),
            is_palette_open,
            visible_commands.len(),
        );
        use_effect_with(deps, |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    let height = if let Some(el) = doc.get_element_by_id("app-container") {
                        el.scroll_height() as f64
                    } else if let Some(body) = doc.body() {
                        body.scroll_height() as f64
                    } else {
                        55.0
                    };
                    let height = height.clamp(55.0, 600.0);
                    if let Err(e) = resize_window(550.0, height).await {
                        log::error!("resize failed: {:?}", e);
                    }
                }
            });
            || ()
        });
    }

    let on_input = {
        let input_ref = input_ref.clone();
        let input_text = input_text.clone();
        let is_palette_dismissed = is_palette_dismissed.clone();

        Callback::from(move |_: InputEvent| {
            if let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() {
                let val = textarea.value();
                let style = textarea.style();
                let _ = style.set_property("height", "auto");
                let scroll_h = textarea.scroll_height();
                let new_h = format!("{}px", scroll_h.min(180));
                let _ = style.set_property("height", &new_h);
                input_text.set(val);
                is_palette_dismissed.set(false);
            }
        })
    };

    let apply_command = {
        let input_ref = input_ref.clone();
        let input_text = input_text.clone();
        let is_palette_dismissed = is_palette_dismissed.clone();

        Rc::new(move |command: &Command| {
            let text = format!("/{} ", command.name);
            if let Some(input) = input_ref.cast::<HtmlTextAreaElement>() {
                input.set_value(&text);
                let style = input.style();
                let _ = style.set_property("height", "auto");
                input_text.set(text);
                is_palette_dismissed.set(true);
                let _ = input.focus();
            }
        })
    };

    let on_cmd_select = {
        let apply_command = apply_command.clone();
        Callback::from(move |cmd: Commands| {
            if let Some(command) = all_commands().into_iter().find(|c| c.cmd == cmd) {
                apply_command(&command);
            }
        })
    };

    let on_cmd_hover = {
        let selected_index = selected_index.clone();
        Callback::from(move |idx: usize| {
            selected_index.set(idx);
        })
    };

    let trigger_prompt = {
        let history = history.dispatcher();
        let is_loading = is_loading.clone();

        Rc::new(move |raw_text: String| {
            let value = raw_text.trim().to_string();
            if !value.is_empty() {
                let history = history.clone();
                let is_loading = is_loading.clone();
                is_loading.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let (cmd, query) = detect_cmd(&value);

                    history.dispatch(HistoryAction::Push(ChatTurn {
                        prompt: query.clone(),
                        response: String::new(),
                        sources: vec![],
                        timestamp: current_time_str(),
                    }));

                    let history_for_stream = history.clone();
                    let on_chunk = move |delta: String| {
                        history_for_stream.dispatch(HistoryAction::AppendDelta(delta));
                    };

                    let (response, sources) = run_cmd(cmd, query, on_chunk).await;

                    history.dispatch(HistoryAction::Finish { response, sources });
                    is_loading.set(false);
                });
            }
        })
    };

    let submit = {
        let input_ref = input_ref.clone();
        let input_text = input_text.clone();
        let trigger_prompt = trigger_prompt.clone();
        let is_palette_dismissed = is_palette_dismissed.clone();

        move || {
            if let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() {
                let value = textarea.value().trim().to_string();

                if !value.is_empty() {
                    textarea.set_value("");
                    let style = textarea.style();
                    let _ = style.set_property("height", "auto");
                    input_text.set(String::new());
                    is_palette_dismissed.set(true);

                    trigger_prompt(value);
                }
            }
        }
    };

    let on_regenerate = {
        let trigger_prompt = trigger_prompt.clone();
        let history_turns = history.0.clone();
        Callback::from(move |idx: usize| {
            if let Some(turn) = history_turns.get(idx) {
                trigger_prompt(turn.prompt.clone());
            }
        })
    };

    let on_click = {
        let submit = submit.clone();
        Callback::from(move |_: MouseEvent| {
            submit();
        })
    };

    let on_keydown = {
        let submit = submit.clone();
        let input_ref = input_ref.clone();
        let input_text = input_text.clone();
        let selected_index = selected_index.clone();
        let is_palette_dismissed = is_palette_dismissed.clone();
        let apply_command = apply_command.clone();
        let visible_cmds = visible_commands.clone();

        Callback::from(move |event: KeyboardEvent| {
            let key = event.key();

            if is_palette_open {
                match key.as_str() {
                    "ArrowDown" => {
                        event.prevent_default();
                        if !visible_cmds.is_empty() {
                            let next = (*selected_index + 1) % visible_cmds.len();
                            selected_index.set(next);
                        }
                        return;
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        if !visible_cmds.is_empty() {
                            let prev = if *selected_index == 0 {
                                visible_cmds.len() - 1
                            } else {
                                *selected_index - 1
                            };
                            selected_index.set(prev);
                        }
                        return;
                    }
                    "Tab" => {
                        event.prevent_default();
                        if let Some(cmd) = visible_cmds.get(*selected_index) {
                            apply_command(cmd);
                        }
                        return;
                    }
                    "Enter" if !event.shift_key() => {
                        event.prevent_default();
                        if let Some(cmd) = visible_cmds.get(*selected_index) {
                            apply_command(cmd);
                        }
                        return;
                    }
                    "Escape" => {
                        event.prevent_default();
                        is_palette_dismissed.set(true);
                        return;
                    }
                    _ => {}
                }
            }

            if key == "Enter" {
                if !event.shift_key() {
                    event.prevent_default();
                    submit();
                } else {
                    if let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() {
                        let style = textarea.style();
                        let _ = style.set_property("height", "auto");
                        let scroll_h = textarea.scroll_height();
                        let new_h = format!("{}px", scroll_h.min(180));
                        let _ = style.set_property("height", &new_h);
                        input_text.set(textarea.value());
                    }
                }
            }
        })
    };

    html! {
        <AskBarView
            input_ref={input_ref}
            on_click={on_click}
            on_keydown={on_keydown}
            on_mousedown={on_mousedown}
            history={history.0.clone()}
            is_loading={*is_loading}
            commands={visible_commands}
            selected_cmd_index={*selected_index}
            on_cmd_select={on_cmd_select}
            on_cmd_hover={on_cmd_hover}
            on_input={on_input}
            on_regenerate={on_regenerate}
        />
    }
}
