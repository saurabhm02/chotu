use pulldown_cmark::{html, Options, Parser};
use yew::prelude::*;

use crate::components::common::{
    ArrowUpIcon, DoubleChevronUpIcon, ExpandCornersIcon, InputCornerMarks, OuterCornerMarks,
};
use crate::components::loaders::LatticeLoader;
use crate::types::{ChatTurn, Command, Commands};

fn render_markdown(input: &str) -> Html {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    Html::from_html_unchecked(AttrValue::from(html_output))
}

#[derive(Properties, PartialEq)]
pub struct AskBarViewProps {
    pub input_ref: NodeRef,
    pub on_click: Callback<MouseEvent>,
    pub on_keydown: Callback<KeyboardEvent>,
    pub on_mousedown: Callback<MouseEvent>,
    pub history: Vec<ChatTurn>,
    pub is_loading: bool,
    pub on_input: Callback<InputEvent>,
    pub commands: Vec<Command>,
    pub on_cmd_select: Callback<Commands>,
}

#[function_component(AskBarView)]
pub fn ask_bar_view(props: &AskBarViewProps) -> Html {
    let is_expanded = !props.history.is_empty() || props.is_loading;

    html! {
      <div
        data-tauri-drag-region="true"
        onmousedown={props.on_mousedown.clone()}
        id="app-container"
        class={classes!(
          "relative", "bg-black", "text-white", "w-full", "flex", "flex-col", "justify-end", "box-border",
          if is_expanded { "p-3.5 gap-3 min-h-[120px]" } else { "p-2.5 gap-2" }
        )}
      >
        if is_expanded {
            <OuterCornerMarks />
        }

        if is_expanded {
            <div class="relative w-full flex-1 max-h-[400px] overflow-y-auto px-1 py-1">
                <div class="flex flex-col gap-2">
                    { for props.history.iter().map(|turn| html! {
                        <>
                            <div class="flex justify-end">
                                <div class="bg-[#323946] text-gray-100 rounded-lg px-3 py-1.5 max-w-[80%] text-sm">
                                    {turn.prompt.clone()}
                                </div>
                            </div>
                            <div class="flex justify-start">
                                <div class="markdown-content bg-[#424854] text-white rounded-lg px-3 py-1.5 max-w-[80%] text-sm">
                                    { render_markdown(&turn.response) }
                                </div>
                            </div>
                        </>
                    }) }
                    if props.is_loading {
                        <div class="flex justify-start py-1">
                            <LatticeLoader label="Thinking" show_timer=true />
                        </div>
                    }
                </div>
            </div>
        }

        if !props.commands.is_empty() {
            <div class="w-full">
                <div class="cmd-scroll max-h-28 overflow-y-auto rounded-lg border border-white/10 bg-white/[0.03] py-1">
                    { for props.commands.iter().map(|c| {
                        let cmd = c.cmd.clone();
                        let on_select = props.on_cmd_select.clone();
                        let onclick = Callback::from(move |_: MouseEvent| {
                            on_select.emit(cmd.clone());
                        });
                        html! {
                            <div
                                onclick={onclick}
                                class="flex items-baseline gap-2 px-3 py-1.5 cursor-pointer hover:bg-white/[0.06] transition-colors"
                            >
                                <span class="shrink-0 text-sm text-gray-100">{format!("/{}", c.name)}</span>
                                <span class="-translate-y-[3px] flex-1 border-b border-dotted border-gray-700"></span>
                                <span class="shrink-0 text-xs text-gray-500">{c.description}</span>
                            </div>
                        }
                    }) }
                </div>
            </div>
        }

        <div class="aura aura-dual w-full text-white/40 rounded-lg p-[1px] block">
            <div class="relative w-full bg-black rounded-lg">
                <InputCornerMarks />
                <div class="px-2.5 py-1.5 flex items-center gap-2 justify-between">
                    <div class="flex items-center justify-center w-6 shrink-0 text-gray-300">
                        <DoubleChevronUpIcon />
                    </div>

                    <div class="w-0.5 h-4 bg-gray-400 shrink-0 self-center"></div>

                    <div class="relative flex items-center flex-1 min-w-0">
                      <textarea
                        ref={props.input_ref.clone()}
                        onkeydown={props.on_keydown.clone()}
                        oninput={props.on_input.clone()}
                        id="input"
                        rows="1"
                        placeholder="Ask anything"
                        class="w-full bg-transparent border-none outline-none resize-none text-sm text-gray-100 placeholder-gray-500 px-1 py-0.5 leading-relaxed overflow-y-auto"
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
                            <button onclick={props.on_click.clone()} class="flex items-center justify-center rounded border border-white/30 hover:border-white/70 hover:bg-white/10 transition-colors p-1 text-white">
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
