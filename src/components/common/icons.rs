use yew::prelude::*;

#[function_component(DoubleChevronUpIcon)]
pub fn double_chevron_up_icon() -> Html {
    html! {
        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none">
            <path d="M6 15L12 9L18 15" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M6 21L12 15L18 21" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

#[function_component(ExpandCornersIcon)]
pub fn expand_corners_icon() -> Html {
    html! {
        <svg class="w-[18px] h-[18px] text-white/70 hover:text-white transition-colors cursor-pointer" viewBox="0 0 24 24" fill="none">
            <path d="M8 3H5C3.895 3 3 3.895 3 5V8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
            <path d="M16 3H19C20.105 3 21 3.895 21 5V8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
            <path d="M3 16V19C3 20.105 3.895 21 5 21H8" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
            <path d="M21 16V19C21 20.105 20.105 21 19 21H16" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
        </svg>
    }
}

#[function_component(ArrowUpIcon)]
pub fn arrow_up_icon() -> Html {
    html! {
        <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 19V5"/>
            <path d="M5 12l7-7 7 7"/>
        </svg>
    }
}
