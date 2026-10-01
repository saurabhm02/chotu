use yew::prelude::*;
use crate::components::common::icons::GlobeIcon;

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
