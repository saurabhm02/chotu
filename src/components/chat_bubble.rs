use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::api::clipboard::copy_to_clipboard;
use crate::api::history::invoke_attachment_data_url;
use crate::api::opener::open_url;
use crate::hooks::use_window::refit_window;
use crate::components::icons::{CheckIcon, CopyIcon, RegenerateIcon};
use crate::components::sources::{Avatar, SourcesList};
use crate::utils::domain::short_model;
use crate::utils::time::format_duration;
use crate::utils::markdown::render_markdown_to_html;

// User message
#[derive(Properties, PartialEq)]
pub struct UserBubbleProps {
    pub prompt: String,
    pub timestamp: String,
    #[prop_or_default]
    pub quote: Option<String>,
    /// Paths of the stored images sent with the question.
    #[prop_or_default]
    pub attachments: Vec<String>,
}

#[derive(Properties, PartialEq)]
struct AttachmentImageProps {
    path: String,
}

#[function_component(AttachmentImage)]
fn attachment_image(props: &AttachmentImageProps) -> Html {
    let source = use_state(|| None::<String>);

    {
        let source = source.clone();
        use_effect_with(props.path.clone(), move |path| {
            let path = path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match invoke_attachment_data_url(&path).await {
                    Ok(url) => {
                        source.set(Some(url));
                        refit_window();
                    }
                    Err(e) => log::warn!("could not load {path}: {e}"),
                }
            });
            || ()
        });
    }

    match &*source {
        Some(url) => html! {
            <img
                src={url.clone()}
                class="mb-1 max-h-44 w-auto max-w-full rounded-lg border border-white/10 object-contain"
            />
        },
        None => html! {
            <div class="mb-1 h-16 w-28 rounded-lg border border-white/10 bg-white/[0.04]"></div>
        },
    }
}

#[function_component(UserBubble)]
pub fn user_bubble(props: &UserBubbleProps) -> Html {
    html! {
        <div class="flex justify-end w-full select-text">
            <div class="bg-[#1c1f26]/90 backdrop-blur-md text-neutral-100 rounded-xl border border-white/10 shadow-[0_2px_12px_rgba(0,0,0,0.3)] max-w-[80%] w-fit px-3 py-1.5 flex flex-col">
                { for props.attachments.iter().map(|path| html! {
                    <AttachmentImage key={path.clone()} path={path.clone()} />
                }) }
                if let Some(quote) = &props.quote {
                    <div class="mb-1 pl-2 border-l-2 border-red-400/70 italic text-[12px] leading-snug text-neutral-400 line-clamp-3 break-words">
                        { format!("“{quote}”") }
                    </div>
                }
                <div class="text-[13px] leading-relaxed font-normal whitespace-pre-wrap break-words text-neutral-100">
                    {&props.prompt}
                </div>
                <div class="flex items-center justify-end gap-1 mt-0.5 text-[10px] text-neutral-500 font-mono select-none">
                    <CopyButton text={props.prompt.clone()} title="Copy message" icon_class="w-2.5 h-2.5" />
                    <span>{&props.timestamp}</span>
                    <CheckIcon class="w-2.5 h-2.5 text-neutral-400" />
                </div>
            </div>
        </div>
    }
}

// A copy icon that turns into a check mark for two seconds after it is clicked.
#[derive(Properties, PartialEq)]
pub struct CopyButtonProps {
    pub text: String,
    #[prop_or("Copy")]
    pub title: &'static str,
    /// Size and colour of the icon.
    #[prop_or("w-3 h-3")]
    pub icon_class: &'static str,
}

