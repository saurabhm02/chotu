use log::info;
use web_sys::HtmlTextAreaElement;
use yew::prelude::*;

use crate::components::ask_bar::view::AskBarView;
use crate::services::tauri_bridge::{invoke_ask_ai, resize_window, start_window_drag};
use crate::types::{all_commands, ChatTurn, Command, Commands};

#[function_component(AskBar)]
pub fn ask_bar() -> Html {
    let input_ref = use_node_ref();
    let history: UseStateHandle<Vec<ChatTurn>> = use_state(Vec::new);
    let is_loading = use_state(|| false);
    let input_text = use_state(String::new);

    let on_mousedown = Callback::from(|e: MouseEvent| {
        if e.button() == 0 {
            if let Some(target) = e.target_dyn_into::<web_sys::HtmlElement>() {
                let tag = target.tag_name().to_lowercase();
                if tag != "textarea" && tag != "input" && tag != "button" && tag != "svg" && tag != "path" {
                    start_window_drag();
                }
            }
        }
    });

    {
        let deps = ((*history).clone(), *is_loading, (*input_text).clone());
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

        Callback::from(move |_: InputEvent| {
            if let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() {
                let val = textarea.value();
                let style = textarea.style();
                let _ = style.set_property("height", "auto");
                let scroll_h = textarea.scroll_height();
                let new_h = format!("{}px", scroll_h.min(180));
                let _ = style.set_property("height", &new_h);
                input_text.set(val);
            }
        })
    };

    let visible_commands: Vec<Command> = {
        let text = (*input_text).clone();
        match text.strip_prefix('/') {
            Some(prefix) => all_commands()
                .into_iter()
                .filter(|c| c.name.starts_with(prefix))
                .collect(),
            None => Vec::new(),
        }
    };

    let on_cmd_select = {
        Callback::from(move |cmd: Commands| match cmd {
            Commands::Web => log::info!("Running - Web!"),
            Commands::Notes => log::info!("adding notes!"),
            Commands::Analysis => log::info!("Running - Analysis!"),
        })
    };

    let submit = {
        let input_ref = input_ref.clone();
        let history = history.clone();
        let is_loading = is_loading.clone();
        let input_text = input_text.clone();

        move || {
            if let Some(textarea) = input_ref.cast::<HtmlTextAreaElement>() {
                let value = textarea.value().trim().to_string();

                if !value.is_empty() {
                    let history = history.clone();
                    let is_loading = is_loading.clone();
                    is_loading.set(true);

                    textarea.set_value("");
                    let style = textarea.style();
                    let _ = style.set_property("height", "auto");
                    input_text.set(String::new());

                    wasm_bindgen_futures::spawn_local(async move {
                        info!("Prompt: {}", value.clone());
                        let response = match invoke_ask_ai(value.clone()).await {
                            Ok(res) => res,
                            Err(e) => {
                                log::error!("invoke_ask_ai error: {}", e);
                                format!("Error: {}", e)
                            }
                        };

                        let mut updated = (*history).clone();
                        updated.push(ChatTurn {
                            prompt: value,
                            response,
                        });
                        history.set(updated);
                        is_loading.set(false);
                    });
                }
            }
        }
    };

    let on_click = {
        let submit = submit.clone();
        Callback::from(move |_: MouseEvent| {
            submit.clone()();
        })
    };

    let on_keydown = {
        let submit = submit.clone();
        let input_ref = input_ref.clone();
        let input_text = input_text.clone();

        Callback::from(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                if !event.shift_key() {
                    event.prevent_default();
                    submit.clone()();
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
            history={(*history).clone()}
            is_loading={*is_loading}
            commands={visible_commands}
            on_cmd_select={on_cmd_select}
            on_input={on_input}
        />
    }
}
