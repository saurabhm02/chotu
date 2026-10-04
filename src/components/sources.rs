use yew::prelude::*;

use crate::api::opener::open_url;
use crate::hooks::use_window::refit_window;
use crate::models::stream::Phase;
use crate::utils::domain::{avatar_color, host_of, initial_of};

/// A round coloured letter for one site.
#[derive(Properties, PartialEq)]
pub struct AvatarProps {
    pub url: String,
    #[prop_or(18)]
    pub size: u32,
}

#[function_component(Avatar)]
pub fn avatar(props: &AvatarProps) -> Html {
    let host = host_of(&props.url);
    html! {
        <span
            class="inline-flex shrink-0 items-center justify-center rounded-full font-semibold text-black/70"
            style={format!(
                "width:{0}px;height:{0}px;font-size:{1}px;background:{2};",
                props.size,
                props.size * 11 / 20,
                avatar_color(&host)
            )}
        >
            { initial_of(&host) }
        </span>
    }
}

/// The numbered list: letter, title, site name. Clicking a row opens the page.
#[derive(Properties, PartialEq)]
pub struct SourcesListProps {
    pub sources: Vec<(String, String)>,
    /// Source number (starting at 1) to light up, for example while its `[n]` is hovered.
    #[prop_or_default]
    pub highlight: Option<usize>,
}

#[function_component(SourcesList)]
pub fn sources_list(props: &SourcesListProps) -> Html {
    html! {
        <div class="flex flex-col gap-0.5 select-none">
            <div class="px-1 pb-1 text-[10px] font-medium uppercase tracking-widest text-neutral-500">
                {"Sources"}
            </div>
            { for props.sources.iter().enumerate().map(|(i, (title, url))| {
                let link = url.clone();
                let onclick = Callback::from(move |_: MouseEvent| open_url(&link));
                html! {
                    <button
                        key={i}
                        {onclick}
                        title={url.clone()}
                        class={classes!(
                            "flex", "w-full", "items-center", "gap-2.5", "rounded-md", "px-1", "py-1",
                            "text-left", "hover:bg-white/[0.06]", "transition-colors", "cursor-pointer",
                            (props.highlight == Some(i + 1)).then_some("bg-white/[0.10]"),
                        )}
                    >
                        <span class="w-4 shrink-0 text-right text-[11px] text-neutral-500">{format!("{}.", i + 1)}</span>
                        <Avatar url={url.clone()} />
                        <span class="min-w-0 flex-1 truncate text-[12px] text-neutral-200">{title}</span>
                        <span class="shrink-0 text-[11px] text-neutral-500">{host_of(url)}</span>
                    </button>
                }
            }) }
        </div>
    }
}

/// "● ● ●  Reading sources (5)" above the answer while `/web` works.
/// Click it to open the list of sources.
#[derive(Properties, PartialEq)]
pub struct StatusStripProps {
    pub phase: Phase,
    pub sources: Vec<(String, String)>,
    /// True once the first word of the answer has arrived.
    pub answering: bool,
}

#[function_component(StatusStrip)]
pub fn status_strip(props: &StatusStripProps) -> Html {
    let open = use_state(|| false);
    let count = props.sources.len();
    let has_sources = count > 0;

    let label = match props.phase {
        Phase::Capturing => "Capturing screen".to_string(),
        Phase::ReadingText => "Reading text".to_string(),
        Phase::Searching => "Searching the web".to_string(),
        Phase::Reading => format!("Reading sources ({count})"),
        // Once the answer is flowing, a web answer goes back to "Reading sources".
        Phase::Thinking if props.answering && has_sources => format!("Reading sources ({count})"),
        Phase::Thinking => "Thinking".to_string(),
    };

    let toggle = {
        let open = open.clone();
        Callback::from(move |_: MouseEvent| {
            open.set(!*open);
            refit_window();
        })
    };

    html! {
        <div class="flex flex-col gap-1.5 px-1 py-0.5 select-none">
            <button
                onclick={toggle}
                disabled={!has_sources}
                class="inline-flex w-fit items-center gap-2 text-neutral-400 cursor-pointer disabled:cursor-default"
            >
                <span class="strip-dots"><i></i><i></i><i></i></span>
                if has_sources {
                    <svg
                        class={classes!("w-2.5", "h-2.5", "transition-transform", open.then_some("rotate-90"))}
                        viewBox="0 0 10 10" fill="currentColor"
                    >
                        <path d="M2 1l6 4-6 4z" />
                    </svg>
                }
                <span class="shimmer-text text-[12px] font-medium">{label}</span>
            </button>
            if *open && has_sources {
                <div class="pl-1">
                    <SourcesList sources={props.sources.clone()} />
                </div>
            }
        </div>
    }
}
