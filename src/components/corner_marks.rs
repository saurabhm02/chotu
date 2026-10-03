use yew::prelude::*;

/// Top corners of the whole widget, flush with its edges. The bottom corners
/// come from `InputCornerMarks`, since the input box sits at the bottom edge.
#[function_component(OuterCornerMarks)]
pub fn outer_corner_marks() -> Html {
    html! {
        <>
            // <div class="pointer-events-none absolute top-0 left-0 w-3.5 h-2.5 border-t-[1.8px] border-l-[1.8px] border-white z-20"></div>
            // <div class="pointer-events-none absolute top-0 right-0 w-3.5 h-2.5 border-t-[1.8px] border-r-[1.8px] border-white z-20"></div>


            <div class="pointer-events-none absolute top-0 left-0 w-3.5 h-2.5 border-t-[1.8px] border-l-[1.8px] border-white z-20"></div>
            <div class="pointer-events-none absolute top-0 right-0 w-3.5 h-2.5 border-t-[1.8px] border-r-[1.8px] border-white z-20"></div>
            <div class="pointer-events-none absolute bottom-0 left-0 w-3.5 h-2.5 border-b-[1.8px] border-l-[1.8px] border-white z-20"></div>
            <div class="pointer-events-none absolute bottom-0 right-0 w-3.5 h-2.5 border-b-[1.8px] border-r-[1.8px] border-white z-20"></div>
        </>
    }
}

#[function_component(InputCornerMarks)]
pub fn input_corner_marks() -> Html {
    html! {
        <>
            <div class="pointer-events-none absolute top-0 left-0 w-3 h-2 border-t-[1.8px] border-l-[1.8px] border-white z-10"></div>
            <div class="pointer-events-none absolute top-0 right-0 w-3 h-2 border-t-[1.8px] border-r-[1.8px] border-white z-10"></div>
            <div class="pointer-events-none absolute bottom-0 left-0 w-3 h-2 border-b-[1.8px] border-l-[1.8px] border-white z-10"></div>
            <div class="pointer-events-none absolute bottom-0 right-0 w-3 h-2 border-b-[1.8px] border-r-[1.8px] border-white z-10"></div>
        </>
    }
}

/// Two small brackets (top-left and bottom-right) framing the selected-text quote.
#[function_component(SelectionCornerMarks)]
pub fn selection_corner_marks() -> Html {
    html! {
        <>
            <div class="pointer-events-none absolute top-0 left-0 w-2.5 h-2 border-t-[1.8px] border-l-[1.8px] border-red-400 z-10"></div>
            <div class="pointer-events-none absolute bottom-0 right-0 w-2.5 h-2 border-b-[1.8px] border-r-[1.8px] border-red-400 z-10"></div>
        </>
    }
}
