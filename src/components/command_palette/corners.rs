use yew::prelude::*;

#[function_component(SelectedRowCornerMarks)]
pub fn selected_row_corner_marks() -> Html {
    html! {
        <>
            <div class="pointer-events-none absolute top-0 right-0 w-3.5 h-2.5 border-t-2 border-r-2 border-white"></div>
            <div class="pointer-events-none absolute bottom-0 left-0 w-3.5 h-2.5 border-b-2 border-l-2 border-white"></div>
        </>
    }
}
