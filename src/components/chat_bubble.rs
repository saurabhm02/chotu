use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::api::clipboard::copy_to_clipboard;
use crate::components::icons::{CheckIcon, CopyIcon, GlobeIcon, RegenerateIcon};
use crate::utils::markdown::render_markdown_to_html;

// User message
#[derive(Properties, PartialEq)]
pub struct UserBubbleProps {
    pub prompt: String,
    pub timestamp: String,
}

#[function_component(UserBubble)]
pub fn user_bubble(props: &UserBubbleProps) -> Html {
    html! {
        <div class="flex justify-end w-full select-text">
            <div class="bg-[#1c1f26]/90 backdrop-blur-md text-neutral-100 rounded-xl border border-white/10 shadow-[0_2px_12px_rgba(0,0,0,0.3)] max-w-[80%] w-fit px-3 py-1.5 flex flex-col">
                <div class="text-[13px] leading-relaxed font-normal whitespace-pre-wrap break-words text-neutral-100">
                    {&props.prompt}
                </div>
                <div class="flex items-center justify-end gap-1 mt-0.5 text-[10px] text-neutral-500 font-mono select-none">
                    <span>{&props.timestamp}</span>
                    <CheckIcon class="w-2.5 h-2.5 text-neutral-400" />
                </div>
            </div>
        </div>
    }
}

// Assistant message
#[derive(Properties, PartialEq)]
pub struct AssistantCardProps {
    pub response: String,
    pub sources: Vec<(String, String)>,
    #[prop_or_default]
    pub on_regenerate: Option<Callback<()>>,
}

#[function_component(AssistantCard)]
pub fn assistant_card(props: &AssistantCardProps) -> Html {
    if props.response.is_empty() {
        return html! {};
    }

    html! {
        <div class="flex justify-start w-full select-text">
            <div class="w-full bg-transparent backdrop-blur-md rounded-xl border border-white/[0.08] shadow-[0_2px_16px_rgba(0,0,0,0.4)] px-3.5 py-2.5 sm:px-4 sm:py-3 flex flex-col transition-all">
                <MarkdownView
                    content={props.response.clone()}
                    on_regenerate={props.on_regenerate.clone()}
                />

                if !props.sources.is_empty() {
                    <SourcesChips sources={props.sources.clone()} />
                }
            </div>
        </div>
    }
}

// Parts of the assistant message
#[derive(Properties, PartialEq)]
pub struct ActionBarProps {
    pub text_to_copy: String,
    #[prop_or_default]
    pub on_regenerate: Option<Callback<()>>,
}

#[function_component(ActionBar)]
pub fn action_bar(props: &ActionBarProps) -> Html {
    let copied = use_state(|| false);

    let on_copy = {
        let copied = copied.clone();
        let text = props.text_to_copy.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            copy_to_clipboard(&text);
            copied.set(true);
            let copied_reset = copied.clone();
            Timeout::new(2000, move || {
                copied_reset.set(false);
            })
            .forget();
        })
    };

    let on_regen = {
        let on_regenerate = props.on_regenerate.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(ref cb) = on_regenerate {
                cb.emit(());
            }
        })
    };

    html! {
        <div class="flex items-center justify-end gap-1 text-neutral-200 select-none">
            <button
                onclick={on_copy}
                title="Copy response"
                class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer"
            >
                if *copied {
                    <CheckIcon class="w-3 h-3 text-neutral-200" />
                     // <span class="text-neutral-200 font-medium">{"Copied"}</span>
                 } else {
                    <CopyIcon class="w-3 h-3" />
                    // <span class="text-neutral-400 hover:text-neutral-200">{"Copy"}</span>
                }
            </button>

            if props.on_regenerate.is_some() {
                <button
                    onclick={on_regen}
                    title="Regenerate response"
                    class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer"
                >
                    <RegenerateIcon class="w-3 h-3" />
                    // <span class="text-neutral-400 hover:text-neutral-200">{"Regenerate"}</span>
                </button>
            }

            // <button
                // title="More options"
                // class="p-0.5 rounded text-[11px] hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer ml-auto text-neutral-500 hover:text-neutral-300"
            // >
                // <MoreHorizontalIcon class="w-3.5 h-3.5" />
            // </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SourcesChipsProps {
    pub sources: Vec<(String, String)>,
}

#[function_component(SourcesChips)]
pub fn sources_chips(props: &SourcesChipsProps) -> Html {
    if props.sources.is_empty() {
        return html! {};
    }

    html! {
        <div class="mt-3 pt-3 border-t border-white/[0.06] flex flex-col gap-1.5 select-none">
            <div class="flex items-center gap-1.5 text-[11px] font-medium text-neutral-400">
                <GlobeIcon class="w-3.5 h-3.5 text-neutral-400" />
                <span>{"Sources"}</span>
            </div>
            <div class="flex flex-wrap gap-1.5">
                { for props.sources.iter().enumerate().map(|(idx, (title, url))| {
                    let display_title = if title.len() > 32 {
                        format!("{}...", &title[..30])
                    } else {
                        title.clone()
                    };
                    let href = url.clone();
                    html! {
                        <a
                            key={idx}
                            href={href}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-white/[0.04] hover:bg-white/[0.08] border border-white/[0.08] text-[11px] text-neutral-300 hover:text-white transition-all max-w-[220px] truncate no-underline cursor-pointer"
                            title={format!("{}\n{}", title, url)}
                        >
                            <span class="font-mono text-neutral-500 font-semibold text-[10px]">{format!("[{}]", idx + 1)}</span>
                            <span class="truncate">{display_title}</span>
                        </a>
                    }
                }) }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MarkdownViewProps {
    pub content: String,
    #[prop_or_default]
    pub on_regenerate: Option<Callback<()>>,
}

#[function_component(MarkdownView)]
pub fn markdown_view(props: &MarkdownViewProps) -> Html {
    let html_content = render_markdown_to_html(&props.content);
    html! {
        <div class="w-full flex flex-col select-text">
            <div class="markdown-content w-full min-w-0">
                { Html::from_html_unchecked(AttrValue::from(html_content)) }
            </div>
            <ActionBar
                text_to_copy={props.content.clone()}
                on_regenerate={props.on_regenerate.clone()}
            />
        </div>
    }
}
