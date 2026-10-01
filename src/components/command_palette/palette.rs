use yew::prelude::*;

use crate::components::command_palette::item::CommandPaletteItem;
use crate::types::{Command, Commands};

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
