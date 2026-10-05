use yew::prelude::*;

use crate::components::icons::{
    ActivityIcon, BookTextIcon, ClockIcon, CodeIcon, CropIcon, GlobeIcon, PencilIcon, PlusIcon,
};
use crate::models::{Command, Commands};

#[derive(Properties, PartialEq)]
pub struct CommandPaletteProps {
    pub commands: Vec<Command>,
    pub selected_index: usize,
    pub on_select: Callback<Commands>,
    pub on_hover: Callback<usize>,
}

#[function_component(CommandPalette)]
pub fn command_palette(props: &CommandPaletteProps) -> Html {
    if props.commands.is_empty() {
        return html! {};
    }

    let on_select = props.on_select.clone();

    html! {
        <div class="w-full flex flex-col gap-1 max-h-[220px] overflow-y-auto cmd-scroll py-1 px-0.5 select-none bg-transparent">
            { for props.commands.iter().enumerate().map(|(idx, command)| {
                let is_active = idx == props.selected_index;
                let on_hover_cb = {
                    let on_hover = props.on_hover.clone();
                    Callback::from(move |_| on_hover.emit(idx))
                };
                html! {
                    <CommandPaletteItem
                        key={command.name}
                        command={command.clone()}
                        is_active={is_active}
                        on_select={on_select.clone()}
                        on_hover={on_hover_cb}
                    />
                }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct CommandPaletteItemProps {
    pub command: Command,
    pub is_active: bool,
    pub on_select: Callback<Commands>,
    pub on_hover: Callback<()>,
}

fn command_icon(cmd: &Commands) -> Html {
    match cmd {
        Commands::Web => html! { <GlobeIcon class="w-3.5 h-3.5" /> },
        Commands::Notes => html! { <BookTextIcon class="w-3.5 h-3.5" /> },
        Commands::Explain => html! { <CodeIcon class="w-3.5 h-3.5" /> },
        Commands::Analyze => html! { <ActivityIcon class="w-3.5 h-3.5" /> },
        Commands::Screen => html! { <CropIcon class="w-3.5 h-3.5" /> },
        Commands::New => html! { <PlusIcon class="w-3.5 h-3.5" /> },
        Commands::History => html! { <ClockIcon class="w-3.5 h-3.5" /> },
        Commands::Rename => html! { <PencilIcon class="w-3.5 h-3.5" /> },
    }
}

#[function_component(CommandPaletteItem)]
pub fn command_palette_item(props: &CommandPaletteItemProps) -> Html {
    let cmd = props.command.cmd;
    let on_select = props.on_select.clone();
    let on_click = Callback::from(move |e: MouseEvent| {
        e.stop_propagation();
        on_select.emit(cmd);
    });

    let on_hover = props.on_hover.clone();
    let on_mouseenter = Callback::from(move |_| {
        on_hover.emit(());
    });

    let is_active = props.is_active;

    let icon_class = if is_active {
        "text-white"
    } else {
        "text-neutral-400 group-hover:text-neutral-300"
    };

    let name_class = if is_active {
        "text-white"
    } else {
        "text-neutral-200 group-hover:text-white"
    };

    let desc_class = if is_active {
        "text-neutral-200"
    } else {
        "text-neutral-400 group-hover:text-neutral-300"
    };

    if is_active {
        html! {
            <div
                onclick={on_click}
                onmouseenter={on_mouseenter}
                class="aura aura-dual w-full text-white/40 rounded-lg p-[1px] block cursor-pointer select-none"
                style="--tw-duration: 6s;"
            >
                <div class="relative w-full bg-black rounded-lg px-3 py-1.5 flex items-center gap-3.5">
                    <SelectedRowCornerMarks />

                    <div class={classes!("flex", "items-center", "justify-center", "w-4", "h-4", "shrink-0", "transition-colors", icon_class)}>
                        { command_icon(&props.command.cmd) }
                    </div>

                    <span class={classes!("shrink-0", "text-xs", "font-mono", "font-medium", "tracking-wide", "transition-colors", name_class)}>
                        { format!("/{}", props.command.name) }
                    </span>

                    <span class={classes!("flex-1", "truncate", "text-xs", "font-normal", "transition-colors", desc_class)}>
                        { props.command.description }
                    </span>
                </div>
            </div>
        }
    } else {
        html! {
            <div
                onclick={on_click}
                onmouseenter={on_mouseenter}
                class="w-full rounded-lg p-[1px] block transition-all duration-100 cursor-pointer select-none group"
            >
                <div class="relative w-full bg-transparent hover:bg-white/[0.04] rounded-lg px-3 py-1.5 flex items-center gap-3.5">
                    <div class={classes!("flex", "items-center", "justify-center", "w-4", "h-4", "shrink-0", "transition-colors", icon_class)}>
                        { command_icon(&props.command.cmd) }
                    </div>

                    <span class={classes!("shrink-0", "text-xs", "font-mono", "font-medium", "tracking-wide", "transition-colors", name_class)}>
                        { format!("/{}", props.command.name) }
                    </span>

                    <span class={classes!("flex-1", "truncate", "text-xs", "font-normal", "transition-colors", desc_class)}>
                        { props.command.description }
                    </span>
                </div>
            </div>
        }
    }
}

#[function_component(SelectedRowCornerMarks)]
pub fn selected_row_corner_marks() -> Html {
    html! {
        <>
            <div class="pointer-events-none absolute top-0 right-0 w-3.5 h-2.5 border-t-2 border-r-2 border-white"></div>
            <div class="pointer-events-none absolute bottom-0 left-0 w-3.5 h-2.5 border-b-2 border-l-2 border-white"></div>
        </>
    }
}
