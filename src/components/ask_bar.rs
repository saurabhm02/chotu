use yew::prelude::*;

use crate::components::chat_bubble::{AssistantCard, UserBubble};
use crate::components::command_palette::CommandPalette;
use crate::components::corner_marks::{InputCornerMarks, OuterCornerMarks, SelectionCornerMarks};
use crate::components::history_panel::HistoryPanel;
use crate::components::icons::{ArrowUpIcon, DoubleChevronUpIcon, ExpandCornersIcon};
use crate::components::loader::LatticeLoader;
use crate::components::sources::StatusStrip;
use crate::components::mini_icon::MiniIcon;
use crate::hooks::{use_chat, use_input, use_selection, use_window};
use crate::models::{ChatTurn, Command, Commands};

/// The whole widget: wires the three hooks to the views below.
#[function_component(AskBar)]
pub fn ask_bar() -> Html {
    let input_ref = use_node_ref();
    let chat = use_chat();
    let selection = use_selection();

    // Send the question together with the highlighted text (if any), then clear it.
    let send_with_selection = {
        let send = chat.send.clone();
        let selected_text = selection.text.clone();
        let clear_selection = selection.clear.clone();

        Callback::from(move |question: String| {
            send.emit((question, selected_text.clone()));
            clear_selection.emit(());
        })
    };

    let input = use_input(input_ref.clone(), send_with_selection);
    let window = use_window(
        input_ref.clone(),
        (
            chat.history.clone(),
            chat.is_loading,
            input.text.clone(),
            input.commands.len(),
            selection.text.clone(),
            chat.history_panel.clone(),
        ),
    );
    let on_clear_selection = selection.clear.reform(|_: MouseEvent| ());

    // After the history list closes or opens a chat, the cursor goes back to the text box.
    let focus_input = {
        let input_ref = input_ref.clone();
        move || {
            if let Some(textarea) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
                let _ = textarea.focus();
            }
        }
    };
    let on_history_close = {
        let close = chat.close_panel.clone();
        let focus_input = focus_input.clone();
        Callback::from(move |_: ()| {
            close.emit(());
            focus_input();
        })
    };
    let on_history_open = {
        let open = chat.open_chat.clone();
        Callback::from(move |id: i64| {
            open.emit(id);
            focus_input();
        })
    };

    if window.is_mini {
        return html! {
            <div class="w-full h-full flex items-center justify-center p-0.5 animate-scale-in">
                <MiniIcon on_expand={window.on_expand} />
            </div>
        };
    }

    html! {
        <div class="w-full animate-fade-in">
            <AskBarView
                input_ref={input_ref}
                on_click={input.on_send_click}
                on_keydown={input.on_keydown}
                on_mousedown={window.on_mousedown}
                history={chat.history}
                is_loading={chat.is_loading}
                commands={input.commands}
                selected_cmd_index={input.selected_index}
                on_cmd_select={input.on_cmd_select}
                on_cmd_hover={input.on_cmd_hover}
                on_input={input.on_input}
                on_regenerate={chat.regenerate}
                history_filter={chat.history_panel.clone()}
                on_history_open={on_history_open}
                on_history_close={on_history_close}
                on_history_deleted={chat.forget_chat.clone()}
                selected_text={selection.text.clone()}
                on_clear_selection={on_clear_selection}
            />
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct AskBarViewProps {
    pub input_ref: NodeRef,
    pub on_click: Callback<MouseEvent>,
    pub on_keydown: Callback<KeyboardEvent>,
    pub on_mousedown: Callback<MouseEvent>,
    pub history: Vec<ChatTurn>,
    pub is_loading: bool,
    pub on_input: Callback<InputEvent>,
    pub commands: Vec<Command>,
    pub selected_cmd_index: usize,
    pub on_cmd_select: Callback<Commands>,
    pub on_cmd_hover: Callback<usize>,
    pub on_regenerate: Callback<usize>,
    /// `Some(filter)` while the `/history` list is open.
    pub history_filter: Option<String>,
    pub on_history_open: Callback<i64>,
    pub on_history_close: Callback<()>,
    pub on_history_deleted: Callback<i64>,
    pub selected_text: Option<String>,
    pub on_clear_selection: Callback<MouseEvent>,
}

#[function_component(AskBarView)]
fn ask_bar_view(props: &AskBarViewProps) -> Html {
    let is_expanded = !props.history.is_empty() || props.is_loading;

    // Keep the newest message in view while the answer streams in, unless the user
    // scrolled up to read something older.
    let scroll_ref = use_node_ref();
    let stuck_to_bottom = use_mut_ref(|| true);
    let last_turn_count = use_mut_ref(|| 0usize);

    let on_scroll = {
        let scroll_ref = scroll_ref.clone();
        let stuck_to_bottom = stuck_to_bottom.clone();
        Callback::from(move |_: Event| {
            if let Some(el) = scroll_ref.cast::<web_sys::Element>() {
                let from_bottom = el.scroll_height() - el.scroll_top() - el.client_height();
                *stuck_to_bottom.borrow_mut() = from_bottom < 40;
            }
        })
    };

    {
        let scroll_ref = scroll_ref.clone();
        use_effect_with(props.history.clone(), move |history| {
            // A new question always scrolls down.
            if history.len() != *last_turn_count.borrow() {
                *last_turn_count.borrow_mut() = history.len();
                *stuck_to_bottom.borrow_mut() = true;
            }
            if *stuck_to_bottom.borrow() {
                if let Some(el) = scroll_ref.cast::<web_sys::Element>() {
                    el.set_scroll_top(el.scroll_height());
                }
            }
            || ()
        });
    }

    // Until the first chunk arrives the answer is empty, so we're still "Thinking";
    // after that, text is streaming in.
    let is_streaming = props
        .history
        .last()
        .is_some_and(|turn| !turn.response.is_empty());
    let loader_label = if is_streaming {
        "Working on it"
    } else {
        "Thinking"
    }
    .to_string();

    // `/web` turns show the status strip instead of the plain loader.
    let last_has_status = props.history.last().is_some_and(|turn| turn.status.is_some());

    html! {
      <div
        data-tauri-drag-region="true"
        onmousedown={props.on_mousedown.clone()}
        id="app-container"
        class={classes!(
          "relative", "bg-black", "text-white", "w-full", "max-h-screen", "flex", "flex-col", "justify-end", "box-border", "pb-0.5", "select-none",
          if is_expanded { "gap-2.5" } else { "gap-1.5" }
        )}
      >
        if is_expanded {
            <OuterCornerMarks />
        }

        if is_expanded {
            <div ref={scroll_ref} onscroll={on_scroll} class="relative w-full flex-1 min-h-0 max-h-[700px] overflow-y-auto px-3 pt-3 space-y-2.5 cmd-scroll">
                { for props.history.iter().enumerate().map(|(idx, turn)| {
                    let on_regen = {
                        let on_regenerate = props.on_regenerate.clone();
                        Callback::from(move |_| {
                            on_regenerate.emit(idx);
                        })
                    };
                    let is_live = props.is_loading && idx + 1 == props.history.len();
                    html! {
                        <div key={idx} class="flex flex-col gap-2 w-full">
                           <UserBubble
                                prompt={turn.prompt.clone()}
                                quote={turn.quote.clone()}
                                attachments={turn.attachments.clone()}
                                timestamp={turn.timestamp.clone()}
                            />
                            if let (true, Some(phase)) = (is_live, turn.status) {
                                <StatusStrip
                                    phase={phase}
                                    sources={turn.sources.clone()}
                                    answering={!turn.response.is_empty()}
                                />
                            }
                            if !turn.response.is_empty() {
                                <AssistantCard
                                    response={turn.response.clone()}
                                    sources={turn.sources.clone()}
                                    model={turn.model.clone()}
                                    elapsed_ms={turn.elapsed_ms}
                                    on_regenerate={Some(on_regen)}
                                />
                            }
                        </div>
                    }
                }) }

                if props.is_loading && !last_has_status {
                    <div class="flex justify-start py-0.5">
                        <LatticeLoader label={loader_label} show_timer=true />
                    </div>
                }
            </div>
        }

        if let Some(filter) = &props.history_filter {
            <HistoryPanel
                filter={filter.clone()}
                on_open={props.on_history_open.clone()}
                on_close={props.on_history_close.clone()}
                on_deleted={props.on_history_deleted.clone()}
            />
        }

        if !props.commands.is_empty() {
            <CommandPalette
                commands={props.commands.clone()}
                selected_index={props.selected_cmd_index}
                on_select={props.on_cmd_select.clone()}
                on_hover={props.on_cmd_hover.clone()}
            />
        }


        <div class="aura aura-dual w-full text-white/40 rounded-lg p-[1px] block placeholder-base-100 ">
            <div class="relative w-full bg-black rounded-lg">
                <InputCornerMarks />

                // Selected text: sits inside the input frame, above the input row.
                if let Some(text) = &props.selected_text {
                    <div class="flex px-3 pt-3">
                        <div class="relative w-fit max-w-full min-w-0 pl-3 pr-6 py-1.5">
                            <SelectionCornerMarks />
                            <p class="italic text-[12px] leading-snug text-neutral-300 line-clamp-3 break-words">
                                { format!("“{text}”") }
                            </p>
                            // Sits where a top-right bracket would be.
                            <button
                                onclick={props.on_clear_selection.clone()}
                                title="Remove selected text"
                                class="absolute -top-0.5 right-0 z-20 text-[16px] leading-none text-white cursor-pointer"
                            >
                                {"×"}
                            </button>
                        </div>
                    </div>
                }

                <div class="px-2.5 py-1.5 flex items-center gap-2 justify-between">
                    <div class="flex items-center justify-center w-5 shrink-0 text-neutral-400 hover:text-white transition-colors cursor-pointer">
                        <DoubleChevronUpIcon />
                    </div>

                    <div class="w-[1px] h-3.5 bg-white/20 shrink-0 self-center"></div>

                    <div class="relative flex items-center flex-1 min-w-0">
                      <textarea
                        ref={props.input_ref.clone()}
                        onkeydown={props.on_keydown.clone()}
                        oninput={props.on_input.clone()}
                        id="input"
                        rows="1"
                        placeholder="Ask anything..."
                        class="w-full bg-transparent border-none outline-none resize-none text-[13.5px] text-neutral-100 placeholder-neutral-500 px-1 py-0.5 leading-relaxed overflow-y-auto"
                        style="min-height: 24px; max-height: 180px;"
                      />
                    </div>

                    <div class="flex items-center gap-2 shrink-0 self-center">
                        <ExpandCornersIcon />

                        if props.is_loading {
                            <div class="aura text-white rounded p-[1px] flex items-center justify-center">
                                <div class="flex items-center justify-center w-[24px] h-[24px] rounded border border-white/60 bg-black">
                                    <div class="w-3 h-3 rounded-[2px] border-[1.5px] border-white bg-transparent"></div>
                                </div>
                            </div>
                        } else {
                            <button
                                onclick={props.on_click.clone()}
                                class="flex items-center justify-center rounded border border-white/25 hover:border-white/70 hover:bg-white/10 bg-white/[0.04] transition-all p-1 text-white cursor-pointer"
                                title="Send message (Enter)"
                            >
                                <ArrowUpIcon />
                            </button>
                        }
                    </div>
                </div>
            </div>
        </div>
      </div>
    }
}
