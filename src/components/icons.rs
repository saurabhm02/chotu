use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct IconProps {
    #[prop_or_default]
    pub class: &'static str,
}

#[function_component(DoubleChevronUpIcon)]
pub fn double_chevron_up_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-5 h-5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M6 15L12 9L18 15" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M6 21L12 15L18 21" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
    }
}

#[function_component(ExpandCornersIcon)]
pub fn expand_corners_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() {
        "w-[18px] h-[18px] text-white/70 hover:text-white transition-colors cursor-pointer"
    } else {
        props.class
    };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M8 3H5C3.895 3 3 3.895 3 5V8" stroke-width="1.8" stroke-linecap="round"/>
            <path d="M16 3H19C20.105 3 21 3.895 21 5V8" stroke-width="1.8" stroke-linecap="round"/>
            <path d="M3 16V19C3 20.105 3.895 21 5 21H8" stroke-width="1.8" stroke-linecap="round"/>
            <path d="M21 16V19C21 20.105 20.105 21 19 21H16" stroke-width="1.8" stroke-linecap="round"/>
        </svg>
    }
}

#[function_component(ArrowUpIcon)]
pub fn arrow_up_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 19V5"/>
            <path d="M5 12l7-7 7 7"/>
        </svg>
    }
}

#[function_component(CheckIcon)]
pub fn check_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3 h-3" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20 6L9 17l-5-5"/>
        </svg>
    }
}

#[function_component(CopyIcon)]
pub fn copy_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
        </svg>
    }
}

#[function_component(RegenerateIcon)]
pub fn regenerate_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21.5 2v6h-6M21.34 15.57a10 10 0 1 1-.57-8.38l5.67-5.67"/>
        </svg>
    }
}

#[function_component(MoreHorizontalIcon)]
pub fn more_horizontal_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="currentColor">
            <circle cx="12" cy="12" r="1.5"/>
            <circle cx="19" cy="12" r="1.5"/>
            <circle cx="5" cy="12" r="1.5"/>
        </svg>
    }
}

#[function_component(GlobeIcon)]
pub fn globe_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"/>
            <line x1="2" y1="12" x2="22" y2="12"/>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
        </svg>
    }
}

#[function_component(BookTextIcon)]
pub fn book_text_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1-2.5-2.5Z"/>
            <path d="M8 7h6"/>
            <path d="M8 11h8"/>
        </svg>
    }
}

#[function_component(CodeIcon)]
pub fn code_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="16 18 22 12 16 6"/>
            <polyline points="8 6 2 12 8 18"/>
        </svg>
    }
}

#[function_component(ActivityIcon)]
pub fn activity_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M22 12h-4l-3 9L9 3l-3 9H2"/>
        </svg>
    }
}

#[function_component(CropIcon)]
pub fn crop_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3.5 h-3.5" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M6 2v14a2 2 0 0 0 2 2h14"/>
            <path d="M18 22V8a2 2 0 0 0-2-2H2"/>
        </svg>
    }
}

#[function_component(ReturnKeyIcon)]
pub fn return_key_icon(props: &IconProps) -> Html {
    let cls = if props.class.is_empty() { "w-3 h-3" } else { props.class };
    html! {
        <svg class={cls} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 10 4 15 9 20"/>
            <path d="M20 4v7a4 4 0 0 1-4 4H4"/>
        </svg>
    }
}
