use yew::prelude::*;

#[function_component(OuterCornerMarks)]
pub fn outer_corner_marks() -> Html {
    html! {
        <>
            <div class="pointer-events-none absolute top-2 left-2 w-4 h-2.5 border-t-2 border-l-2 border-white"></div>
            <div class="pointer-events-none absolute top-2 right-2 w-4 h-2.5 border-t-2 border-r-2 border-white"></div>
            <div class="pointer-events-none absolute bottom-2 left-2 w-4 h-2.5 border-b-2 border-l-2 border-white"></div>
            <div class="pointer-events-none absolute bottom-2 right-2 w-4 h-2.5 border-b-2 border-r-2 border-white"></div>
        </>
    }
}

#[function_component(InputCornerMarks)]
pub fn input_corner_marks() -> Html {
    html! {
        <>
            <div class="pointer-events-none absolute top-0 left-0 w-3.5 h-2.5 border-t-2 border-l-2 border-white"></div>
            <div class="pointer-events-none absolute top-0 right-0 w-3.5 h-2.5 border-t-2 border-r-2 border-white"></div>
            <div class="pointer-events-none absolute bottom-0 left-0 w-3.5 h-2.5 border-b-2 border-l-2 border-white"></div>
            <div class="pointer-events-none absolute bottom-0 right-0 w-3.5 h-2.5 border-b-2 border-r-2 border-white"></div>
        </>
    }
}
