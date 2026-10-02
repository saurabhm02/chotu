use yew::prelude::*;

use crate::components::chat_bubble::{AssistantCard, UserBubble};
use crate::components::command_palette::CommandPalette;
use crate::components::corner_marks::{InputCornerMarks, OuterCornerMarks};
use crate::components::icons::{ArrowUpIcon, DoubleChevronUpIcon, ExpandCornersIcon};
use crate::components::loader::LatticeLoader;
use crate::components::mini_icon::MiniIcon;
use crate::hooks::{use_chat, use_input, use_window};
use crate::models::{ChatTurn, Command, Commands};

/// The whole widget: wires the three hooks to the views below.
#[function_component(AskBar)]
pub fn ask_bar() -> Html {
    let input_ref = use_node_ref();
    let chat = use_chat();
    let input = use_input(input_ref.clone(), chat.send.clone());
    let window = use_window(
        input_ref.clone(),
        (
            chat.history.clone(),
            chat.is_loading,
            input.text.clone(),
            input.commands.len(),
        ),
    );

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
}

#[function_component(AskBarView)]
fn ask_bar_view(props: &AskBarViewProps) -> Html {
    let is_expanded = !props.history.is_empty() || props.is_loading;

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

    html! {
      <div
        data-tauri-drag-region="true"
        onmousedown={props.on_mousedown.clone()}
        id="app-container"
        class={classes!(
          "relative", "bg-black", "text-white", "w-full", "flex", "flex-col", "justify-end", "box-border", "pb-0.5", "select-none",
          if is_expanded { "gap-2.5" } else { "gap-1.5" }
        )}
      >
        if is_expanded {
            <OuterCornerMarks />
        }

        if is_expanded {
            <div class="relative w-full flex-1 max-h-[420px] overflow-y-auto px-3 pt-3 space-y-2.5 cmd-scroll">
                { for props.history.iter().enumerate().map(|(idx, turn)| {
                    let on_regen = {
                        let on_regenerate = props.on_regenerate.clone();
                        Callback::from(move |_| {
                            on_regenerate.emit(idx);
                        })
                    };
                    html! {
                        <div key={idx} class="flex flex-col gap-2 w-full">
                            <UserBubble
                                prompt={turn.prompt.clone()}
                                timestamp={turn.timestamp.clone()}
                            />
                            if !turn.response.is_empty() {
                                <AssistantCard
                                    response={turn.response.clone()}
                                    sources={turn.sources.clone()}
                                    on_regenerate={Some(on_regen)}
                                />
                            }
                        </div>
                    }
                }) }

                if props.is_loading {
                    <div class="flex justify-start py-0.5">
                        <LatticeLoader label={loader_label} show_timer=true />
                    </div>
                }
            </div>
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
