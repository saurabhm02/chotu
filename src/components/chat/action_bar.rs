use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::components::common::icons::{CheckIcon, CopyIcon, MoreHorizontalIcon, RegenerateIcon};
use crate::services::tauri_bridge::copy_to_clipboard;

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
        <div class="flex items-center gap-1 text-neutral-400 mt-2 pt-1.5 border-t border-white/[0.06] select-none">
            <button
                onclick={on_copy}
                title="Copy response"
                class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer"
            >
                if *copied {
                    <CheckIcon class="w-3 h-3 text-neutral-200" />
                    <span class="text-neutral-200 font-medium">{"Copied"}</span>
                } else {
                    <CopyIcon class="w-3 h-3" />
                    <span class="text-neutral-400 hover:text-neutral-200">{"Copy"}</span>
                }
            </button>

            if props.on_regenerate.is_some() {
                <button
                    onclick={on_regen}
                    title="Regenerate response"
                    class="flex items-center gap-1 px-1.5 py-0.5 rounded text-[11px] hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer"
                >
                    <RegenerateIcon class="w-3 h-3" />
                    <span class="text-neutral-400 hover:text-neutral-200">{"Regenerate"}</span>
                </button>
            }

            <button
                title="More options"
                class="p-0.5 rounded text-[11px] hover:bg-white/[0.08] hover:text-white transition-colors cursor-pointer ml-auto text-neutral-500 hover:text-neutral-300"
            >
                <MoreHorizontalIcon class="w-3.5 h-3.5" />
            </button>
        </div>
    }
}
