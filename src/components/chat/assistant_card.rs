use yew::prelude::*;

use crate::components::chat::action_bar::ActionBar;
use crate::components::chat::markdown_view::MarkdownView;
use crate::components::chat::sources_chips::SourcesChips;

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
            <div class="w-full bg-[#13151b]/90 backdrop-blur-md rounded-xl border border-white/[0.08] shadow-[0_2px_16px_rgba(0,0,0,0.4)] px-3.5 py-2.5 sm:px-4 sm:py-3 flex flex-col transition-all">
                <MarkdownView content={props.response.clone()} />

                if !props.sources.is_empty() {
                    <SourcesChips sources={props.sources.clone()} />
                }

                <ActionBar
                    text_to_copy={props.response.clone()}
                    on_regenerate={props.on_regenerate.clone()}
                />
            </div>
        </div>
    }
}
