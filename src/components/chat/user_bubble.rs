use yew::prelude::*;
use crate::components::common::icons::CheckIcon;

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
