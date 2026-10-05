use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;

use crate::api::history::{invoke_chat_delete, invoke_chat_list, invoke_chat_rename};
use crate::components::icons::{PencilIcon, TrashIcon};
use crate::hooks::use_window::refit_window;
use crate::models::chat::ChatSummary;
use crate::utils::time::{now_ms, time_ago};

#[derive(Properties, PartialEq)]
pub struct HistoryPanelProps {
    /// Only chats whose title contains this are listed; empty lists all.
    pub filter: String,
    pub on_open: Callback<i64>,
    pub on_close: Callback<()>,
    pub on_deleted: Callback<i64>,
}

/// The `/history` list. Keys: arrows move, Enter opens, F2 renames, Delete twice deletes,
/// Escape closes.
#[function_component(HistoryPanel)]
pub fn history_panel(props: &HistoryPanelProps) -> Html {
    let chats = use_state(Vec::<ChatSummary>::new);
    let loaded = use_state(|| false);
    let selected = use_state(|| 0usize);
    // Delete needs a second press on the same chat to confirm.
    let editing = use_state(|| None::<i64>);
    let delete_armed = use_state(|| None::<i64>);
    // Incremented to reload the list after a rename or delete.
    let reload = use_state(|| 0u32);
    let root_ref = use_node_ref();

    // Reload on open, on filter change and after `reload` changes.
    {
        let chats = chats.clone();
        let loaded = loaded.clone();
        let selected = selected.clone();
        use_effect_with((props.filter.clone(), *reload), move |(filter, _)| {
            let filter = filter.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_chat_list(&filter).await {
                    Ok(list) => {
                        if *selected >= list.len() {
                            selected.set(list.len().saturating_sub(1));
                        }
                        chats.set(list);
                        loaded.set(true);
                        refit_window();
                    }
                    Err(e) => log::warn!("could not load the chat list: {e}"),
                }
            });
            || ()
        });
    }

    // Focus the panel so the arrow keys work without a click.
    {
        let root_ref = root_ref.clone();
        use_effect_with((), move |_| {
            if let Some(root) = root_ref.cast::<HtmlElement>() {
                let _ = root.focus();
            }
            || ()
        });
    }

    let delete_chat = {
        let reload = reload.clone();
        let delete_armed = delete_armed.clone();
        let on_deleted = props.on_deleted.clone();
        Callback::from(move |id: i64| {
            delete_armed.set(None);
            let next = *reload + 1;
            let reload = reload.clone();
            let on_deleted = on_deleted.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_chat_delete(id).await {
                    Ok(()) => {
                        on_deleted.emit(id);
                        reload.set(next);
                    }
                    Err(e) => log::warn!("could not delete the chat: {e}"),
                }
            });
        })
    };

    let save_name = {
        let reload = reload.clone();
        let editing = editing.clone();
        let root_ref = root_ref.clone();
        Callback::from(move |(id, title): (i64, String)| {
            let next = *reload + 1;
            let reload = reload.clone();
            let editing = editing.clone();
            let root_ref = root_ref.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = invoke_chat_rename(id, &title).await {
                    log::warn!("could not rename the chat: {e}");
                }
                editing.set(None);
                reload.set(next);
                // Return the keyboard to the list.
                if let Some(root) = root_ref.cast::<HtmlElement>() {
                    let _ = root.focus();
                }
            });
        })
    };

    let on_keydown = {
        let chats = chats.clone();
        let selected = selected.clone();
        let editing = editing.clone();
        let delete_armed = delete_armed.clone();
        let delete_chat = delete_chat.clone();
        let on_open = props.on_open.clone();
        let on_close = props.on_close.clone();

        Callback::from(move |event: KeyboardEvent| {
            // The rename box handles its own keys.
            if editing.is_some() {
                return;
            }
            let count = chats.len();
            let current = chats.get(*selected).map(|chat| chat.id);

            match event.key().as_str() {
                "ArrowDown" if count > 0 => {
                    event.prevent_default();
                    selected.set((*selected + 1) % count);
                    delete_armed.set(None);
                }
                "ArrowUp" if count > 0 => {
                    event.prevent_default();
                    selected.set((*selected + count - 1) % count);
                    delete_armed.set(None);
                }
                "Enter" => {
                    event.prevent_default();
                    if let Some(id) = current {
                        on_open.emit(id);
                    }
                }
                "F2" => {
                    event.prevent_default();
                    editing.set(current);
                }
                "Delete" | "Backspace" => {
                    event.prevent_default();
                    if let Some(id) = current {
                        if *delete_armed == Some(id) {
                            delete_chat.emit(id);
                        } else {
                            delete_armed.set(Some(id));
                        }
                    }
                }
                "Escape" => {
                    event.prevent_default();
                    on_close.emit(());
                }
                _ => {}
            }
        })
    };

    let now = now_ms();

    html! {
        <div
            ref={root_ref.clone()}
            tabindex="0"
            onkeydown={on_keydown}
            class="w-full flex flex-col gap-0.5 max-h-[260px] overflow-y-auto cmd-scroll py-1 px-0.5 outline-none select-none"
        >
            <div class="flex items-center justify-between px-2 pb-1 text-[10px] text-neutral-500">
                <span class="uppercase tracking-widest">{"Chat history"}</span>
                <span>{"↑↓ move · Enter open · F2 rename · ⌫ delete · Esc close"}</span>
            </div>

            if chats.is_empty() {
                <div class="px-2 py-3 text-[12px] text-neutral-500">
                    { if !*loaded {
                        "Loading…".to_string()
                    } else if props.filter.trim().is_empty() {
                        "No saved chats yet.".to_string()
                    } else {
                        format!("No chat matches “{}”.", props.filter.trim())
                    } }
                </div>
            }

            { for chats.iter().enumerate().map(|(index, chat)| {
                let id = chat.id;
                let is_selected = index == *selected;
                let is_armed = *delete_armed == Some(id);

                let on_row_click = {
                    let on_open = props.on_open.clone();
                    Callback::from(move |_: MouseEvent| on_open.emit(id))
                };
                let on_hover = {
                    let selected = selected.clone();
                    Callback::from(move |_: MouseEvent| selected.set(index))
                };
                let on_edit = {
                    let editing = editing.clone();
                    Callback::from(move |event: MouseEvent| {
                        event.stop_propagation();
                        editing.set(Some(id));
                    })
                };
                let on_trash = {
                    let delete_armed = delete_armed.clone();
                    let delete_chat = delete_chat.clone();
                    Callback::from(move |event: MouseEvent| {
                        event.stop_propagation();
                        if *delete_armed == Some(id) {
                            delete_chat.emit(id);
                        } else {
                            delete_armed.set(Some(id));
                        }
                    })
                };

                html! {
                    <div
                        key={id}
                        onclick={on_row_click}
                        onmouseenter={on_hover}
                        class={classes!(
                            "group", "flex", "items-center", "gap-2", "rounded-md", "px-2", "py-1.5",
                            "cursor-pointer", "transition-colors",
                            is_selected.then_some("bg-white/[0.08]"),
                        )}
                    >
                        if *editing == Some(id) {
                            <RenameBox
                                initial={chat.title.clone()}
                                on_save={{
                                    let save_name = save_name.clone();
                                    Callback::from(move |title: String| save_name.emit((id, title)))
                                }}
                                on_cancel={{
                                    let editing = editing.clone();
                                    let root_ref = root_ref.clone();
                                    Callback::from(move |_: ()| {
                                        editing.set(None);
                                        if let Some(root) = root_ref.cast::<HtmlElement>() {
                                            let _ = root.focus();
                                        }
                                    })
                                }}
                            />
                        } else {
                            <span class="min-w-0 flex-1 truncate text-[12.5px] text-neutral-200">
                                { &chat.title }
                            </span>
                            <span class="shrink-0 text-[11px] text-neutral-500">
                                { time_ago(now, chat.updated_at) }
                            </span>
                            <button
                                onclick={on_edit}
                                title="Rename (F2)"
                                class="shrink-0 rounded p-1 text-neutral-500 opacity-0 transition-opacity hover:text-white group-hover:opacity-100"
                            >
                                <PencilIcon class="w-3 h-3" />
                            </button>
                            <button
                                onclick={on_trash}
                                title="Delete (press twice)"
                                class={classes!(
                                    "shrink-0", "flex", "items-center", "gap-1", "rounded", "p-1", "text-[11px]",
                                    "transition-opacity", "hover:text-red-300", "group-hover:opacity-100",
                                    if is_armed { "text-red-400 opacity-100" } else { "text-neutral-500 opacity-0" },
                                )}
                            >
                                <TrashIcon class="w-3 h-3" />
                                if is_armed {
                                    <span>{"Delete?"}</span>
                                }
                            </button>
                        }
                    </div>
                }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct RenameBoxProps {
    initial: String,
    on_save: Callback<String>,
    on_cancel: Callback<()>,
}

/// Inline rename field. Enter saves; Escape or blur cancels.
#[function_component(RenameBox)]
fn rename_box(props: &RenameBoxProps) -> Html {
    let input_ref = use_node_ref();

    // Start with the old name selected.
    {
        let input_ref = input_ref.clone();
        use_effect_with((), move |_| {
            if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                let _ = input.focus();
                input.select();
            }
            || ()
        });
    }

    let on_keydown = {
        let input_ref = input_ref.clone();
        let on_save = props.on_save.clone();
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |event: KeyboardEvent| {
            // Keep keys away from the list's own handler.
            event.stop_propagation();
            match event.key().as_str() {
                "Enter" => {
                    event.prevent_default();
                    if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                        on_save.emit(input.value());
                    }
                }
                "Escape" => {
                    event.prevent_default();
                    on_cancel.emit(());
                }
                _ => {}
            }
        })
    };
    let on_blur = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |_: FocusEvent| on_cancel.emit(()))
    };
    let on_click = Callback::from(|event: MouseEvent| event.stop_propagation());

    html! {
        <input
            ref={input_ref}
            type="text"
            value={props.initial.clone()}
            onkeydown={on_keydown}
            onblur={on_blur}
            onclick={on_click}
            class="min-w-0 flex-1 rounded border border-white/20 bg-black px-1.5 py-0.5 text-[12.5px] text-white outline-none focus:border-white/40"
        />
    }
}
