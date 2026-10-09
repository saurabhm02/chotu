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
            class="relative w-12 h-12 rounded-xl cursor-pointer select-none hover:scale-105 active:scale-95 transition-transform duration-200 shadow-[0_4px_20px_rgba(0,0,0,0.6)]"
            title="Click or press Cmd + / to expand"
        >
            <img
                src="/public/chotu-mini.png"
                alt="Chotu"
                draggable="false"
                class="w-12 h-12 pointer-events-none select-none"
            />
        </div>
    }
}
