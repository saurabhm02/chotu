use crate::components::ask_bar::AskBar;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <AskBar />
    }
}
