use crate::components::corner_marks::InputCornerMarks;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MiniIconProps {
    pub on_expand: Callback<MouseEvent>,
}

#[function_component(MiniIcon)]
pub fn mini_icon(props: &MiniIconProps) -> Html {
    html! {
        <div
            data-tauri-drag-region="true"
            onclick={props.on_expand.clone()}
            class="relative w-12 h-12 bg-black rounded-xl border border-white/10 flex items-center justify-center cursor-pointer select-none hover:scale-105 active:scale-95 transition-transform duration-200 group shadow-[0_4px_20px_rgba(0,0,0,0.6)]"
            title="Click or press Cmd + / to expand"
        >
            <InputCornerMarks />

            <div class="flex items-center justify-center font-bold text-sm tracking-tighter select-none pointer-events-none">
                <span class="text-white">{"T"}</span>
                <span class="text-[#c084fc]">{"Y"}</span>
            </div>
        </div>
    }
}