#[function_component(CopyButton)]
pub fn copy_button(props: &CopyButtonProps) -> Html {
    let copied = use_state(|| false);

    let on_copy = {
        let copied = copied.clone();
        let text = props.text.clone();
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

    html! {
        <button
            onclick={on_copy}
            title={props.title}
            class="flex items-center px-1 py-0.5 rounded text-neutral-400 hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer"
        >
            if *copied {
                <CheckIcon class={props.icon_class} />
            } else {
                <CopyIcon class={props.icon_class} />
            }
        </button>
    }
}

// Assistant message
#[derive(Properties, PartialEq)]
pub struct AssistantCardProps {
    pub response: String,
    pub sources: Vec<(String, String)>,
    /// The model that answered (may be empty).
    #[prop_or_default]
    pub model: String,
    /// How long the answer took (milliseconds), once it is finished.
    #[prop_or_default]
    pub elapsed_ms: Option<u64>,
    #[prop_or_default]
    pub on_regenerate: Option<Callback<()>>,
}

#[function_component(AssistantCard)]
pub fn assistant_card(props: &AssistantCardProps) -> Html {
    let sources_open = use_state(|| false);
    // The `[n]` the mouse is over (starting at 1), so the matching source can light up.
    let hovered_citation = use_state_eq(|| None::<usize>);

    if props.response.is_empty() {
        return html! {};
    }

    let on_toggle_sources = {
        let sources_open = sources_open.clone();
        Callback::from(move |_| {
            sources_open.set(!*sources_open);
            refit_window();
        })
    };

    html! {
        <div class="flex justify-start w-full select-text">
            <div class="w-full bg-transparent backdrop-blur-md rounded-xl border border-white/[0.08] shadow-[0_2px_16px_rgba(0,0,0,0.4)] px-3.5 py-2.5 sm:px-4 sm:py-3 flex flex-col transition-all">
                <MarkdownView
                    content={props.response.clone()}
                    sources={props.sources.clone()}
                    on_hover={{
                        let hovered_citation = hovered_citation.clone();
                        Callback::from(move |n| hovered_citation.set(n))
                    }}
                />

                if *sources_open && !props.sources.is_empty() {
                    <div class="mt-2">
                        <SourcesList sources={props.sources.clone()} highlight={*hovered_citation} />
                    </div>
                }

                <ActionBar
                    text_to_copy={props.response.clone()}
                    sources={props.sources.clone()}
                    model={props.model.clone()}
                    elapsed_ms={props.elapsed_ms}
                    on_regenerate={props.on_regenerate.clone()}
                    on_toggle_sources={on_toggle_sources}
                />
            </div>
        </div>
    }
}

// Footer: copy | letters + "N sources" | model chip | regenerate
#[derive(Properties, PartialEq)]
pub struct ActionBarProps {
    pub text_to_copy: String,
    pub sources: Vec<(String, String)>,
    pub model: String,
    pub elapsed_ms: Option<u64>,
    #[prop_or_default]
    pub on_regenerate: Option<Callback<()>>,
    pub on_toggle_sources: Callback<()>,
}

#[function_component(ActionBar)]
pub fn action_bar(props: &ActionBarProps) -> Html {
    let on_regen = {
        let on_regenerate = props.on_regenerate.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(ref cb) = on_regenerate {
                cb.emit(());
            }
        })
    };

    let on_sources = {
        let toggle = props.on_toggle_sources.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            toggle.emit(());
        })
    };

    let count = props.sources.len();
    let button = "flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer";

    html! {
        <div class="mt-2 flex items-center gap-2 text-neutral-300 select-none">
            <CopyButton text={props.text_to_copy.clone()} title="Copy response" />

            // Regenerate sits right next to copy.
            if props.on_regenerate.is_some() {
                <button onclick={on_regen} title="Regenerate response" class={button}>
                    <RegenerateIcon class="w-3 h-3" />
                </button>
            }

            if count > 0 {
                <button onclick={on_sources} title="Show sources" class={button}>
                    <span class="flex items-center">
                        { for props.sources.iter().take(3).enumerate().map(|(i, (_, url))| html! {
                            <span key={i} class={classes!("rounded-full", "shadow-[0_0_0_1.5px_#000]", (i > 0).then_some("-ml-1.5"))}>
                                <Avatar url={url.clone()} size={16} />
                            </span>
                        }) }
                    </span>
                    <span class="text-neutral-400">
                        { if count == 1 { "1 source".to_string() } else { format!("{count} sources") } }
                    </span>
                </button>
            }

            if !props.model.is_empty() {
                <span
                    title={props.model.clone()}
                    class="inline-flex items-center gap-1.5 rounded-md border border-red-400 bg-transparent px-2 py-0.5 text-[11px] text-white"
                >
                    <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <rect x="6" y="6" width="12" height="12" rx="2" />
                        <path d="M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4" />
                    </svg>
                    { short_model(&props.model) }
                </span>
            }

            if let Some(ms) = props.elapsed_ms {
                <span class="font-mono text-[11px] tabular-nums text-neutral-500">{ format_duration(ms) }</span>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MarkdownViewProps {
    pub content: String,
    pub sources: Vec<(String, String)>,
    /// Told which `[n]` the mouse is over, or `None` when it is over something else.
    pub on_hover: Callback<Option<usize>>,
}

/// If the mouse event happened on a `[n]` citation, its number (starting at 1).
fn citation_number(event: &MouseEvent) -> Option<usize> {
    event
        .target_dyn_into::<web_sys::Element>()?
        .get_attribute("data-n")?
        .parse()
        .ok()
}

#[function_component(MarkdownView)]
pub fn markdown_view(props: &MarkdownViewProps) -> Html {
    let html_content = render_markdown_to_html(&props.content, props.sources.len());

    // One click handler for the whole answer: a click on a `[n]` opens source n.
    let on_click = {
        let sources = props.sources.clone();
        Callback::from(move |e: MouseEvent| {
            let source = citation_number(&e).and_then(|n| sources.get(n.checked_sub(1)?));
            if let Some((_, url)) = source {
                open_url(url);
            }
        })
    };

    let on_over = {
        let on_hover = props.on_hover.clone();
        Callback::from(move |e: MouseEvent| on_hover.emit(citation_number(&e)))
    };
    let on_leave = {
        let on_hover = props.on_hover.clone();
        Callback::from(move |_: MouseEvent| on_hover.emit(None))
    };

    html! {
        <div class="markdown-content w-full min-w-0 select-text" onclick={on_click} onmouseover={on_over} onmouseleave={on_leave}>
            { Html::from_html_unchecked(AttrValue::from(html_content)) }
        </div>
    }
}
